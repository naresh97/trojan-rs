pub struct ClientConfig {
    pub listening_addr: String,
    pub password: String,
    pub server_domain: String,
    pub server_port: u16,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            listening_addr: "0.0.0.0:1235".to_string(),
            password: "12345".to_string(),
            server_domain: "example.com".to_string(),
            server_port: 443,
        }
    }
}
