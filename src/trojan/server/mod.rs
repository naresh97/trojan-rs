mod socket_handling;

use std::path::Path;

use log::{debug, info};
use socket_handling::handle_socket;
use tokio::net::TcpListener;

use crate::{
    config::{LoadFromToml, ServerConfig},
    tls::io::get_tls_acceptor,
};

pub async fn server_main() -> anyhow::Result<()> {
    info!("Starting Trojan Server");
    let server_config = ServerConfig::load(Path::new("samples/server.toml"))?;
    info!("Loaded configs, ready to listen.");

    let tls_acceptor = get_tls_acceptor(&server_config)?;

    let listener = TcpListener::bind(&server_config.listen_addr).await?;
    loop {
        let (tcp_stream, _) = listener.accept().await?;
        debug!("Incoming socket");

        let server_config = server_config.clone();
        let tls_acceptor = tls_acceptor.clone();

        tokio::spawn(async move {
            let result = match tls_acceptor.accept(tcp_stream).await {
                Ok(tls_stream) => handle_socket(&server_config, tls_stream).await,
                Err(e) => Err(e.into()),
            };
            if let Err(e) = result {
                debug!("Socket handling error: {}", e);
            }
        });
    }
}
