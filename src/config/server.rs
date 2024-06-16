use aws_lc_rs::digest;

pub struct ServerConfig {
    pub listen_addr: String,
    pub password: String,
    pub fallback_addr: String,
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
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            password: "12345".to_string(),
            fallback_addr: "192.168.0.107:2205".to_string(),
            listen_addr: "0.0.0.0:1234".to_string(),
            certificate_path: "samples/cert.pem".to_string(),
            private_key_path: "samples/private.pem".to_string(),
        }
    }
}
