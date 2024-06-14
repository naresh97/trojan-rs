use std::net::SocketAddr;

use anyhow::{anyhow, Result};

use crate::utils::advance_buffer;

pub struct Request {
    command: RequestCommand,
    destination: SocketAddr,
}

#[derive(PartialEq, Eq, Debug)]
pub enum RequestCommand {
    Connect,
    Bind,
    UdpAssociate,
}
impl RequestCommand {
    pub fn parse(buffer: &[u8]) -> Result<(RequestCommand, &[u8])> {
        let command = buffer
            .get(0..1)
            .ok_or(anyhow!("Buffer not long enought to get command"))?;

        let command = match command {
            [0x01] => Some(RequestCommand::Connect),
            [0x02] => Some(RequestCommand::Bind),
            [0x03] => Some(RequestCommand::UdpAssociate),
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
}

pub enum RequestAddressType {
    Ipv4,
    DomainName,
    Ipv6,
}

impl RequestAddressType {
    pub fn parse(buffer: &[u8]) -> Result<(RequestAddressType, &[u8])> {
        let address_type = buffer
            .first()
            .ok_or(anyhow!("Buffer not long enough to get address type"))?;
        let address_type = match address_type {
            0x01 => Some(RequestAddressType::Ipv4),
            0x03 => Some(RequestAddressType::DomainName),
            0x04 => Some(RequestAddressType::Ipv6),
            _ => None,
        }
        .ok_or(anyhow!("Unknown address type"))?;
        Ok((address_type, advance_buffer(1, buffer)?))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_parse_request_command() {
        assert!(RequestCommand::parse(&[]).is_err());
        assert!(RequestCommand::parse(&[123u8]).is_err());

        let (command, buffer) = RequestCommand::parse(&[0x1, 0x88]).unwrap();
        assert_eq!(RequestCommand::Connect, command);
        assert_eq!(0x88, *buffer.first().unwrap());
        assert_eq!(1, buffer.len());
    }
}
