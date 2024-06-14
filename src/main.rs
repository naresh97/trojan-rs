use std::{
    fs::File,
    io::{self, BufReader},
    sync::Arc,
};

use anyhow::{anyhow, Result};
use tokio::{
    io::AsyncReadExt,
    net::{TcpListener, TcpStream},
};
use tokio_rustls::{client::TlsStream, TlsAcceptor};

mod trojan_request;

enum SocketState {
    Initial,
    Approved,
    Rejected,
}

fn read_certificates(
    cert_path: &std::path::Path,
    private_key_path: &std::path::Path,
) -> Result<Certificates<'static>> {
    let cert_file = File::open(cert_path)?;
    let mut cert_file = BufReader::new(cert_file);
    let cert = rustls_pemfile::certs(&mut cert_file).collect::<Result<Vec<_>, _>>()?;

    let private_key_file = File::open(private_key_path)?;
    let mut private_key_file = BufReader::new(private_key_file);
    let private_key = rustls_pemfile::private_key(&mut private_key_file)?
        .ok_or(anyhow!("No private keys found"))?;
    Ok(Certificates { cert, private_key })
}

struct Certificates<'a> {
    cert: Vec<rustls::pki_types::CertificateDer<'a>>,
    private_key: rustls::pki_types::PrivateKeyDer<'a>,
}

async fn handle_socket(socket: TlsStream<TcpStream>) -> Result<()> {
    let mut socket_state = SocketState::Initial;
    let (socket, _) = socket.into_inner();
    loop {
        socket.readable().await?;
        let mut buf = Vec::with_capacity(0x1000);
        match socket.try_read_buf(&mut buf) {
            Ok(0) => break,
            Ok(_n) => {
                buf.shrink_to_fit();
                parse_incoming(&mut socket_state, buf).await;
            }
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => continue,
            Err(e) => return Err(e.into()),
        }
    }
    Ok(())
}

async fn parse_incoming(socket_state: &mut SocketState, buffer: Vec<u8>) {
    match socket_state {
        SocketState::Initial => parse_initial_handshake(),
        SocketState::Approved => todo!(),
        SocketState::Rejected => todo!(),
    }
}

async fn run() -> Result<()> {
    let config = rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(
            certificates.cert.clone(),
            certificates.private_key.clone_key(),
        )?;
    let acceptor = TlsAcceptor::from(Arc::new(config));

    let listener = TcpListener::bind("0.0.0.0:123").await?;
    loop {
        let (socket, _) = listener.accept().await?;
        let acceptor = acceptor.clone();
        tokio::spawn(async move {
            let socket = acceptor.accept(socket).await?;
            let _ = handle_socket(socket).await;
        });
    }
    Ok(())
}

#[tokio::main]
async fn main() {}
