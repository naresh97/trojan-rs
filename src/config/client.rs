use anyhow::Context;
use serde::{Deserialize, Serialize};

use crate::trojan::hash_password;

#[derive(Clone, Serialize, Deserialize)]
pub struct ClientConfig {
    pub password: String,
    pub hashed_password: Option<String>,
    pub server_addr: String,
    pub websocket_path: Option<String>,
    pub socks5: Option<Socks5>,
    pub tun: Option<Tun>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Socks5 {
    pub listening_addr: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Tun {
    pub tun_device_ip: String,
}

impl ClientConfig {
    pub fn load(path: &str) -> anyhow::Result<ClientConfig> {
        let config =
            std::fs::read_to_string(path).with_context(|| format!("Unable to load {:?}", path))?;
        let mut config: ClientConfig = toml::from_str(&config)?;
        config.hashed_password = Some(hash_password(&config.password));
        Ok(config)
    }
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            password: "12345".to_string(),
            hashed_password: None,
            server_addr: "example.com:443".to_string(),
            socks5: Some(Socks5 {
                listening_addr: "0.0.0.0:1080".to_string(),
            }),
            tun: Some(Tun {
                tun_device_ip: "10.0.0.1".to_string(),
            }),
            websocket_path: None,
        }
    }
}
