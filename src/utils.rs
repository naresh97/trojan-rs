use anyhow::{anyhow, Result};
use tokio::io::{AsyncRead, AsyncReadExt};

pub fn advance_buffer(length: usize, buffer: &[u8]) -> Result<&[u8]> {
    buffer
        .get(length..)
        .ok_or(anyhow!("Couldn't get remaning buffer"))
}

pub const BUFFER_SIZE: usize = 0x1000;
pub const CRLF: [u8; 2] = [0x0D, 0x0A];

pub async fn read_to_buffer(stream: &mut (impl AsyncRead + Unpin)) -> Result<Vec<u8>> {
    let mut buffer = Vec::with_capacity(BUFFER_SIZE);
    match stream.read_buf(&mut buffer).await {
        Ok(0) => Err(anyhow!("Socket closed")),
        Err(e) => Err(e.into()),
        Ok(n) => Ok(buffer[..n].to_vec()),
    }
}
