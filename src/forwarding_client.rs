use std::net::SocketAddr;

use anyhow::{anyhow, Result};
use log::debug;
use tokio::{
    io::{copy, AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt},
    net::TcpStream,
};
use tokio_rustls::TlsConnector;

use crate::{dns::DnsResolver, socks5::destination::Destination, utils::BUFFER_SIZE};

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

    pub async fn forward(&mut self, buffer: &[u8]) -> Result<Vec<u8>> {
        debug!("Forwarding message length {}", buffer.len());
        self.stream.write_all(buffer).await?;
        let mut data = Vec::<u8>::new();
        loop {
            let mut buffer = Vec::with_capacity(BUFFER_SIZE);
            match self.stream.read_buf(&mut buffer).await {
                Ok(0) => break Err(anyhow!("Socket closed.")),
                Ok(n) => {
                    debug!("Received reply length {}", n);
                    buffer.shrink_to_fit();
                    data.append(&mut buffer);
                    if n <= BUFFER_SIZE {
                        break Ok(());
                    }
                }
                Err(e) => break Err(e.into()),
            }
        }?;
        Ok(data)
    }
    pub async fn forward_into_writer(
        &mut self,
        buffer: &[u8],
        mut writer: impl AsyncWrite + Unpin,
    ) -> Result<()> {
        debug!("Forwarding message length {}", buffer.len());
        self.stream.write_all(buffer).await?;
        copy(&mut self.stream, &mut writer).await?;
        Ok(())
    }
}
