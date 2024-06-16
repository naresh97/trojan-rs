use std::sync::Arc;

use anyhow::Result;
use tokio_rustls::{TlsAcceptor, TlsConnector};

use super::certificates::Certificates;

#[derive(Clone)]
pub struct TlsAdapters {
    pub acceptor: TlsAcceptor,
    pub connector: TlsConnector,
}

impl TlsAdapters {
    pub fn new(certificates: Certificates<'static>) -> Result<TlsAdapters> {
        Ok(TlsAdapters {
            acceptor: get_tls_acceptor(certificates)?,
            connector: get_tls_connector(),
        })
    }
}

pub fn get_tls_acceptor(certificates: Certificates<'static>) -> Result<TlsAcceptor> {
    let config = rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(certificates.cert, certificates.private_key)?;
    let acceptor = TlsAcceptor::from(Arc::new(config));
    Ok(acceptor)
}

pub fn get_tls_connector() -> TlsConnector {
    let root_store =
        rustls::RootCertStore::from_iter(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
    let config = rustls::ClientConfig::builder()
        .with_root_certificates(root_store)
        .with_no_client_auth();
    TlsConnector::from(Arc::new(config))
}
