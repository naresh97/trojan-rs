use std::net::{Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6};

use anyhow::{anyhow, Result};

use crate::{dns::DnsResolver, utils::advance_buffer};

pub struct Request {
    command: RequestCommand,
    destination: SocketAddr,
}

pub fn parse_request_command(buffer: &[u8]) -> Result<(RequestCommand, &[u8])> {
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

pub async fn parse_request_destination<'a>(
    dns_resolver: &DnsResolver,
    buffer: &'a [u8],
) -> Result<(SocketAddr, &'a [u8])> {
    let (address_type, buffer) = parse_request_address_type(buffer)?;
    match address_type {
        RequestAddressType::Ipv4 => parse_ipv4(buffer),
        RequestAddressType::DomainName => parse_domain_name(dns_resolver, buffer).await,
        RequestAddressType::Ipv6 => parse_ipv6(buffer),
    }
}

fn parse_ipv6(buffer: &[u8]) -> Result<(SocketAddr, &[u8])> {
    let ip = buffer
        .get(0..16)
        .ok_or(anyhow!("Not long enough for IPv4 address"))?;
    let ip = Ipv6Addr::from_bits(u128::from_be_bytes(ip.try_into()?));
    let buffer = advance_buffer(16, buffer)?;
    let (port, buffer) = parse_port(buffer)?;
    Ok((SocketAddrV6::new(ip, port, 0, 0).into(), buffer))
}

async fn parse_domain_name<'a>(
    dns_resolver: &DnsResolver,
    buffer: &'a [u8],
) -> Result<(SocketAddr, &'a [u8])> {
    let length = buffer
        .first()
        .ok_or(anyhow!("Couldn't get Domain Name length"))?;
    let length = *length as usize;
    let buffer = advance_buffer(1, buffer)?;

    let domain_name = buffer
        .get(0..length)
        .ok_or(anyhow!("Buffer not long enough to contain domain name"))?;
    let domain_name = std::str::from_utf8(domain_name)?;
    let ip = dns_resolver.resolve(domain_name).await?;
    let buffer = advance_buffer(length, buffer)?;

    let (port, buffer) = parse_port(buffer)?;

    let socket_address = SocketAddr::new(ip, port);

    Ok((socket_address, buffer))
}

fn parse_ipv4(buffer: &[u8]) -> Result<(SocketAddr, &[u8])> {
    let address = buffer
        .get(0..4)
        .ok_or(anyhow!("Not long enough for IPv4 address"))?;
    let address = Ipv4Addr::from_bits(u32::from_be_bytes(address.try_into()?));
    let buffer = advance_buffer(4, buffer)?;
    let (port, buffer) = parse_port(buffer)?;
    Ok((SocketAddrV4::new(address, port).into(), buffer))
}

fn parse_port(buffer: &[u8]) -> Result<(u16, &[u8])> {
    let port = buffer
        .get(..2)
        .ok_or(anyhow!("Buffer not long enough to contain port"))?;
    let port = u16::from_be_bytes(port.try_into()?);
    Ok((port, advance_buffer(2, buffer)?))
}

fn parse_request_address_type(buffer: &[u8]) -> Result<(RequestAddressType, &[u8])> {
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

#[derive(PartialEq, Eq, Debug)]
pub enum RequestCommand {
    Connect,
    Bind,
    UdpAssociate,
}

pub enum RequestAddressType {
    Ipv4,
    DomainName,
    Ipv6,
}

#[cfg(test)]
mod tests {

    use core::slice::SlicePattern;

    use super::*;

    #[test]
    fn test_parse_request_command() {
        assert!(parse_request_command(&[]).is_err());
        assert!(parse_request_command(&[123u8]).is_err());

        let (command, buffer) = parse_request_command(&[0x1, 0x88]).unwrap();
        assert_eq!(RequestCommand::Connect, command);
        assert_eq!(0x88, *buffer.first().unwrap());
        assert_eq!(1, buffer.len());
    }

    #[tokio::test]
    async fn test_parse_ipv4() {
        let dns_resolver = DnsResolver::new().await;

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
        let (destination, buffer) = parse_request_destination(&dns_resolver, buffer.as_slice())
            .await
            .unwrap();
        let google1: SocketAddr = SocketAddrV4::new(Ipv4Addr::new(8, 8, 8, 8), 80).into();
        assert_eq!(google1, destination);
        assert_eq!(4, buffer.len());
        assert_eq!([1, 2, 3, 4], buffer);
    }

    #[tokio::test]
    async fn test_parse_dns() {
        let dns_resolver = DnsResolver::new().await;
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
        let (ip, buffer) = parse_request_destination(&dns_resolver, buffer.as_slice())
            .await
            .unwrap();
        let google1 = SocketAddrV4::new(Ipv4Addr::new(8, 8, 8, 8), 80).into();
        let google2 = SocketAddrV4::new(Ipv4Addr::new(8, 8, 4, 4), 80).into();
        assert!(ip == google1 || ip == google2);
        assert_eq!(4, buffer.len());
        assert_eq!([1, 2, 3, 4], buffer.as_slice());
    }
}
