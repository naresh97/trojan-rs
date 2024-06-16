mod client;
mod server;

use anyhow::Context;
pub use client::ClientConfig;
use serde::de::DeserializeOwned;
pub use server::ServerConfig;

pub trait LoadFromToml {
    fn load(path: &std::path::Path) -> anyhow::Result<Self>
    where
        Self: Sized + DeserializeOwned,
    {
        let config =
            std::fs::read_to_string(path).with_context(|| format!("Unable to load {:?}", path))?;
        let config = toml::from_str(&config)?;
        Ok(config)
    }
}

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
