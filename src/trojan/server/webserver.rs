use anyhow::{bail, Context, Result};

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
