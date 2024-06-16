use std::net::{SocketAddr, ToSocketAddrs};

use anyhow::{anyhow, Result};
use tokio::io::{AsyncRead, AsyncReadExt};

pub fn advance_buffer(length: usize, buffer: &[u8]) -> Result<&[u8]> {
    buffer
        .get(length..)
        .ok_or(anyhow!("Couldn't get remaning buffer"))
}

pub async fn read_to_buffer(stream: &mut (impl AsyncRead + Unpin)) -> Result<Vec<u8>> {
    const BUFFER_SIZE: usize = 0x1000;

    let mut buffer = Vec::with_capacity(BUFFER_SIZE);
    match stream.read_buf(&mut buffer).await {
        Ok(0) => Err(anyhow!("Socket closed")),
        Err(e) => Err(e.into()),
        Ok(n) => Ok(buffer[..n].to_vec()),
    }
}

pub fn as_socket_address(domain: &str, port: u16) -> Result<SocketAddr> {
    let mut address = (domain, port).to_socket_addrs()?;
    let address = address.next().ok_or(anyhow!(
        "Couldn't resolve socket address from ({},{})",
        domain,
        port
    ))?;
    Ok(address)
}
