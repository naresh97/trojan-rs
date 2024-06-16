#![feature(slice_pattern)]

use clap::Parser;
use config::cli::{Application, Cli};
use simple_logger::SimpleLogger;

mod config;
mod forwarding;
mod socks5;
mod tls;
mod trojan;
mod utils;

#[tokio::main]
async fn main() {
    SimpleLogger::new()
        .with_level(log::LevelFilter::Info)
        .env()
        .init()
        .unwrap();
    let cli = Cli::parse();
    match cli.command {
        Application::Client { adapter: _adapter } => {
            socks5::client::main(cli.config_file).await.unwrap()
        }
        Application::Server => trojan::server::main(cli.config_file).await.unwrap(),
    };
}
