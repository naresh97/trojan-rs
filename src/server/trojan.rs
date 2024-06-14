use std::net::SocketAddr;

use anyhow::{anyhow, Result};

pub fn parse_trojan_handshake(buffer: &[u8]) -> Result<&[u8]> {
    // let password = buffer
    //     .get(0..56)
    //     .ok_or(anyhow!("Buffer too short, couldn't get password."))?;

    // if !buffer
    //     .get(56..58)
    //     .map(|crfl| crfl == [0x0d, 0x0a])
    //     .ok_or(anyhow!("Buffer too short."))?
    // {
    //     return Err(anyhow!("CRLF after password not found"));
    // }

    // let command = buffer
    //     .get(58..59)
    //     .ok_or(anyhow!("Could not get command."))?;
    // let command = match command {
    //     [0x01] => Ok(Command::Connect),
    //     [0x03] => Ok(Command::UdpAssociate),
    //     _ => Err(anyhow!("Could not parse command")),
    // }?;

    todo!()
}
