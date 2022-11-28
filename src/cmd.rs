
pub mod parser;
mod config;
mod resolver;
mod error;
mod executor;

use core::fmt;
use std::collections::HashMap;

use self::{parser::{CmdArgs, Commands}, config::{Config, APIConfig}, error::ExecutorError};

impl fmt::Display for ExecutorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExecutorError::ValidationError(msg) => write!(f, "{}", msg.as_str())
        }
    }
}

fn validate(cmdArgs: &CmdArgs, apis: &HashMap<String, APIConfig>) -> Result<(), ExecutorError> {
    match &cmdArgs.command {
        Commands::List => (),
        Commands::Run { api, input: _, context: _, endpoint} => {
            if !apis.contains_key(api) {
                return Result::Err(ExecutorError::ValidationError(format!("api {} not found", api)))
            }
            if !apis.get(api).map(|api_config| api_config.contains_endpoint(endpoint) ).unwrap_or(false) {
                return Result::Err(ExecutorError::ValidationError(format!("endpoint {} not found", endpoint)))
            }
        }
    }
    return Result::Ok(())
}

pub fn execute(cmdArgs: &CmdArgs, config: &Config) -> Result<(), ExecutorError> {
    let apis = config.read_apis();
    let validation_result = validate(cmdArgs, &apis);
    if validation_result.is_err() {
        return validation_result;
    }
    match &cmdArgs.command {
        Commands::List => print!("execute list"),
        Commands::Run { api, context, endpoint, input } => {
            let api_config = apis.get(api).unwrap();
            let mut engine = executor::new();
            engine.run(api_config, endpoint, context);
        }
    }
    return Result::Ok(())
}