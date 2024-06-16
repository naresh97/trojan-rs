use anyhow::{anyhow, Context, Result};

use aws_lc_rs::digest;

use crate::{
    socks5::{self, destination::Destination},
    utils::{advance_buffer, CRLF},
};
pub struct TrojanHandshake {
    pub password: String,
    #[allow(unused)]
    pub command: socks5::request::RequestCommand,
    pub destination: Destination,
    pub payload: Vec<u8>,
}

impl TrojanHandshake {
    pub async fn parse(buffer: &[u8]) -> Result<TrojanHandshake> {
        let password = buffer
            .get(0..56)
            .ok_or(anyhow!("Buffer too short, couldn't get password."))?;
        let password = std::str::from_utf8(password)
            .with_context(|| {
                format!(
                    "Reading password from handshake: {}",
                    String::from_utf8_lossy(password)
                )
            })?
            .to_string();

        let buffer = advance_buffer(56, buffer)?;

        let buffer = check_crlf_and_advance(buffer)?;

        let (command, buffer) = socks5::request::RequestCommand::parse(buffer)?;
        let (destination, buffer) = Destination::parse(buffer)?;

        let payload = check_crlf_and_advance(buffer)?.to_vec();
        Ok(TrojanHandshake {
            password,
            command,
            payload,
            destination,
        })
    }
    pub fn as_bytes(&self) -> Vec<u8> {
        let password = self.password.as_bytes();
        let command = [self.command.as_byte()];
        let destination = self.destination.as_bytes();
        let data = [
            password,
            CRLF.as_slice(),
            command.as_slice(),
            destination.as_slice(),
            CRLF.as_slice(),
            self.payload.as_slice(),
        ]
        .concat();
        data
    }
}

fn check_crlf_and_advance(buffer: &[u8]) -> Result<&[u8]> {
    buffer
        .get(0..2)
        .map(|crlf| {
            if crlf == [0x0d, 0x0a] {
                Ok(())
            } else {
                Err(anyhow!("Expected CRLF, found {:?}", crlf))
            }
        })
        .ok_or(anyhow!("Buffer too short."))??;
    advance_buffer(2, buffer)
}

pub fn hash_password(clear_text: &str) -> String {
    let hash = digest::digest(&digest::SHA224, clear_text.as_bytes());
    hex::encode(hash.as_ref())
}
