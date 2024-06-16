use serde::{Deserialize, Serialize};

use super::LoadFromToml;

#[derive(Clone, Serialize, Deserialize)]
pub struct ClientConfig {
    pub listening_addr: String,
    pub password: String,
    pub server_addr: String,
}

impl LoadFromToml for ClientConfig {}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            listening_addr: "0.0.0.0:1080".to_string(),
            password: "12345".to_string(),
            server_addr: "example.com:443".to_string(),
        }
    }
}
