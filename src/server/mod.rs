mod trojan;

use std::{io::ErrorKind, net::SocketAddr, sync::Arc};

use anyhow::Result;
use tokio::net::{TcpListener, TcpStream};
use tokio_rustls::{server, TlsAcceptor};

use crate::certificates::Certificates;

pub async fn run_server(
    server_address: &SocketAddr,
    certificates: Certificates<'static>,
) -> Result<()> {
    let acceptor = get_tls_acceptor(certificates)?;
    let listener = TcpListener::bind(server_address).await?;
    loop {
        let (tcp_stream, _) = listener.accept().await?;
        let acceptor = acceptor.clone();
        tokio::spawn(async move {
            match acceptor.accept(tcp_stream).await {
                Ok(tls_stream) => handle_socket(tls_stream).await,
                Err(e) => Err(e.into()),
            }
        });
    }
}

fn get_tls_acceptor(certificates: Certificates<'static>) -> Result<TlsAcceptor> {
    let config = rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(certificates.cert, certificates.private_key)?;
    let acceptor = TlsAcceptor::from(Arc::new(config));
    Ok(acceptor)
}

async fn handle_socket(socket: server::TlsStream<TcpStream>) -> Result<()> {
    let mut socket_state = SocketState::Initial;

    let (socket, _) = socket.into_inner();
    loop {
        socket.readable().await?;
        let mut buf = Vec::with_capacity(0x1000);
        match socket.try_read_buf(&mut buf) {
            Ok(0) => break,
            Ok(n) => handle_incoming_data(&mut socket_state, &buf[..n]).await,
            Err(ref e) if e.kind() == ErrorKind::WouldBlock => continue,
            Err(e) => return Err(e.into()),
        }?;
    }
    Ok(())
}

enum SocketState {
    Initial,
    Approved,
    Rejected,
}

async fn handle_incoming_data(socket_state: &mut SocketState, buffer: &[u8]) -> Result<()> {
    match socket_state {
        SocketState::Initial => todo!(),
        SocketState::Approved => todo!(),
        SocketState::Rejected => todo!(),
    }
    Ok(())
}
