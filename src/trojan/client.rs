use std::net::SocketAddr;

use crate::{
    config::ClientConfig,
    socks5::{self, destination::Destination},
    utils::as_socket_address,
};
use anyhow::Result;
use tokio::{
    io::{copy_bidirectional, AsyncRead, AsyncWrite, AsyncWriteExt},
    net::TcpStream,
};
use tokio_native_tls::{TlsConnector, TlsStream};

use super::protocol::{hash_password, TrojanHandshake};

pub struct TrojanClient {
    stream: TlsStream<TcpStream>,
    destination: Destination,
    pub local_addr: SocketAddr,
}

impl TrojanClient {
    pub async fn new(
        destination: Destination,
        client_config: &ClientConfig,
        connector: &TlsConnector,
    ) -> Result<TrojanClient> {
        let domain = client_config.server_domain.clone();
        let port = client_config.server_port;
        let address = as_socket_address(&domain, port)?;
        let stream = TcpStream::connect(address).await?;
        let local_addr = stream.local_addr()?;
        let stream = connector.connect(&domain, stream).await?;
        Ok(TrojanClient {
            stream,
            destination,
            local_addr,
        })
    }

    pub async fn send_handshake(
        &mut self,
        payload: &[u8],
        client_config: &ClientConfig,
    ) -> Result<()> {
        let handshake = TrojanHandshake {
            password: hash_password(&client_config.password),
            command: socks5::request::RequestCommand::Connect,
            destination: self.destination.clone(),
            payload: payload.to_vec(),
        };
        let handshake = handshake.as_bytes();
        self.stream.write_all(&handshake).await?;
        Ok(())
    }

    pub async fn forward(
        &mut self,
        client_stream: &mut (impl AsyncRead + AsyncWrite + Unpin),
    ) -> std::result::Result<(), anyhow::Error> {
        copy_bidirectional(client_stream, &mut self.stream).await?;
        Ok(())
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
