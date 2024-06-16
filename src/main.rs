#![feature(slice_pattern)]

use adapters::socks5;
use config::cli::{Application, Cli};
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
        Application::Client => socks5::client::main(cli.config_file).await.unwrap(),
        Application::Server => trojan::server::main(cli.config_file).await.unwrap(),
    };
}
