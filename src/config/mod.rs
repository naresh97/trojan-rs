pub mod cli;
mod client;
mod server;

pub use client::ClientConfig;
pub use server::ServerConfig;

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn default_config() {
        let client = ClientConfig::default();
        let client = toml::to_string_pretty(&client).unwrap();
        std::fs::write("samples/client.toml", client).unwrap();

        let server = ServerConfig::default();
        let server = toml::to_string_pretty(&server).unwrap();
        std::fs::write("samples/server.toml", server).unwrap();
    }
}
