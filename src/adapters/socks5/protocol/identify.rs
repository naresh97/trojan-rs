use anyhow::{anyhow, bail, Result};

use crate::utils::advance_buffer;

pub fn parse_identify_block(buffer: &[u8]) -> Result<&[u8]> {
    let version = *buffer.first().ok_or(anyhow!("Buffer empty"))?;
    if version != 5 {
        bail!("Only SOCKS5 supported");
    }
    let buffer = advance_buffer(1, buffer)?;

    let length = *buffer
        .first()
        .ok_or(anyhow!("Could not get method length"))? as usize;
    let buffer = advance_buffer(1, buffer)?;

    let _methods = buffer
        .get(..length)
        .ok_or(anyhow!("Could not read specified number of methods"))?;

    advance_buffer(length, buffer)
}

pub const IDENTIFY_RESPONSE: [u8; 2] = [5, 0];
