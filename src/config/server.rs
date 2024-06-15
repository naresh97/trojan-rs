#[derive(Clone)]
pub struct ServerConfig {
    pub listen_addr: String,
    pub password: String,
    pub fallback_addr: String,
    pub certificate_path: String,
    pub private_key_path: String,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            password: "12345".to_string(),
            fallback_addr: "192.168.0.107:2205".to_string(),
            listen_addr: "0.0.0.0:443".to_string(),
            certificate_path: "samples/cert.pem".to_string(),
            private_key_path: "samples/private.pem".to_string(),
        }
    }
}
