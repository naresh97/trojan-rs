use anyhow::Result;
use log::debug;
use tokio::{
    io::{copy_bidirectional, AsyncRead, AsyncWrite, AsyncWriteExt},
    net::TcpStream,
};

use crate::{adapters::socks5, utils::as_socket_address};

pub struct SimpleForwardingClient {
    stream: TcpStream,
}

impl SimpleForwardingClient {
    pub async fn new(destination: socks5::protocol::Destination) -> Result<SimpleForwardingClient> {
        let (address, domain) = match destination {
            socks5::protocol::Destination::Address(ip) => (ip, ip.ip().to_string()),
            socks5::protocol::Destination::DomainName { domain, port } => {
                (as_socket_address(&domain, port)?, domain)
            }
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
