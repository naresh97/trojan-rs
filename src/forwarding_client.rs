use std::net::SocketAddr;

use anyhow::Result;
use log::debug;
use tokio::{
    io::{copy_bidirectional, AsyncRead, AsyncWrite},
    net::TcpStream,
};
use tokio_rustls::TlsConnector;

use crate::{dns::DnsResolver, socks5::destination::Destination};

pub struct ForwardingClient {
    stream: Box<dyn AsyncStream>,
    pub local_addr: SocketAddr,
}

trait AsyncStream: AsyncRead + AsyncWrite + Unpin + Send {}
impl<T: AsyncRead + AsyncWrite + Unpin + Send> AsyncStream for T {}

impl ForwardingClient {
    pub async fn new(
        connector: &TlsConnector,
        dns_resolver: &DnsResolver,
        destination: Destination,
        use_tls: bool,
    ) -> Result<ForwardingClient> {
        let (ip, domain) = match destination {
            Destination::Ip(ip) => (ip, ip.ip().to_string()),
            Destination::DomainName { domain, port } => (
                SocketAddr::new(dns_resolver.resolve(&domain).await?, port),
                domain,
            ),
        };
        debug!(
            "Creating new forwarding socket with IP: {}, Domain: {}",
            ip, domain
        );
        let stream = TcpStream::connect(ip).await?;
        let local_addr = stream.local_addr()?;
        let stream: Box<dyn AsyncStream> = if use_tls {
            Box::new(connector.connect(domain.try_into()?, stream).await?)
        } else {
            Box::new(stream)
        };
        Ok(ForwardingClient { stream, local_addr })
    }

    pub async fn forward(&mut self, client_stream: &mut TcpStream) -> Result<()> {
        copy_bidirectional(client_stream, &mut self.stream).await?;
        Ok(())
    }
}
