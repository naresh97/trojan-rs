use tokio::io::{AsyncRead, AsyncWrite};

pub mod forwarding;
pub mod http_server;
pub mod tls;

pub trait AsyncStream: AsyncWrite + AsyncRead + Send + Sync + Unpin {}
impl<T> AsyncStream for T where T: AsyncWrite + AsyncRead + Send + Sync + Unpin {}
