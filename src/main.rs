#![feature(slice_pattern)]

use clap::Parser;
use simple_logger::SimpleLogger;

mod cli;
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
    let cli = cli::Cli::parse();
    match cli.command {
        cli::Application::Client { adapter: _adapter } => {
            socks5::client::main(cli.config_file).await.unwrap()
        }
        cli::Application::Server => trojan::server::main(cli.config_file).await.unwrap(),
    };
}
