use crate::{
    adapters::socks5,
    config::ServerConfig,
    networking::{
        forwarding::{ForwardingClient, SimpleForwardingClient},
        AsyncStream,
    },
    trojan::{
        protocol::{hash_password, TrojanHandshake},
        websocket::WebsocketWrapper,
    },
    utils::read_to_buffer,
};
use anyhow::{anyhow, bail, Result};
use log::debug;
use tokio::{net::TcpStream, sync::oneshot};
use tokio_native_tls::TlsStream;
use tokio_tungstenite::tungstenite::handshake;

pub async fn handle_socket(
    server_config: &ServerConfig,
    stream: TlsStream<TcpStream>,
) -> Result<()> {
    let mut socket_state = SocketState::WaitingForHandshake;

    let mut stream: Box<dyn AsyncStream> =
        if let Some(websocket_path) = &server_config.websocket_path {
            create_websocket(stream, websocket_path).await?
        } else {
            Box::new(stream)
        };

    loop {
        match &mut socket_state {
            SocketState::WaitingForHandshake => {
                handle_handshake(&mut socket_state, &mut stream, server_config).await?
            }
            SocketState::Open(forwarding_client) => {
                handle_forwarding(&mut stream, forwarding_client).await?
            }
        }
    }
}

async fn create_websocket(
    stream: TlsStream<TcpStream>,
    websocket_path: &str,
) -> Result<Box<WebsocketWrapper>> {
    #[cfg(feature = "websockets")]
    {
        let (tx, rx) = oneshot::channel();
        let callback = |request: &handshake::server::Request, response| {
            let _ = tx.send(request.clone());
            Ok(response)
        };
        let stream = tokio_tungstenite::accept_hdr_async(stream, callback).await?;
        let path = rx.await?.uri().path().to_string();
        if path != *websocket_path {
            bail!(
                "Incorrect websocket path. Expected {}, got {}",
                websocket_path,
                path
            );
        }
        debug!("WebSocket path: {}", path);
        Ok(Box::new(WebsocketWrapper::new(stream)))
    }
    #[cfg(not(feature = "websockets"))]
    panic!("Not compiled with websockets")
}

async fn handle_handshake(
    socket_state: &mut SocketState,
    stream: &mut Box<dyn AsyncStream>,
    server_config: &ServerConfig,
) -> Result<()> {
    debug!("Begin handling handshake");
    let buffer = read_to_buffer(stream).await?;
    let request = TrojanHandshake::parse(&buffer).await.and_then(|req| {
        match hash_password(&server_config.password) == req.hashed_password {
            true => Ok(req),
            false => Err(anyhow!("Password was incorrect")),
        }
    });
    debug!("Handshake parsed");

    match request {
        Ok(request) => {
            debug!("Handshake succeeded");
            let payload = request.payload.clone();
            let mut forwarding_client =
                SimpleForwardingClient::new(&request.destination.try_into()?).await?;
            forwarding_client.write_buffer(&payload).await?;
            *socket_state = SocketState::Open(Box::new(forwarding_client));
        }
        Err(e) => {
            debug!("Handshake failed: {}. Using fallback.", e);
            let fallback_destination =
                socks5::protocol::Destination::Address(server_config.fallback_addr.parse()?);
            let mut forwarding_client =
                SimpleForwardingClient::new(&fallback_destination.try_into()?).await?;
            forwarding_client.write_buffer(&buffer).await?;
            *socket_state = SocketState::Open(Box::new(forwarding_client));
        }
    }

    Ok(())
}

async fn handle_forwarding(
    stream: &mut Box<dyn AsyncStream>,
    forwarding_client: &mut Box<dyn ForwardingClient + Send + Sync>,
) -> Result<()> {
    forwarding_client.forward(stream).await?;
    Ok(())
}

enum SocketState {
    WaitingForHandshake,
    Open(Box<dyn ForwardingClient + Send + Sync>),
}
