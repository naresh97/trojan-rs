use anyhow::Result;
use log::{debug, error};
use tokio::{
    io::AsyncWriteExt,
    net::{TcpListener, TcpStream},
};
use tokio_rustls::TlsConnector;

use crate::{
    config::ClientConfig,
    dns::DnsResolver,
    forwarding_client::ForwardingClient,
    socks5::identify::parse_identify_block,
    tls::io::get_tls_connector,
    utils::read_to_buffer,
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
        match &mut client_state {
            ClientState::Open(forwarding) => handle_forwarding(forwarding, &mut stream).await?,
            _ => {
                handle_socket_setup(&mut client_state, &mut stream, connector, dns_resolver).await?
            }
        }
    }
}

async fn handle_forwarding(
    forwarding: &mut ForwardingClient,
    client_stream: &mut TcpStream,
) -> Result<()> {
    forwarding.forward(client_stream).await?;
    Ok(())
}

async fn handle_socket_setup(
    client_state: &mut ClientState,
    stream: &mut TcpStream,
    connector: &TlsConnector,
    dns_resolver: &DnsResolver,
) -> Result<()> {
    let buffer = read_to_buffer(stream).await?;
    match client_state {
        ClientState::Open(_) => unreachable!(),
        ClientState::WaitForIdentify => {
            let _ = parse_identify_block(&buffer)?;
            *client_state = ClientState::WaitForRequest;
            stream.write_all(IDENTIFY_RESPONSE.as_slice()).await?;
            debug!("SOCKS5: ID done");
        }
        ClientState::WaitForRequest => {
            let (request, _buffer) = Request::parse(&buffer)?;
            let client =
                ForwardingClient::new(connector, dns_resolver, request.destination, false).await?;
            let response = create_response(&client.local_addr)?;
            *client_state = ClientState::Open(client);
            stream.write_all(&response).await?;
            debug!("SOCKS5: Request created");
        }
    }
    Ok(())
}
