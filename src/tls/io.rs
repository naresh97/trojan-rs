use std::sync::Arc;

use anyhow::Result;
use rustls::pki_types::CertificateDer;
use tokio_rustls::{TlsAcceptor, TlsConnector};

use super::certificates::Certificates;

pub fn get_tls_acceptor(certificates: Certificates<'static>) -> Result<TlsAcceptor> {
    let config = rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(
            certificates.cert.clone(),
            certificates.private_key.clone_key(),
        )?;
    let acceptor = TlsAcceptor::from(Arc::new(config));
    Ok(acceptor)
}

pub fn get_tls_connector(self_signed: Option<CertificateDer<'static>>) -> Result<TlsConnector> {
    let mut root_store =
        rustls::RootCertStore::from_iter(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());

    if cfg!(debug_assertions) {
        // Allow the use of self-signed certificates in the debug builds to test
        if let Some(self_signed) = self_signed {
            root_store.add(self_signed)?;
        }
    }

    let config = rustls::ClientConfig::builder()
        .with_root_certificates(root_store)
        .with_no_client_auth();
    Ok(TlsConnector::from(Arc::new(config)))
}
