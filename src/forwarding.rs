use anyhow::Result;
use log::debug;
use tokio::{
    io::{copy_bidirectional, AsyncRead, AsyncWrite, AsyncWriteExt},
    net::TcpStream,
};

use crate::{socks5::destination::Destination, utils::as_socket_address};

pub struct SimpleForwardingClient {
    stream: TcpStream,
}

impl SimpleForwardingClient {
    pub async fn new(destination: Destination) -> Result<SimpleForwardingClient> {
        let (address, domain) = match destination {
            Destination::Address(ip) => (ip, ip.ip().to_string()),
            Destination::DomainName { domain, port } => (as_socket_address(&domain, port)?, domain),
        };
        debug!(
            "Creating new forwarding socket with IP: {}, Domain: {}",
            address, domain
        );
        let stream = TcpStream::connect(address).await?;
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
