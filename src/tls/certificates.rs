use std::{fs::File, io::BufReader};

use anyhow::{anyhow, Result};
use rustls::pki_types::CertificateDer;

use crate::config::ServerConfig;

pub struct Certificates<'a> {
    pub cert: Vec<rustls::pki_types::CertificateDer<'a>>,
    pub private_key: rustls::pki_types::PrivateKeyDer<'a>,
}

impl<'a> Certificates<'a> {
    pub fn load(server_config: &ServerConfig) -> Result<Certificates<'a>> {
        let cert = load_certificates(&server_config.certificate_path)?;

        let private_key_file = File::open(&server_config.private_key_path)?;
        let mut private_key_file = BufReader::new(private_key_file);
        let private_key = rustls_pemfile::private_key(&mut private_key_file)?
            .ok_or(anyhow!("No private keys found"))?;

        Ok(Certificates { cert, private_key })
    }
}

pub fn load_certificates(path: &str) -> Result<Vec<CertificateDer<'static>>> {
    let cert_file = File::open(path)?;
    let mut cert_file = BufReader::new(cert_file);
    let cert = rustls_pemfile::certs(&mut cert_file).collect::<Result<Vec<_>, _>>()?;
    Ok(cert)
}
