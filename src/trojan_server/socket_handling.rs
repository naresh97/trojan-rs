use crate::{
    config::ServerConfig, dns::DnsResolver, forwarding_client::ForwardingClient,
    socks5::destination::Destination, utils::read_to_buffer,
};
use anyhow::{anyhow, Result};
use tokio::{io::AsyncWriteExt, net::TcpStream};
use tokio_rustls::{server::TlsStream, TlsConnector};

use super::trojan::TrojanRequest;

pub async fn handle_socket(
    dns_resolver: &DnsResolver,
    server_config: &ServerConfig,
    tls_connector: &TlsConnector,
    mut stream: TlsStream<TcpStream>,
) -> Result<()> {
    let mut socket_state = SocketState::WaitingForHandshake;
    loop {
        match &mut socket_state {
            SocketState::WaitingForHandshake => {
                handle_handshake(
                    &mut socket_state,
                    &mut stream,
                    server_config,
                    tls_connector,
                    dns_resolver,
                )
                .await?
            }
            SocketState::Open(forwarding_client) => {
                handle_forwarding(&mut stream, forwarding_client).await?
            }
        }
    }
}

async fn handle_handshake(
    socket_state: &mut SocketState,
    stream: &mut TlsStream<TcpStream>,
    server_config: &ServerConfig,
    connector: &TlsConnector,
    dns_resolver: &DnsResolver,
) -> Result<()> {
    let buffer = read_to_buffer(stream).await?;

    let request = TrojanRequest::parse(&buffer).await.and_then(|req| {
        match server_config.is_password_correct(&req.password) {
            true => Ok(req),
            false => Err(anyhow!("Password was incorrect")),
        }
    });

    match request {
        Ok(request) => {
            let payload = request.payload.clone();
            let forwarding_client = request
                .into_forwarding_client(connector, dns_resolver)
                .await?;
            stream.write_all(&payload).await?;
            *socket_state = SocketState::Open(forwarding_client);
        }
        Err(_e) => {
            stream.write_all(&buffer).await?;
            let fallback_destination = Destination::Ip(server_config.fallback_addr.parse()?);
            let forwarding_client =
                ForwardingClient::new(connector, dns_resolver, fallback_destination, true).await?;
            *socket_state = SocketState::Open(forwarding_client);
        }
    }

    Ok(())
}

async fn handle_forwarding(
    stream: &mut TlsStream<TcpStream>,
    forwarding_client: &mut ForwardingClient,
) -> Result<()> {
    forwarding_client.forward(stream).await?;
    Ok(())
}

enum SocketState {
    WaitingForHandshake,
    Open(ForwardingClient),
}
