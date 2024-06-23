use anyhow::Context;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub listen_addr: String,
    pub password: String,
    pub fallback_addr: String,
    pub websocket_path: Option<String>,
    pub disable_port_80_redirect: Option<bool>,
    pub serve_files_from: Option<String>,
    pub certificate_path: String,
    pub private_key_path: String,
}

impl ServerConfig {
    pub fn load(path: &str) -> anyhow::Result<ServerConfig> {
        let config =
            std::fs::read_to_string(path).with_context(|| format!("Unable to load {:?}", path))?;
        let config = toml::from_str(&config)?;
        Ok(config)
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            password: "12345".to_string(),
            fallback_addr: "127.0.0.1:8080".to_string(),
            listen_addr: "0.0.0.0:443".to_string(),
            certificate_path: "samples/cert.pem".to_string(),
            private_key_path: "samples/private.pem".to_string(),
            websocket_path: None,
            disable_port_80_redirect: None,
            serve_files_from: None,
        }
    }
}
