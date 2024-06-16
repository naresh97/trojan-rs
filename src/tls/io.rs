use crate::config::ServerConfig;

use anyhow::{Context, Result};
use native_tls::{Certificate, Identity};

pub fn get_tls_acceptor(server_config: &ServerConfig) -> Result<tokio_native_tls::TlsAcceptor> {
    let pem = std::fs::read_to_string(&server_config.certificate_path)
        .context("Server Certificate File")?;
    let key = std::fs::read_to_string(&server_config.private_key_path)
        .context("Server Private Key File")?;
    let identity = Identity::from_pkcs8(pem.as_bytes(), key.as_bytes())?;
    let acceptor = native_tls::TlsAcceptor::builder(identity).build()?;
    let acceptor = tokio_native_tls::TlsAcceptor::from(acceptor);
    Ok(acceptor)
}

pub fn get_tls_connector() -> Result<tokio_native_tls::TlsConnector> {
    let mut builder = native_tls::TlsConnector::builder();

    if true {
        //cfg!(debug_assertions) {
        builder
            .danger_accept_invalid_certs(true)
            .danger_accept_invalid_hostnames(true);
        let ca = std::fs::read_to_string("ca.pem")
            .context("Can't read ca.pem")
            .and_then(|ca| {
                Certificate::from_pem(ca.as_bytes()).context("Can't parse ca.pem as certificate")
            });
        if let Ok(ca) = ca {
            builder.add_root_certificate(ca);
        }
    }
    let connector = builder.build()?;
    let connector = tokio_native_tls::TlsConnector::from(connector);
    Ok(connector)
}
