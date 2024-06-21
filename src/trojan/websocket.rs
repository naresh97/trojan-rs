use std::{
    io::{self, ErrorKind},
    task::Poll,
};

use futures::{SinkExt, StreamExt};
use tokio::{
    io::{AsyncRead, AsyncWrite},
    net::TcpStream,
};
use tokio_native_tls::TlsStream;
use tokio_tungstenite::{tungstenite::Message, WebSocketStream};

pub struct WebsocketWrapper {
    websocket: WebSocketStream<TlsStream<TcpStream>>,
}

impl WebsocketWrapper {
    pub fn new(websocket: WebSocketStream<TlsStream<TcpStream>>) -> WebsocketWrapper {
        WebsocketWrapper { websocket }
    }
}

impl Unpin for WebsocketWrapper {}

impl AsyncRead for WebsocketWrapper {
    fn poll_read(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        let websocket = &mut self.get_mut().websocket;
        let result = match websocket.poll_next_unpin(cx) {
            Poll::Ready(Some(message)) => match message {
                Ok(message) => {
                    buf.put_slice(message.into_data().as_slice());
                    Poll::Ready(Ok(()))
                }
                Err(e) => Poll::Ready(Err(into_io_error(e))),
            },
            Poll::Ready(None) => Poll::Ready(Err(io::Error::new(
                ErrorKind::UnexpectedEof,
                "Stream ended",
            ))),
            Poll::Pending => Poll::Pending,
        };
        result
    }
}

impl AsyncWrite for WebsocketWrapper {
    fn poll_write(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<Result<usize, std::io::Error>> {
        let websocket = &mut self.get_mut().websocket;
        let message = Message::binary(buf);

        match websocket.poll_ready_unpin(cx) {
            Poll::Ready(Ok(())) => match websocket.start_send_unpin(message) {
                Ok(()) => match websocket.poll_flush_unpin(cx) {
                    Poll::Ready(e) => Poll::Ready(e.map(|_| buf.len()).map_err(into_io_error)),
                    Poll::Pending => Poll::Pending,
                },
                Err(e) => Poll::Ready(Err(into_io_error(e))),
            },
            Poll::Ready(Err(e)) => Poll::Ready(Err(into_io_error(e))),
            Poll::Pending => Poll::Pending,
        }
    }

    fn poll_flush(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), std::io::Error>> {
        let websocket = &mut self.get_mut().websocket;
        match websocket.poll_flush_unpin(cx) {
            Poll::Ready(res) => Poll::Ready(res.map_err(into_io_error)),
            Poll::Pending => Poll::Pending,
        }
    }

    fn poll_shutdown(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), std::io::Error>> {
        let websocket = &mut self.get_mut().websocket;
        match websocket.poll_close_unpin(cx) {
            Poll::Ready(res) => Poll::Ready(res.map_err(into_io_error)),
            Poll::Pending => Poll::Pending,
        }
    }
}

fn into_io_error(e: impl std::error::Error + Send + Sync + 'static) -> std::io::Error {
    io::Error::new(ErrorKind::InvalidData, e)
}
