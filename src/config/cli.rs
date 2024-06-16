use std::str::FromStr;

use anyhow::anyhow;
use log::LevelFilter;

pub struct Cli {
    pub command: Application,
    pub config_file: Option<String>,
    pub log_level: LevelFilter,
    // Client Options
    #[allow(unused)]
    pub client_adapter: ClientAdapters,
}

pub enum Application {
    Client,
    Server,
}
impl FromStr for Application {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        ["CLIENT", "SERVER"]
            .iter()
            .position(|&name| name.eq_ignore_ascii_case(s))
            .map(|p| match p {
                0 => Application::Client,
                1 => Application::Server,
                _ => unreachable!(),
            })
            .ok_or(anyhow!("Couldn't parse"))
    }
}

pub enum ClientAdapters {
    Socks5,
}

impl FromStr for ClientAdapters {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        ["SOCKS5"]
            .iter()
            .position(|&name| name.eq_ignore_ascii_case(s))
            .map(|p| match p {
                0 => ClientAdapters::Socks5,
                _ => unreachable!(),
            })
            .ok_or(anyhow!("Couldn't parse"))
    }
}

impl Cli {
    pub fn parse() -> anyhow::Result<Cli> {
        let mut pargs = pico_args::Arguments::from_env();
        let args = Cli {
            config_file: pargs.opt_value_from_str("--config")?,
            log_level: pargs
                .opt_value_from_str("--log")?
                .unwrap_or(LevelFilter::Info),
            command: pargs.free_from_str()?,
            client_adapter: pargs
                .opt_value_from_str("--adapter")?
                .unwrap_or(ClientAdapters::Socks5),
        };
        Ok(args)
    }
}
