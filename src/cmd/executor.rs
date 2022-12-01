use std::{collections::HashMap, hash::Hash, str::FromStr};

use reqwest::{self, header::{HeaderMap, HeaderValue, HeaderName}};

use super::{resolver::{self, Resolver}, parser::CmdArgs, config::{self, APIConfig}};

pub struct Engine {
    resolver: Box<dyn Resolver>,
    http_client: reqwest::blocking::Client,
}

pub fn new() -> Engine {
    let resolver = resolver::new();
    return Engine{ resolver: resolver, http_client: reqwest::blocking::Client::new() };
}

impl Engine {
    fn resolve_headers(&mut self, headers: &HashMap<String, String>) -> HeaderMap {
      let mut header_map = HeaderMap::new();
      for (k,v) in headers.iter() {
        let header_name = HeaderName::from_str(k).unwrap();
        let header_value = HeaderValue::from_str(v).unwrap();
        header_map.insert(header_name, header_value);
      }
      header_map
    }

    pub fn run(&mut self, api_config: &APIConfig, endpoint: &str, context: &Option<String>) {
        println!("{:?}", api_config);
        let maybe_context = context
        .as_ref().map(|c| api_config.get_api_context(c.as_str()))
        .flatten();
        if let Some(context_to_add) = maybe_context {
            for (k, v) in context_to_add.iter() {
                self.resolver.add_context(k.clone(), v.as_str())
            }
        }
        let api_endpoint = api_config.get_api_endpoint(endpoint).unwrap();
        let url = self.resolver.resolve(&api_endpoint.url);
        let resolved_headers = api_endpoint.headers.as_ref().map(|h| self.resolve_headers(h)).unwrap_or_else(|| HeaderMap::new());
        let mut request = match &api_endpoint.method {
            config::APIMethod::GET => self.http_client.get(&url).headers(resolved_headers),
            config::APIMethod::POST => self.http_client.post(&url).headers(resolved_headers),
            config::APIMethod::DELETE => todo!(),
            config::APIMethod::PATCH => todo!(),
        };
        // request.headers(resolved_headers);
        request.send().unwrap();
        print!("{:?}", url);
        //println!("{:?}", resolved_headers);

    }
}