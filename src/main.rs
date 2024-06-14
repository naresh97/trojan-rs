#![feature(ip_bits)]
#![feature(slice_pattern)]
#![allow(dead_code)]

use server::server_main;

mod config;
mod dns;
mod server;
mod socks5;
mod tls;
mod utils;

#[tokio::main]
async fn main() {
    server_main().await.unwrap();
}
