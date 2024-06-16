use serde::{Deserialize, Serialize};

use super::LoadFromToml;

#[derive(Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub listen_addr: String,
    pub password: String,
    pub fallback_addr: String,
    pub certificate_path: String,
    pub private_key_path: String,
}

impl LoadFromToml for ServerConfig {}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            password: "12345".to_string(),
            fallback_addr: "127.0.0.1:8080".to_string(),
            listen_addr: "0.0.0.0:443".to_string(),
            certificate_path: "samples/cert.pem".to_string(),
            private_key_path: "samples/private.pem".to_string(),
        }
    }
}
