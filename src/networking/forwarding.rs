use std::net::SocketAddr;

use anyhow::Result;
use log::debug;
use tokio::{
    io::{copy_bidirectional, AsyncRead, AsyncWrite, AsyncWriteExt},
    net::TcpStream,
};

pub struct SimpleForwardingClient {
    stream: TcpStream,
}

impl SimpleForwardingClient {
    pub async fn new(destination: &SocketAddr) -> Result<SimpleForwardingClient> {
        debug!(
            "Creating new forwarding socket with address: {}",
            destination
        );
        let stream = TcpStream::connect(destination).await?;
        Ok(SimpleForwardingClient { stream })
    }
    pub async fn forward(
        &mut self,
        client_stream: &mut (impl AsyncRead + AsyncWrite + Unpin),
    ) -> Result<()> {
        copy_bidirectional(client_stream, &mut self.stream).await?;
        Ok(())
    }

    pub async fn write_buffer(&mut self, buffer: &[u8]) -> Result<()> {
        self.stream.write_all(buffer).await?;
        Ok(())
    }
}
