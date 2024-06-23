use tokio::io::AsyncWriteExt;

use crate::{
    networking::http_server::{parse_get_request, GetRequest},
    utils::read_to_buffer,
};

pub async fn serve_redirect(
    mut tcp_stream: tokio::net::TcpStream,
) -> Result<Result<(), anyhow::Error>, anyhow::Error> {
    let buffer = read_to_buffer(&mut tcp_stream).await?;
    let request = std::str::from_utf8(&buffer)?;
    let GetRequest { host, uri } = parse_get_request(request)?;
    let response = format!(
        r#"HTTP/1.1 301 Moved Permanently
Location: https://{}{}
"#,
        host, uri
    );
    tcp_stream.write_all(response.as_bytes()).await?;
    Ok(Ok::<(), anyhow::Error>(()))
}
