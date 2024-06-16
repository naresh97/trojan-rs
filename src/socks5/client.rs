use std::io::ErrorKind;

use anyhow::Result;
use log::{debug, error};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};
use tokio_rustls::TlsConnector;

use crate::{
    config::ClientConfig, dns::DnsResolver, forwarding_client::ForwardingClient,
    socks5::identify::parse_identify_block, tls::io::get_tls_connector, utils::BUFFER_SIZE,
};

use super::{
    identify::IDENTIFY_RESPONSE,
    request::{create_response, Request},
};

pub async fn client_main() -> Result<()> {
    debug!("Starting SOCKS5 Trojan Client");
    let config = ClientConfig::default();
    let listener = TcpListener::bind(config.listening_addr).await?;
    let dns_resolver = DnsResolver::new().await;
    let connector = get_tls_connector();

    loop {
        let (stream, _) = listener.accept().await?;
        let dns_resolver = dns_resolver.clone();
        let connector = connector.clone();
        tokio::spawn(async move {
            if let Err(err) = handle_socket(stream, &dns_resolver, &connector).await {
                error!("{}", err);
            }
            debug!("Ending socket.");
        });
    }
}

enum ClientState {
    WaitForIdentify,
    WaitForRequest,
    Open(ForwardingClient),
}

async fn handle_socket(
    mut stream: TcpStream,
    dns_resolver: &DnsResolver,
    connector: &TlsConnector,
) -> Result<()> {
    let mut client_state = ClientState::WaitForIdentify;
    loop {
        stream.readable().await?;
        let mut buffer = Vec::with_capacity(BUFFER_SIZE);
        match stream.read_buf(&mut buffer).await {
            Ok(0) => break,
            Ok(n) => {
                handle_incoming(
                    &buffer[..n],
                    &mut stream,
                    &mut client_state,
                    dns_resolver,
                    connector,
                )
                .await?
            }
            Err(ref e) if e.kind() == ErrorKind::WouldBlock => continue,
            Err(e) => return Err(e.into()),
        }
    }
    Ok(())
}

async fn handle_incoming(
    buffer: &[u8],
    stream: &mut TcpStream,
    client_state: &mut ClientState,
    dns_resolver: &DnsResolver,
    connector: &TlsConnector,
) -> Result<()> {
    match client_state {
        ClientState::WaitForIdentify => {
            let _ = parse_identify_block(buffer)?;
            *client_state = ClientState::WaitForRequest;
            stream.write_all(IDENTIFY_RESPONSE.as_slice()).await?;
            debug!("SOCKS5: ID done");
        }
        ClientState::WaitForRequest => {
            let (request, _buffer) = Request::parse(buffer)?;
            let client =
                ForwardingClient::new(connector, dns_resolver, request.destination, false).await?;
            let response = create_response(&client.local_addr)?;
            *client_state = ClientState::Open(client);
            debug!("SOCKS5: Request created");
            stream.write_all(&response).await?;
        }
        ClientState::Open(client) => {
            client.forward_into_writer(buffer, stream).await?;
        }
    }
    Ok(())
}
