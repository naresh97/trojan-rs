use anyhow::{anyhow, Result};

use tokio_rustls::TlsConnector;

use crate::{
    dns::DnsResolver,
    forwarding_client::ForwardingClient,
    socks5::{self, destination::Destination},
    utils::advance_buffer,
};

pub struct TrojanRequest {
    pub password: Vec<u8>,
    pub command: socks5::request::RequestCommand,
    pub destination: Destination,
    pub payload: Vec<u8>,
}

impl TrojanRequest {
    pub async fn parse(buffer: &[u8]) -> Result<TrojanRequest> {
        let password = buffer
            .get(0..56)
            .ok_or(anyhow!("Buffer too short, couldn't get password."))?
            .to_vec();
        let buffer = advance_buffer(56, buffer)?;

        let buffer = check_crlf_and_advance(buffer)?;

        let (command, buffer) = socks5::request::RequestCommand::parse(buffer)?;
        let (destination, buffer) = Destination::parse(buffer)?;

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
        dns_resolver: &DnsResolver,
    ) -> Result<ForwardingClient> {
        ForwardingClient::new(connector, dns_resolver, self.destination, true).await
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
