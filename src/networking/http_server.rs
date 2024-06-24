use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use async_trait::async_trait;
use log::debug;
use tokio::io::AsyncWriteExt;

use crate::config::ServerConfig;

use super::{forwarding::ForwardingClient, AsyncStream};

pub struct HttpServer {
    file_path: PathBuf,
    buffer: Vec<u8>,
}

impl HttpServer {
    pub async fn new(server_config: &ServerConfig) -> Result<HttpServer> {
        let file_path = get_server_file_path(server_config)?;
        Ok(HttpServer {
            file_path,
            buffer: Vec::new(),
        })
    }
}

fn get_server_file_path(server_config: &ServerConfig) -> Result<PathBuf> {
    let file_path = server_config
        .serve_files_from
        .as_ref()
        .context("File path not configured.")?;
    let file_path = PathBuf::from(file_path);
    let file_path = file_path.strip_prefix("/")?.to_owned();
    Ok(file_path)
}

#[async_trait]
impl ForwardingClient for HttpServer {
    async fn forward(&mut self, client_stream: &mut Box<dyn AsyncStream>) -> Result<()> {
        debug!("Sending response: {}", std::str::from_utf8(&self.buffer)?);
        client_stream.write_all(&self.buffer).await?;
        bail!("Done");
    }
    async fn write_buffer(&mut self, buffer: &[u8]) -> Result<()> {
        let uri = get_uri(buffer)?;
        let requested_file = self.file_path.join(uri);
        debug!("Requested file: {}", requested_file.to_string_lossy());
        let mut response = generate_response(&requested_file)?;
        self.buffer.append(&mut response);
        Ok(())
    }
}

fn get_uri(buffer: &[u8]) -> Result<PathBuf> {
    let request = std::str::from_utf8(buffer)?;
    let GetRequest { uri, host: _ } = parse_get_request(request)?;
    let uri = if uri == "/" {
        "/index.html".to_string()
    } else {
        uri
    };
    let uri = PathBuf::from(uri);
    Ok(uri)
}

fn generate_response(requested_file: &Path) -> Result<Vec<u8>> {
    if requested_file.is_file() {
        let data = std::fs::read_to_string(requested_file)?;
        let mime = mime_guess::from_path(requested_file);
        let mime = mime.first_or_text_plain();
        let mime = mime.essence_str();
        let response = format!(
            r#"HTTP/1.1 200 OK
Content-Type: {}

{}
"#,
            mime, data
        );
        Ok(response.as_bytes().to_vec())
    } else {
        let response_404 = r#"HTTP/1.1 404 Not Found
Content-Type: text/html

<h1>404 Not Found</h1>
"#;
        Ok(response_404.as_bytes().to_vec())
    }
}

pub struct GetRequest {
    pub uri: String,
    pub host: String,
}

pub fn parse_get_request(request: &str) -> Result<GetRequest> {
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
    let uri = first_line.get(1).context("Couldn't get URI")?;
    let uri = (*uri).to_string();
    let second_line = lines
        .get(1)
        .context("Couldn't get second line")?
        .split_ascii_whitespace()
        .collect::<Vec<&str>>();
    if *second_line.first().context("Couldn't get Host:")? != "Host:" {
        bail!("Request malformed");
    }
    let host = second_line.get(1).context("Couldn't get host")?;
    let host = (*host).to_string();

    Ok(GetRequest { uri, host })
}
