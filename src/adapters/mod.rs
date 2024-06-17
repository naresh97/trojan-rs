pub mod socks5;
pub mod tun;

use std::future::Future;

pub trait ClientAdapter {
    fn main(config_file: Option<String>) -> impl Future<Output = anyhow::Result<()>>;
}
