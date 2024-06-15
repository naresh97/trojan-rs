use std::net::SocketAddr;

use anyhow::Result;
use tokio::{
    io::{AsyncRead, AsyncWrite, AsyncWriteExt},
    net::TcpStream,
};
use tokio_rustls::{client::TlsStream, TlsConnector};

use crate::{
    config::ClientConfig,
    dns::DnsResolver,
    socks5::{self, destination::Destination},
};

use super::protocol::TrojanHandshake;

pub struct TrojanClient {
    stream: TlsStream<TcpStream>,
}

impl TrojanClient {
    pub async fn write(
        payload: &[u8],
        destination: Destination,
        client_config: &ClientConfig,
        dns_resolver: &DnsResolver,
        connector: &TlsConnector,
    ) -> Result<TrojanClient> {
        let handshake = TrojanHandshake {
            password: client_config.password.clone(),
            command: socks5::request::RequestCommand::Connect,
            destination,
            payload: payload.to_vec(),
        };
        let handshake = handshake.as_bytes();
        let domain = client_config.server_domain.clone();
        let ip = dns_resolver.resolve(&domain).await?;
        let port = client_config.server_port;
        let address = SocketAddr::new(ip, port);
        let stream = TcpStream::connect(address).await?;
        let mut stream = connector.connect(domain.try_into()?, stream).await?;
        stream.write_all(&handshake).await?;
        Ok(TrojanClient { stream })
    }
}

impl Unpin for TrojanClient {}

impl AsyncRead for TrojanClient {
    fn poll_read(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        let pinned = std::pin::pin!(&mut self.get_mut().stream);
        pinned.poll_read(cx, buf)
    }
}

impl AsyncWrite for TrojanClient {
    fn poll_write(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<Result<usize, std::io::Error>> {
        let pinned = std::pin::pin!(&mut self.get_mut().stream);
        pinned.poll_write(cx, buf)
    }

    fn poll_flush(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), std::io::Error>> {
        let pinned = std::pin::pin!(&mut self.get_mut().stream);
        pinned.poll_flush(cx)
    }

    fn poll_shutdown(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), std::io::Error>> {
        let pinned = std::pin::pin!(&mut self.get_mut().stream);
        pinned.poll_shutdown(cx)
    }
}
