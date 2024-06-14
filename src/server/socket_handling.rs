use crate::{config::ServerConfig, dns::DnsResolver};
use anyhow::{anyhow, Result};
use log::debug;
use std::io::ErrorKind;
use tokio::{io::AsyncReadExt, net::TcpStream};
use tokio_rustls::{server, TlsConnector};

use super::{forwarding_client::ForwardingClient, trojan::TrojanRequest};

pub async fn handle_socket(
    dns_resolver: &DnsResolver,
    server_config: &ServerConfig,
    tls_connector: &TlsConnector,
    mut socket: server::TlsStream<TcpStream>,
) -> Result<()> {
    let mut socket_state = SocketState::Initial;
    loop {
        let mut buf = Vec::with_capacity(0x1000);
        match socket.read_buf(&mut buf).await {
            Ok(0) => break,
            Ok(n) => {
                socket_state
                    .forward(dns_resolver, server_config, tls_connector, &buf[..n])
                    .await
            }
            Err(ref e) if e.kind() == ErrorKind::WouldBlock => continue,
            Err(e) => return Err(e.into()),
        }?;
    }
    Ok(())
}
enum SocketState {
    Initial,
    Approved(ForwardingClient),
    Rejected(ForwardingClient),
}

impl SocketState {
    async fn forward(
        &mut self,
        dns_resolver: &DnsResolver,
        server_config: &ServerConfig,
        tls_connector: &TlsConnector,
        buffer: &[u8],
    ) -> Result<Vec<u8>> {
        match self {
            SocketState::Initial => {
                self.handle_initial(dns_resolver, server_config, tls_connector, buffer)
                    .await
            }
            SocketState::Approved(client) => client.forward(buffer).await,
            SocketState::Rejected(client) => client.forward(buffer).await,
        }
    }

    async fn handle_initial(
        &mut self,
        dns_resolver: &DnsResolver,
        server_config: &ServerConfig,
        tls_connector: &TlsConnector,
        buffer: &[u8],
    ) -> Result<Vec<u8>> {
        debug!("Initializing new socket");

        let request = TrojanRequest::parse(buffer).await;
        let request =
            request.and_then(
                |req| match server_config.is_password_correct(&req.password) {
                    true => Ok(req),
                    false => Err(anyhow!("Password was incorrec")),
                },
            );
        match request {
            Ok(request) => {
                debug!("Trojan handshake recognized, creating client.");
                let payload = request.payload.clone();
                let mut client = request
                    .into_forwarding_client(tls_connector, dns_resolver)
                    .await?;
                let result = client.forward(&payload).await;
                *self = SocketState::Approved(client);
                result
            }
            Err(e) => {
                debug!("Trojan handshake failed: {}", e);
                let mut client = ForwardingClient::new(
                    tls_connector,
                    dns_resolver,
                    server_config.get_fallback_addr()?,
                )
                .await?;
                let result = client.forward(buffer).await;
                *self = SocketState::Rejected(client);
                result
            }
        }
    }
}
