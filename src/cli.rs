use clap::{command, Parser, Subcommand, ValueEnum};

#[derive(Parser)]
#[command(version,about,long_about=None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Application,

    #[arg(short = 'c', long = "config")]
    pub config_file: Option<String>,
}

#[derive(Subcommand)]
pub enum Application {
    Client {
        #[arg(value_enum, default_value_t=ClientAdapters::Socks5)]
        adapter: ClientAdapters,
    },
    Server,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum ClientAdapters {
    Socks5,
}
