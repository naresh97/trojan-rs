use std::net::SocketAddr;

use anyhow::{anyhow, bail, Result};

use crate::utils::advance_buffer;

use super::destination::Destination;

pub struct Request {
    #[allow(unused)]
    pub command: Command,
    pub destination: Destination,
}

impl Request {
    pub fn parse(buffer: &[u8]) -> Result<(Request, &[u8])> {
        let version = *buffer.first().ok_or(anyhow!("Could not get version."))?;
        if version != 5 {
            bail!("Only SOCKS5 supported");
        }
        let buffer = advance_buffer(1, buffer)?;
        let (command, buffer) = Command::parse(buffer)?;

        let empty = *buffer.first().ok_or(anyhow!("Could not get RSV."))?;
        if empty != 0 {
            bail!("RSV must be 0");
        }
        let buffer = advance_buffer(1, buffer)?;
        let (destination, buffer) = Destination::parse(buffer)?;
        Ok((
            Request {
                command,
                destination,
            },
            buffer,
        ))
    }
}

#[derive(PartialEq, Eq, Debug)]
pub enum Command {
    Connect,
    Bind,
    UdpAssociate,
}

impl Command {
    pub fn parse(buffer: &[u8]) -> Result<(Command, &[u8])> {
        let command = buffer
            .get(0..1)
            .ok_or(anyhow!("Buffer not long enought to get command"))?;

        let command = match command {
            [0x01] => Some(Command::Connect),
            [0x02] => Some(Command::Bind),
            [0x03] => Some(Command::UdpAssociate),
            _ => None,
        }
        .ok_or(anyhow!("Unknown command type."))?;

        Ok((
            command,
            buffer
                .get(1..)
                .ok_or(anyhow!("Cannot return remaining buffer"))?,
        ))
    }
    pub fn as_byte(&self) -> u8 {
        match self {
            Command::Connect => 0x01,
            Command::Bind => 0x02,
            Command::UdpAssociate => 0x03,
        }
    }
}

pub enum AddressType {
    Ipv4,
    DomainName,
    Ipv6,
}

impl AddressType {
    pub fn parse(buffer: &[u8]) -> Result<(AddressType, &[u8])> {
        let address_type = buffer
            .first()
            .ok_or(anyhow!("Buffer not long enough to get address type"))?;
        let address_type = match address_type {
            0x01 => Some(AddressType::Ipv4),
            0x03 => Some(AddressType::DomainName),
            0x04 => Some(AddressType::Ipv6),
            _ => None,
        }
        .ok_or(anyhow!("Unknown address type"))?;
        Ok((address_type, advance_buffer(1, buffer)?))
    }
    pub fn as_byte(&self) -> u8 {
        match self {
            AddressType::Ipv4 => 0x01,
            AddressType::DomainName => 0x03,
            AddressType::Ipv6 => 0x04,
        }
    }
}

pub fn create_response(local_addr: &SocketAddr) -> Result<Vec<u8>> {
    let header = [5u8, 0, 0];
    let is_ipv4 = local_addr.is_ipv4();
    let atype = [if is_ipv4 { 1 } else { 4 }];
    let address = match local_addr.ip() {
        std::net::IpAddr::V4(ip) => ip.to_bits().to_be_bytes().to_vec(),
        std::net::IpAddr::V6(ip) => ip.to_bits().to_be_bytes().to_vec(),
    };
    let port = local_addr.port().to_be_bytes();
    let response = [
        header.as_slice(),
        atype.as_slice(),
        address.as_slice(),
        port.as_slice(),
    ]
    .concat();
    Ok(response)
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_parse_request_command() {
        assert!(Command::parse(&[]).is_err());
        assert!(Command::parse(&[123u8]).is_err());

        let (command, buffer) = Command::parse(&[0x1, 0x88]).unwrap();
        assert_eq!(Command::Connect, command);
        assert_eq!(0x88, *buffer.first().unwrap());
        assert_eq!(1, buffer.len());
    }
}
