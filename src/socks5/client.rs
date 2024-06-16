use std::path::Path;

use anyhow::Result;
use log::{debug, info};
use tokio::{
    io::AsyncWriteExt,
    net::{TcpListener, TcpStream},
};
use tokio_native_tls::TlsConnector;

use crate::{
    config::{ClientConfig, LoadFromToml},
    socks5::identify::parse_identify_block,
    tls::io::get_tls_connector,
    trojan::client::TrojanClient,
    utils::read_to_buffer,
};

use super::{
    identify::IDENTIFY_RESPONSE,
    request::{create_response, Request},
};

pub async fn main(config_file: Option<String>) -> Result<()> {
    info!("Starting SOCKS5 Trojan Client");
    let config_file = config_file.unwrap_or("client.toml".to_string());
    let config = ClientConfig::load(Path::new(&config_file))?;
    let listener = TcpListener::bind(&config.listening_addr).await?;

    let connector = get_tls_connector()?;
    info!("Loaded configs and ready to proxy requests");

    loop {
        let (stream, _) = listener.accept().await?;
        let connector = connector.clone();
        let config = config.clone();
        tokio::spawn(async move {
            if let Err(err) = handle_socket(stream, &connector, &config).await {
                debug!("{}", err);
            }
            debug!("Ending socket.");
        });
    }
}

enum ClientState {
    WaitForIdentify,
    WaitForRequest,
    Open(TrojanClient),
}

async fn handle_socket(
    mut stream: TcpStream,
    connector: &TlsConnector,
    client_config: &ClientConfig,
) -> Result<()> {
    let mut client_state = ClientState::WaitForIdentify;

    loop {
        match &mut client_state {
            ClientState::Open(forwarding) => {
                handle_forwarding(forwarding, &mut stream, client_config).await?
            }
            _ => {
                handle_socket_setup(&mut client_state, &mut stream, connector, client_config)
                    .await?
            }
        }
    }
}

async fn handle_forwarding(
    forwarding: &mut TrojanClient,
    client_stream: &mut TcpStream,
    client_config: &ClientConfig,
) -> Result<()> {
    let payload = read_to_buffer(client_stream).await?;
    debug!("Initial payload: {}", String::from_utf8_lossy(&payload));
    forwarding.send_handshake(&payload, client_config).await?;
    forwarding.forward(client_stream).await?;
    Ok(())
}

async fn handle_socket_setup(
    client_state: &mut ClientState,
    stream: &mut TcpStream,
    connector: &TlsConnector,
    client_config: &ClientConfig,
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

            let client = TrojanClient::new(request.destination, client_config, connector).await?;
            let response = create_response(&client.local_addr)?;
            *client_state = ClientState::Open(client);
            stream.write_all(&response).await?;
            debug!("SOCKS5: Request created");
        }
    }
    Ok(())
}
