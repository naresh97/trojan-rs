use serde::{Deserialize, Serialize};

use super::LoadFromToml;

#[derive(Clone, Serialize, Deserialize)]
pub struct ClientConfig {
    pub socks5: Option<Socks5>,
    pub tun: Option<Tun>,
    pub password: String,
    pub server_addr: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Socks5 {
    pub listening_addr: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Tun {
    pub tun_device_ip: String,
}

impl LoadFromToml for ClientConfig {}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            password: "12345".to_string(),
            server_addr: "example.com:443".to_string(),
            socks5: Some(Socks5 {
                listening_addr: "0.0.0.0:1080".to_string(),
            }),
            tun: Some(Tun {
                tun_device_ip: "10.0.0.1".to_string(),
            }),
        }
    }
}
