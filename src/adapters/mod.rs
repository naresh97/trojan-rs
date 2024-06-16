pub mod socks5;

use std::future::Future;

pub trait ClientAdapter {
    fn main(&self) -> impl Future<Output = anyhow::Result<()>>;
}
