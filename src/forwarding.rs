use std::net::SocketAddr;

use anyhow::Result;
use log::debug;
use tokio::{
    io::{copy_bidirectional, AsyncRead, AsyncWrite, AsyncWriteExt},
    net::TcpStream,
};
use tokio_rustls::TlsConnector;

use crate::{dns::DnsResolver, socks5::destination::Destination};

pub struct SimpleForwardingClient {
    stream: Box<dyn AsyncStream>,
}

trait AsyncStream: AsyncRead + AsyncWrite + Unpin + Send {}
impl<T: AsyncRead + AsyncWrite + Unpin + Send> AsyncStream for T {}

impl SimpleForwardingClient {
    pub async fn new(
        connector: &TlsConnector,
        dns_resolver: &DnsResolver,
        destination: Destination,
        use_tls: bool,
    ) -> Result<SimpleForwardingClient> {
        let (address, domain) = match destination {
            Destination::Address(ip) => (ip, ip.ip().to_string()),
            Destination::DomainName { domain, port } => (
                SocketAddr::new(dns_resolver.resolve(&domain).await?, port),
                domain,
            ),
        };
        debug!(
            "Creating new forwarding socket with IP: {}, Domain: {}",
            address, domain
        );
        let stream = TcpStream::connect(address).await?;
        let stream: Box<dyn AsyncStream> = if use_tls {
            Box::new(connector.connect(domain.try_into()?, stream).await?)
        } else {
            Box::new(stream)
        };
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
