pub struct ClientConfig {
    pub listening_addr: String,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            listening_addr: "0.0.0.0:1235".to_string(),
        }
    }
}
