use std::{path::PathBuf, sync::Arc};

use log::debug;
use tokio::net::TcpListener;

use crate::{
    config::ServerConfig,
    dns::DnsResolver,
    tls::{
        certificates::read_certificates,
        io::{get_tls_acceptor, get_tls_connector},
    },
};

use super::socket_handling::handle_socket;

pub async fn server_main() -> anyhow::Result<()> {
    let server_config: Arc<ServerConfig> = Arc::new(ServerConfig::default());
    let certificates = read_certificates(
        PathBuf::from(&server_config.certificate_path).as_path(),
        PathBuf::from(&server_config.private_key_path).as_path(),
    )?;
    let acceptor = get_tls_acceptor(certificates)?;
    let connector = get_tls_connector();
    let listener = TcpListener::bind(&server_config.listen_addr).await?;
    let dns_resolver = Arc::new(DnsResolver::new().await);
    debug!("Loaded configs, ready to listen.");
    loop {
        let (tcp_stream, _) = listener.accept().await?;
        debug!("Incoming socket");
        let acceptor = acceptor.clone();
        let connector = connector.clone();
        let dns_resolver = dns_resolver.clone();
        let server_config = server_config.clone();
        tokio::spawn(async move {
            let result = match acceptor.accept(tcp_stream).await {
                Ok(tls_stream) => {
                    handle_socket(&dns_resolver, &server_config, &connector, tls_stream).await
                }
                Err(e) => Err(e.into()),
            };
            if let Err(e) = result {
                debug!("Socket handling error: {}", e);
            }
        });
    }
}
