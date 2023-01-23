mod config;
mod error;
mod executor;
pub mod parser;
mod resolver;

use std::collections::HashMap;

use self::{
    config::{APIConfig, Config},
    error::ExecutorError,
    parser::{CmdArgs, Commands}, executor::{Engine, HttpResponse},
};

fn validate(cmd_args: &CmdArgs, apis: &HashMap<String, APIConfig>) -> Result<(), ExecutorError> {
    match &cmd_args.command {
        Commands::List => (),
        Commands::Run {
            api,
            input: _,
            context: _,
            endpoint,
            verbose: _,
        } => {
            if !apis.contains_key(api) {
                return Result::Err(ExecutorError::APINotFound(api.clone()));
            }
            if !apis
                .get(api)
                .map(|api_config| api_config.contains_endpoint(endpoint))
                .unwrap_or(false)
            {
                return Result::Err(ExecutorError::EndpointNotFound(endpoint.clone()));
            }
        }
    }
    return Result::Ok(());
}

fn printer(http_response: HttpResponse, verbose: &bool) -> Result<(), ExecutorError> {
    if *verbose {
        println!("{} {}", http_response.status, http_response.version);
        for header in http_response.headers.iter() {
            println!("{}: {}", header.key, header.value);
        }
    }
    if !http_response.body.is_empty() {
        let body_text = String::from_utf8(http_response.body).map_err(|e| ExecutorError::FailedToReadBody(e.to_string()))?;
        println!("{:?}", body_text);
    }
    Ok(())
}

pub fn execute(cmd_args: &CmdArgs, config: &Config) -> Result<(), ExecutorError> {
    let apis = config.read_apis();
    validate(cmd_args, &apis)?;
    match &cmd_args.command {
        Commands::List => print!("execute list"),
        Commands::Run {
            api,
            context,
            endpoint,
            input, 
            verbose 
        } => {
            let api_config = apis.get(api).unwrap();
            let mut engine = Engine::new();
            let result = engine.run(api_config, endpoint, context, input)?;
            printer(result, verbose)?
        }
    }
    return Result::Ok(());
}
