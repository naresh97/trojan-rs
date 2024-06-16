use std::net::SocketAddr;

use anyhow::{anyhow, Result};

use tokio_rustls::TlsConnector;

use crate::{config::ServerConfig, dns::DnsResolver, socks5, utils::advance_buffer};

use super::forwarding_client::ForwardingClient;

pub struct TrojanRequest {
    pub password: Vec<u8>,
    pub command: socks5::request::RequestCommand,
    pub destination: SocketAddr,
    pub payload: Vec<u8>,
}

impl TrojanRequest {
    pub async fn parse(dns_resolver: &DnsResolver, buffer: &[u8]) -> Result<TrojanRequest> {
        let password = buffer
            .get(0..56)
            .ok_or(anyhow!("Buffer too short, couldn't get password."))?
            .to_vec();
        let buffer = advance_buffer(56, buffer)?;

        let buffer = check_crlf_and_advance(buffer)?;

        let (command, buffer) = socks5::request::RequestCommand::parse(buffer)?;
        let (destination, buffer) =
            socks5::destination::parse_request_destination(dns_resolver, buffer).await?;

        let payload = check_crlf_and_advance(buffer)?.to_vec();
        Ok(TrojanRequest {
            password,
            command,
            payload,
            destination,
        })
    }
    pub async fn into_forwarding_client(
        self,
        connector: &TlsConnector,
        server_config: &ServerConfig,
    ) -> Result<ForwardingClient> {
        ForwardingClient::new(connector, server_config, self.destination).await
    }
}

fn check_crlf_and_advance(buffer: &[u8]) -> Result<&[u8]> {
    buffer
        .get(0..2)
        .map(|crlf| {
            if crlf == [0x0d, 0x0a] {
                Err(anyhow!("Expected CRLF"))
            } else {
                Ok(())
            }
        })
        .ok_or(anyhow!("Buffer too short."))??;
    advance_buffer(2, buffer)
}
