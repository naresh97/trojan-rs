#![feature(ip_bits)]
#![feature(slice_pattern)]
#![allow(dead_code)]

use std::env;

mod config;
mod dns;
mod forwarding_client;
mod socks5;
mod tls;
mod trojan;
mod utils;

#[tokio::main]
async fn main() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .init();
    let args: Vec<String> = env::args().collect();
    match args.get(1).map(|x| x.as_str()) {
        Some("client") => socks5::client::client_main().await.unwrap(),
        Some(_) | None => trojan::server_main().await.unwrap(),
    };
}
