#![feature(slice_pattern)]

use adapters::{socks5, ClientAdapter};
use config::cli::{Application, Cli, ClientAdapterType};
use simple_logger::SimpleLogger;

mod adapters;
mod config;
mod forwarding;
mod tls;
mod trojan;
mod utils;

#[tokio::main]
async fn main() {
    let cli = Cli::parse().unwrap();
    SimpleLogger::new()
        .with_level(cli.log_level)
        .env()
        .init()
        .unwrap();

    match cli.command {
        Application::Client => match cli.client_adapter_type {
            ClientAdapterType::Socks5 => {
                socks5::Socks5Adapter::main(cli.config_file).await.unwrap()
            }
        },
        Application::Server => trojan::server::main(cli.config_file).await.unwrap(),
    };
}
