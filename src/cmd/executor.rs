use reqwest;

use super::{resolver::{self, Resolver}, parser::CmdArgs, config::{self, APIConfig}};

pub struct Engine {
    resolver: Box<dyn Resolver>,
    http_client: reqwest::Client,
}

pub fn new() -> Engine {
    let resolver = resolver::new();
    return Engine{ resolver: resolver, http_client: reqwest::Client::new() };
}

impl Engine {
    pub fn run(&mut self, api_config: &APIConfig, endpoint: &str, context: &Option<String>) {
        let maybe_context = context
        .as_ref().map(|c| api_config.get_api_context(c.as_str()))
        .flatten();
        if let Some(context_to_add) = maybe_context {
            for (k, v) in context_to_add.iter() {
                self.resolver.add_context(k.clone(), v.clone())
            }
        }
        let api_endpoint = api_config.get_api_endpoint(endpoint).unwrap();
        let url = self.resolver.resolve(&api_endpoint.url);
        print!("{:?}", url)
    }
}