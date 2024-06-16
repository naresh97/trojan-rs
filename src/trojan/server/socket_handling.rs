use crate::{
    config::ServerConfig,
    dns::DnsResolver,
    forwarding::SimpleForwardingClient,
    socks5::destination::Destination,
    trojan::protocol::{hash_password, TrojanHandshake},
    utils::read_to_buffer,
};
use anyhow::{anyhow, Result};
use log::debug;
use tokio::net::TcpStream;
use tokio_rustls::server::TlsStream;

pub async fn handle_socket(
    dns_resolver: &DnsResolver,
    server_config: &ServerConfig,
    mut stream: TlsStream<TcpStream>,
) -> Result<()> {
    let mut socket_state = SocketState::WaitingForHandshake;
    loop {
        match &mut socket_state {
            SocketState::WaitingForHandshake => {
                handle_handshake(&mut socket_state, &mut stream, server_config, dns_resolver)
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
    dns_resolver: &DnsResolver,
) -> Result<()> {
    let buffer = read_to_buffer(stream).await?;

    let request = TrojanHandshake::parse(&buffer).await.and_then(|req| {
        match hash_password(&server_config.password) == req.password {
            true => Ok(req),
            false => Err(anyhow!("Password was incorrect")),
        }
    });

    match request {
        Ok(request) => {
            debug!("Handshake succeeded");
            let payload = request.payload.clone();
            let mut forwarding_client =
                SimpleForwardingClient::new(dns_resolver, request.destination).await?;
            forwarding_client.write_buffer(&payload).await?;
            *socket_state = SocketState::Open(forwarding_client);
        }
        Err(e) => {
            debug!("Handshake failed: {}. Using fallback.", e);
            let fallback_destination = Destination::Address(server_config.fallback_addr.parse()?);
            let mut forwarding_client =
                SimpleForwardingClient::new(dns_resolver, fallback_destination).await?;
            forwarding_client.write_buffer(&buffer).await?;
            *socket_state = SocketState::Open(forwarding_client);
        }
    }

    Ok(())
}

async fn handle_forwarding(
    stream: &mut TlsStream<TcpStream>,
    forwarding_client: &mut SimpleForwardingClient,
) -> Result<()> {
    forwarding_client.forward(stream).await?;
    Ok(())
}

enum SocketState {
    WaitingForHandshake,
    Open(SimpleForwardingClient),
}
