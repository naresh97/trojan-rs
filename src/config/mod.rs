mod client;
mod server;

pub use client::ClientConfig;
use serde::de::DeserializeOwned;
pub use server::ServerConfig;

pub trait LoadFromToml {
    fn load(path: &std::path::Path) -> anyhow::Result<Self>
    where
        Self: Sized + DeserializeOwned,
    {
        let config = std::fs::read_to_string(path)?;
        let config = toml::from_str(&config)?;
        Ok(config)
    }
}
