use ipstack::{stream::IpStackStream, IpStackConfig};
use tokio_native_tls::TlsConnector;

use crate::{
    config::{ClientConfig, LoadFromToml},
    networking::tls::get_tls_connector,
    trojan::client::TrojanClient,
    utils::read_to_buffer,
};

use super::{socks5::protocol::Destination, ClientAdapter};

pub struct TunAdapter;
impl ClientAdapter for TunAdapter {
    async fn main(config_file: Option<String>) -> anyhow::Result<()> {
        let config_path = config_file.unwrap_or("client.toml".to_string());
        let client_config = ClientConfig::load(&config_path)?;
        let mut tun_config = tun2::Configuration::default();
        tun_config.up();
        let dev = tun2::create_as_async(&tun_config)?;
        let ipstack_config = IpStackConfig::default();
        let mut ipstack = ipstack::IpStack::new(ipstack_config, dev);
        let connector = get_tls_connector()?;

        loop {
            let stream = ipstack.accept().await?;
            let client_config = client_config.clone();
            let connector = connector.clone();
            tokio::spawn(async move {
                let _result = handle_socket(stream, &client_config, &connector).await;
            });
        }
    }
}

async fn handle_socket(
    mut stream: IpStackStream,
    client_config: &ClientConfig,
    connector: &TlsConnector,
) -> anyhow::Result<()> {
    if let IpStackStream::Tcp(stream) = &mut stream {
        let destination = Destination::Address(stream.peer_addr());
        let mut trojan_client = TrojanClient::new(destination, client_config, connector).await?;
        let payload = read_to_buffer(stream).await?;
        trojan_client
            .send_handshake(&payload, client_config)
            .await?;
        trojan_client.forward(stream).await?;
    }
    Ok(())
}
