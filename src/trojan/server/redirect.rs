use anyhow::{bail, Context};
use tokio::io::AsyncWriteExt;

use crate::utils::read_to_buffer;

pub async fn serve_redirect(
    mut tcp_stream: tokio::net::TcpStream,
) -> Result<Result<(), anyhow::Error>, anyhow::Error> {
    let buffer = read_to_buffer(&mut tcp_stream).await?;
    let request = std::str::from_utf8(&buffer)?;
    let lines = request.split('\n').collect::<Vec<&str>>();
    if lines.len() < 2 {
        bail!("Not enough lines");
    }
    let first_line = lines
        .first()
        .context("Couldn't get first line")?
        .split_ascii_whitespace()
        .collect::<Vec<&str>>();
    if *first_line.first().context("Couldn't get GET")? != "GET" {
        bail!("Not GET request");
    }
    let uri = *first_line.get(1).context("Couldn't get URI")?;
    let second_line = lines
        .get(1)
        .context("Couldn't get second line")?
        .split_ascii_whitespace()
        .collect::<Vec<&str>>();
    if *second_line.first().context("Couldn't get Host:")? != "Host:" {
        bail!("Request malformed");
    }
    let host = *second_line.get(1).context("Couldn't get host")?;
    let response = format!(
        r#"HTTP/1.1 301 Moved Permanently
Location: https://{}{}
"#,
        host, uri
    );
    tcp_stream.write_all(response.as_bytes()).await?;
    Ok(Ok::<(), anyhow::Error>(()))
}
