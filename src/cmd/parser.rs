use std::path::PathBuf;

use clap::{Parser, Subcommand};

use super::config::{Config, self};

#[derive(Parser, Debug)]
pub struct CmdArgs {
    #[arg(short, long, value_name = "FILE")]
    config: Option<PathBuf>,
    #[command(subcommand)]
    command: Option<Commands>
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    List,
    Run {
        #[arg(short = 'i', long)]
        input: String,
        #[arg(short = 'c', long)]
        context: String,
        #[arg(short = 'e', long)]
        endpoint: String
    }
}

pub fn parse_cli_args() -> CmdArgs {
    return CmdArgs::parse()
}

impl CmdArgs {
    pub fn read_config(&self) -> Config {
        return config::read_config_or_create_default(&self.config)
    }
}