#![feature(slice_pattern)]

use std::env;

mod config;
mod dns;
mod forwarding;
mod socks5;
mod tls;
mod trojan;
mod utils;

#[tokio::main]
async fn main() {
    env_logger::builder().init();
    let args: Vec<String> = env::args().collect();
    match args.get(1).map(|x| x.as_str()) {
        Some("client") => socks5::client::client_main().await.unwrap(),
        Some(_) | None => trojan::server_main().await.unwrap(),
    };
}
