mod config;
mod error;
mod executor;
pub mod parser;
mod resolver;

use std::collections::HashMap;

use self::{
    config::{APIConfig, Config},
    error::ExecutorError,
    parser::{CmdArgs, Commands},
};

fn validate(cmd_args: &CmdArgs, apis: &HashMap<String, APIConfig>) -> Result<(), ExecutorError> {
    match &cmd_args.command {
        Commands::List => (),
        Commands::Run {
            api,
            input: _,
            context: _,
            endpoint,
        } => {
            if !apis.contains_key(api) {
                return Result::Err(ExecutorError::ValidationError(format!(
                    "api {} not found",
                    api
                )));
            }
            if !apis
                .get(api)
                .map(|api_config| api_config.contains_endpoint(endpoint))
                .unwrap_or(false)
            {
                return Result::Err(ExecutorError::ValidationError(format!(
                    "endpoint {} not found",
                    endpoint
                )));
            }
        }
    }
    return Result::Ok(());
}

pub fn execute(cmd_args: &CmdArgs, config: &Config) -> Result<(), ExecutorError> {
    let apis = config.read_apis();
    let validation_result = validate(cmd_args, &apis);
    if validation_result.is_err() {
        return validation_result;
    }
    match &cmd_args.command {
        Commands::List => print!("execute list"),
        Commands::Run {
            api,
            context,
            endpoint,
            input,
        } => {
            let api_config = apis.get(api).unwrap();
            let mut engine = executor::new();
            engine.run(api_config, endpoint, context);
        }
    }
    return Result::Ok(());
}
