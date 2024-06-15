use std::net::{Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6};

use anyhow::{anyhow, Result};

use crate::utils::advance_buffer;

use super::request::RequestAddressType;

#[derive(Debug, Clone)]
pub enum Destination {
    Address(SocketAddr),
    DomainName { domain: String, port: u16 },
}

impl Destination {
    pub fn parse(buffer: &[u8]) -> Result<(Destination, &[u8])> {
        let (address_type, buffer) = RequestAddressType::parse(buffer)?;
        match address_type {
            RequestAddressType::Ipv4 => parse_ipv4(buffer),
            RequestAddressType::DomainName => parse_domain_name(buffer),
            RequestAddressType::Ipv6 => parse_ipv6(buffer),
        }
    }
    pub fn as_bytes(&self) -> Vec<u8> {
        match self {
            Destination::Address(address) => match address {
                SocketAddr::V4(address) => {
                    let address_type = [RequestAddressType::Ipv4.as_byte()];
                    let ip = address.ip().to_bits().to_be_bytes();
                    let port = address.port().to_be_bytes();
                    [address_type.as_slice(), ip.as_slice(), port.as_slice()]
                        .concat()
                        .to_vec()
                }
                SocketAddr::V6(address) => {
                    let address_type = [RequestAddressType::Ipv6.as_byte()];
                    let ip = address.ip().to_bits().to_be_bytes();
                    let port = address.port().to_be_bytes();
                    [address_type.as_slice(), ip.as_slice(), port.as_slice()]
                        .concat()
                        .to_vec()
                }
            },
            Destination::DomainName { domain, port } => {
                let address_type = [RequestAddressType::DomainName.as_byte()];
                let domain = domain.as_bytes();
                let length = [domain.len() as u8];
                let port = port.to_be_bytes();
                [
                    address_type.as_slice(),
                    length.as_slice(),
                    domain,
                    port.as_slice(),
                ]
                .concat()
                .to_vec()
            }
        }
    }
}

fn parse_ipv4(buffer: &[u8]) -> Result<(Destination, &[u8])> {
    let address = buffer
        .get(0..4)
        .ok_or(anyhow!("Not long enough for IPv4 address"))?;
    let address = Ipv4Addr::from_bits(u32::from_be_bytes(address.try_into()?));
    let buffer = advance_buffer(4, buffer)?;
    let (port, buffer) = parse_port(buffer)?;
    Ok((
        Destination::Address(SocketAddrV4::new(address, port).into()),
        buffer,
    ))
}

fn parse_domain_name(buffer: &[u8]) -> Result<(Destination, &[u8])> {
    let length = buffer
        .first()
        .ok_or(anyhow!("Couldn't get Domain Name length"))?;
    let length = *length as usize;
    let buffer = advance_buffer(1, buffer)?;

    let domain = buffer
        .get(0..length)
        .ok_or(anyhow!("Buffer not long enough to contain domain name"))?;
    let domain = std::str::from_utf8(domain)?.to_string();
    let buffer = advance_buffer(length, buffer)?;

    let (port, buffer) = parse_port(buffer)?;

    Ok((Destination::DomainName { domain, port }, buffer))
}

fn parse_ipv6(buffer: &[u8]) -> Result<(Destination, &[u8])> {
    let ip = buffer
        .get(0..16)
        .ok_or(anyhow!("Not long enough for IPv4 address"))?;
    let ip = Ipv6Addr::from_bits(u128::from_be_bytes(ip.try_into()?));
    let buffer = advance_buffer(16, buffer)?;
    let (port, buffer) = parse_port(buffer)?;
    Ok((
        Destination::Address(SocketAddrV6::new(ip, port, 0, 0).into()),
        buffer,
    ))
}

fn parse_port(buffer: &[u8]) -> Result<(u16, &[u8])> {
    let port = buffer
        .get(..2)
        .ok_or(anyhow!("Buffer not long enough to contain port"))?;
    let port = u16::from_be_bytes(port.try_into()?);
    Ok((port, advance_buffer(2, buffer)?))
}

#[cfg(test)]
mod tests {

    use core::slice::SlicePattern;
    use std::net::SocketAddr;

    use super::*;

    #[tokio::test]
    async fn test_parse_ipv4() {
        let atype = [0x01];
        let ip = [8u8, 8, 8, 8];
        let port = 80u16.to_be_bytes();
        let payload = [1, 2, 3, 4];
        let buffer = [
            atype.as_slice(),
            ip.as_slice(),
            port.as_slice(),
            payload.as_slice(),
        ]
        .concat();
        let (destination, buffer) = Destination::parse(buffer.as_slice()).unwrap();
        let google1: SocketAddr = SocketAddrV4::new(Ipv4Addr::new(8, 8, 8, 8), 80).into();
        if let Destination::Address(ip) = destination {
            assert_eq!(google1, ip);
            assert_eq!(4, buffer.len());
            assert_eq!([1, 2, 3, 4], buffer);
        } else {
            panic!();
        }
    }

    #[tokio::test]
    async fn test_parse_domain() {
        let atype = [0x3u8];
        let name = "dns.google".as_bytes();
        let length = [name.len() as u8];
        let port = 80u16.to_be_bytes();
        let payload = [1, 2, 3, 4];
        let buffer = [
            atype.as_slice(),
            length.as_slice(),
            name.as_slice(),
            port.as_slice(),
            payload.as_slice(),
        ]
        .concat();
        let (ip, buffer) = Destination::parse(buffer.as_slice()).unwrap();
        if let Destination::DomainName { domain, port } = ip {
            assert_eq!(domain, "dns.google");
            assert_eq!(port, 80);
            assert_eq!(4, buffer.len());
            assert_eq!([1, 2, 3, 4], buffer.as_slice());
        } else {
            panic!()
        }
    }
}
