use std::net::SocketAddr;

use anyhow::{anyhow, Result};
use log::debug;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};
use tokio_rustls::TlsConnector;

use crate::{dns::DnsResolver, socks5::destination::Destination};

pub struct ForwardingClient {
    stream: tokio_rustls::client::TlsStream<TcpStream>,
}

impl ForwardingClient {
    pub async fn new(
        connector: &TlsConnector,
        dns_resolver: &DnsResolver,
        destination: Destination,
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
        let stream = connector.connect(domain.try_into()?, stream).await?;
        Ok(ForwardingClient { stream })
    }

    pub async fn forward(&mut self, buffer: &[u8]) -> Result<Vec<u8>> {
        debug!(
            "Forwarding payload: {:?}",
            std::str::from_utf8(buffer)
                .map(|x| x.to_string())
                .unwrap_or(format!("{:?}", buffer))
        );
        self.stream.write_all(buffer).await?;
        let mut buffer = Vec::with_capacity(0x1000);
        match self.stream.read(&mut buffer).await {
            Ok(0) => Err(anyhow!("Socket closed.")),
            Ok(n) => Ok(buffer[..n].to_vec()),
            Err(e) => Err(e.into()),
        }
    }
}
