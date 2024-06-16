#![feature(slice_pattern)]

use std::env;

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

    let args: Vec<String> = env::args().collect();
    match args.get(1).map(|x| x.as_str()) {
        Some("client") => socks5::client::client_main().await.unwrap(),
        Some(_) | None => trojan::server_main().await.unwrap(),
    };
}
