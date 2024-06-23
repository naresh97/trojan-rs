use std::net::SocketAddr;

use anyhow::Result;
use async_trait::async_trait;
use log::debug;
use tokio::{
    io::{copy_bidirectional, AsyncWriteExt},
    net::TcpStream,
};

use super::AsyncStream;

#[async_trait]
pub trait ForwardingClient: Send + Sync {
    async fn forward(&mut self, client_stream: &mut Box<dyn AsyncStream>) -> Result<()>;
    async fn write_buffer(&mut self, buffer: &[u8]) -> Result<()>;
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
}

#[async_trait]
impl ForwardingClient for SimpleForwardingClient {
    async fn forward(&mut self, client_stream: &mut Box<dyn AsyncStream>) -> Result<()> {
        copy_bidirectional(client_stream, &mut self.stream).await?;
        Ok(())
    }

    async fn write_buffer(&mut self, buffer: &[u8]) -> Result<()> {
        self.stream.write_all(buffer).await?;
        Ok(())
    }
}

pub struct SimpleForwardingClient {
    stream: TcpStream,
}
