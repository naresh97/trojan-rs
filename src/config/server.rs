use std::{net::SocketAddr, str::FromStr};

use anyhow::Result;
use aws_lc_rs::digest;

use crate::socks5::destination::Destination;

pub struct ServerConfig {
    pub domain: String,
    pub listening_address: String,
    password: String,
    fallback_destination: String,
    pub certificate_path: String,
    pub private_key_path: String,
}

impl ServerConfig {
    fn get_password_hash(&self) -> String {
        let hash = digest::digest(&digest::SHA224, self.password.as_bytes());
        hex::encode(hash.as_ref())
    }
    pub fn is_password_correct(&self, password: &[u8]) -> bool {
        if let Ok(password) = std::str::from_utf8(password) {
            return password == self.get_password_hash();
        }
        false
    }
    pub fn get_fallback_addr(&self) -> Result<Destination> {
        Ok(Destination::Ip(SocketAddr::from_str(
            &self.fallback_destination,
        )?))
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            password: "12345".to_string(),
            domain: "example.com".to_string(),
            fallback_destination: "142.250.179.206:80".to_string(),
            listening_address: "0.0.0.0:1234".to_string(),
            certificate_path: "samples/cert.pem".to_string(),
            private_key_path: "samples/private.pem".to_string(),
        }
    }
}
