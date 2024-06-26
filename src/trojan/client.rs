use std::net::{SocketAddr, ToSocketAddrs};

use crate::{
    adapters::socks5, config::ClientConfig, networking::AsyncStream,
    trojan::websocket::WebsocketWrapper,
};
use anyhow::{Context, Result};
use log::debug;
use tokio::{
    io::{copy_bidirectional, AsyncRead, AsyncWrite, AsyncWriteExt},
    net::TcpStream,
};
use tokio_native_tls::{TlsConnector, TlsStream};

use super::{hash_password, protocol::TrojanHandshake};

pub struct TrojanClient {
    stream: Box<dyn AsyncStream>,
    destination: socks5::protocol::Destination,
    hashed_password: String,
    pub local_addr: SocketAddr,
}

impl TrojanClient {
    pub async fn new(
        destination: socks5::protocol::Destination,
        client_config: &ClientConfig,
        connector: &TlsConnector,
    ) -> Result<TrojanClient> {
        let (domain, stream) = create_tcp_stream(client_config).await?;
        let local_addr = stream.local_addr()?;

        let tls_stream = connector.connect(domain, stream).await?;

        let stream: Box<dyn AsyncStream> =
            if let Some(websocket_path) = &client_config.websocket_path {
                create_websocket_stream(tls_stream, websocket_path).await?
            } else {
                Box::new(tls_stream)
            };

        let hashed_password = client_config
            .hashed_password
            .clone()
            .unwrap_or(hash_password(&client_config.password));

        Ok(TrojanClient {
            stream,
            destination,
            local_addr,
            hashed_password,
        })
    }

    pub async fn send_handshake(&mut self, payload: &[u8]) -> Result<()> {
        let handshake = TrojanHandshake {
            hashed_password: self.hashed_password.clone(),
            command: socks5::protocol::request::Command::Connect,
            destination: self.destination.clone(),
            payload: payload.to_vec(),
        };
        let handshake = handshake.as_bytes();
        self.stream.write_all(&handshake).await?;
        debug!("Handshake sent");
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

async fn create_websocket_stream(
    stream: TlsStream<TcpStream>,
    websocket_path: &str,
) -> Result<Box<WebsocketWrapper>, anyhow::Error> {
    #[cfg(feature = "websockets")]
    {
        let (stream, _response) = tokio_tungstenite::client_async(websocket_path, stream)
            .await
            .context("Could not initialize WebSocketStream")?;
        let stream = WebsocketWrapper::new(stream);
        Ok(Box::new(stream))
    }
    #[cfg(not(feature = "websockets"))]
    panic!("Not compiled with websocket support.");
}

async fn create_tcp_stream(
    client_config: &ClientConfig,
) -> Result<(&str, TcpStream), anyhow::Error> {
    let domain = client_config
        .server_addr
        .split(':')
        .next()
        .context("Couldn't get domain from address string")?;
    let address = client_config
        .server_addr
        .to_socket_addrs()
        .context("Couldn't get SocketAddr from address")
        .and_then(|mut x| {
            x.find(|x| x.is_ipv4())
                .context("Couldn't get SocketAddr from address")
        })?;
    let stream = TcpStream::connect(address)
        .await
        .with_context(|| format!("Couldn't connect to address: {}", address))?;
    Ok((domain, stream))
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
