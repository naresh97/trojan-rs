use std::{net::SocketAddr, str::FromStr};

use anyhow::bail;
use log::{debug, info};
use tokio::{join, net::TcpListener, task::JoinHandle};

use crate::{
    config::ServerConfig,
    networking::tls::get_tls_acceptor,
    trojan::server::{redirect::serve_redirect, socket_handling::handle_socket},
};

pub async fn main(config_file: Option<String>) -> anyhow::Result<()> {
    info!("Starting Trojan Server");
    let config_file = config_file.unwrap_or("server.toml".to_string());
    let server_config = ServerConfig::load(&config_file)?;
    info!("Loaded configs, ready to listen.");

    let tls_acceptor = get_tls_acceptor(&server_config)?;

    let main_server: JoinHandle<Result<(), anyhow::Error>>;
    {
        let server_config = server_config.clone();
        let tls_acceptor = tls_acceptor.clone();
        main_server = tokio::spawn(async move {
            let listener = TcpListener::bind(&server_config.listen_addr).await?;
            debug!("Main server listening");
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
            #[allow(unreachable_code)]
            Ok::<(), anyhow::Error>(())
        });
    }

    let redirect_server = tokio::spawn(async move {
        if let Some(disable) = &server_config.disable_port_80_redirect {
            if *disable {
                bail!("Port 80 redirect disabled.");
            }
        }

        let mut redirect_address = SocketAddr::from_str(&server_config.listen_addr)?;
        redirect_address.set_port(80);
        let listener = TcpListener::bind(redirect_address).await?;
        debug!("Redirect server listening.");
        loop {
            let (tcp_stream, _) = listener.accept().await?;
            tokio::spawn(async move { serve_redirect(tcp_stream).await? });
        }
        #[allow(unreachable_code)]
        Ok::<(), anyhow::Error>(())
    });

    let _ = join!(main_server, redirect_server);
    Ok(())
}
