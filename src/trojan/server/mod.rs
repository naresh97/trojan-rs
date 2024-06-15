mod socket_handling;

use std::{path::Path, sync::Arc};

use log::debug;
use socket_handling::handle_socket;
use tokio::net::TcpListener;

use crate::{
    config::{LoadFromToml, ServerConfig},
    dns::DnsResolver,
    tls::{certificates::Certificates, io::get_tls_acceptor},
};

pub async fn server_main() -> anyhow::Result<()> {
    let server_config = ServerConfig::load(Path::new("samples/server.toml"))?;
    debug!("Loaded configs, ready to listen.");

    let tls_acceptor = get_tls_acceptor(Certificates::load(&server_config)?)?;
    let dns_resolver = Arc::new(DnsResolver::new().await);

    let listener = TcpListener::bind(&server_config.listen_addr).await?;
    loop {
        let (tcp_stream, _) = listener.accept().await?;
        debug!("Incoming socket");

        let server_config = server_config.clone();
        let tls_acceptor = tls_acceptor.clone();
        let dns_resolver = dns_resolver.clone();

        tokio::spawn(async move {
            let result = match tls_acceptor.accept(tcp_stream).await {
                Ok(tls_stream) => handle_socket(&dns_resolver, &server_config, tls_stream).await,
                Err(e) => Err(e.into()),
            };
            if let Err(e) = result {
                debug!("Socket handling error: {}", e);
            }
        });
    }
}
