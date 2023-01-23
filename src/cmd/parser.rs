use std::path::PathBuf;

use clap::{Parser, Subcommand, builder::TypedValueParser, error::{ContextKind, ContextValue}};
use std::error::Error;

use super::config::{self, Config};

#[derive(Parser, Debug)]
pub struct CmdArgs {
    #[arg(short, long, value_name = "FILE")]
    config: Option<PathBuf>,
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    List,
    Run {
        #[arg(short = 'a', long)]
        api: String,
        #[arg(short = 'c', long)]
        context: Option<String>,
        #[arg(short = 'e', long)]
        endpoint: String,
        #[arg(short = 'i', long, value_parser = PairStringParser::new())]
        input: Vec<(String, String)>,
        #[arg(short = 'v', long, default_value = "false")]
        verbose: bool,
    },
}
#[derive(Copy, Clone, Debug)]
struct PairStringParser {}

impl PairStringParser {
    pub fn new() -> Self {
        Self {  }
    }
}

impl TypedValueParser for PairStringParser {
    type Value = (String, String);

    fn parse_ref(
        &self,
        cmd: &clap::Command,
        arg: Option<&clap::Arg>,
        value: &std::ffi::OsStr,
    ) -> Result<Self::Value, clap::Error> {
        return value.to_str().map(|str| parse_key_val(str)).unwrap_or_else(|| {
                let mut err = clap::Error::new(clap::error::ErrorKind::InvalidValue);
                err.insert(ContextKind::InvalidArg, ContextValue::String("argument cannot be empty".to_owned()));
                Result::Err(err)
            }
        )
    }

}

/// Parse a single key-value pair
fn parse_key_val<T, U>(s: &str) -> Result<(T, U), clap::Error>
where
    T: std::str::FromStr,
    T::Err: Error + Send + Sync + 'static,
    U: std::str::FromStr,
    U::Err: Error + Send + Sync + 'static,
{
    let pos = s
        .find('=')
        .ok_or_else(|| format!("invalid KEY=value: no `=` found in `{}`", s)).expect("msg");
    Ok((s[..pos].parse().expect("msg"), s[pos + 1..].parse().expect("msg")))
}

pub fn parse_cli_args() -> CmdArgs {
    return CmdArgs::parse();
}

impl CmdArgs {
    pub fn read_config(&self) -> Config {
        return config::read_config_or_create_default(&self.config);
    }
}
