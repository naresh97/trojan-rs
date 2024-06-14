#![feature(ip_bits)]
#![feature(slice_pattern)]
#![allow(dead_code)]

mod config;
mod dns;
mod socks5;
mod tls;
mod trojan_server;
mod utils;

#[tokio::main]
async fn main() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .init();
    trojan_server::server_main().await.unwrap();
}
