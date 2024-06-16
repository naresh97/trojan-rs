use std::net::SocketAddr;

use anyhow::{anyhow, Result};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};
use tokio_rustls::TlsConnector;

use crate::config::ServerConfig;

pub struct ForwardingClient {
    stream: tokio_rustls::client::TlsStream<TcpStream>,
}

impl ForwardingClient {
    pub async fn new(
        connector: &TlsConnector,
        server_config: &ServerConfig,
        socket_address: SocketAddr,
    ) -> Result<ForwardingClient> {
        let stream = TcpStream::connect(socket_address).await?;
        let stream = connector
            .connect(server_config.domain.clone().try_into()?, stream)
            .await?;
        Ok(ForwardingClient { stream })
    }

    pub async fn forward(&mut self, buffer: &[u8]) -> Result<Vec<u8>> {
        self.stream.write_all(buffer).await?;
        let mut buffer = Vec::with_capacity(0x1000);
        match self.stream.read(&mut buffer).await {
            Ok(0) => Err(anyhow!("Socket closed.")),
            Ok(n) => Ok(buffer[..n].to_vec()),
            Err(e) => Err(e.into()),
        }
    }
}
