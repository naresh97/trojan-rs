use std::{fs::File, io::BufReader};

use anyhow::{anyhow, Result};

pub fn read_certificates(
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

pub struct Certificates<'a> {
    pub cert: Vec<rustls::pki_types::CertificateDer<'a>>,
    pub private_key: rustls::pki_types::PrivateKeyDer<'a>,
}
