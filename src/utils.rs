use anyhow::{anyhow, Result};

pub fn advance_buffer(length: usize, buffer: &[u8]) -> Result<&[u8]> {
    buffer
        .get(length..)
        .ok_or(anyhow!("Couldn't get remaning buffer"))
}
