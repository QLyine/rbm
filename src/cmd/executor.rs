use std::{collections::HashMap, fs::File, path::PathBuf, str::FromStr};

use reqwest::{
    self,
    blocking::{Body, RequestBuilder},
    header::{HeaderMap, HeaderName, HeaderValue},
};

use super::{
    config::{self, APIBody, APIConfig},
    resolver::{self, Resolver},
};

pub struct Engine {
    resolver: Box<dyn Resolver>,
    http_client: reqwest::blocking::Client,
}

pub fn new() -> Engine {
    let resolver = resolver::new();
    return Engine {
        resolver: resolver,
        http_client: reqwest::blocking::Client::new(),
    };
}

impl Engine {
    fn resolve_headers(&mut self, headers: &HashMap<String, String>) -> HeaderMap {
        let mut header_map = HeaderMap::new();
        for (k, v) in headers.iter() {
            let header_name = HeaderName::from_str(k).unwrap();
            let header_value = HeaderValue::from_str(self.resolver.resolve(v).as_str()).unwrap();
            header_map.insert(header_name, header_value);
        }
        header_map
    }

    fn add_body(
        &mut self,
        request: RequestBuilder,
        maybe_body: Option<&APIBody>,
    ) -> RequestBuilder {
        if maybe_body.is_none() {
            return request;
        }
        let body = maybe_body.unwrap();
        let content = self.resolver.resolve(&body.content);
        let body_req = match body.api_body_type {
            config::APIBodyType::FILE => Body::new(File::open(PathBuf::from(content)).unwrap()),
            config::APIBodyType::STRING => Body::from(content),
        };
        request.body(body_req)
    }

    pub fn run(&mut self, api_config: &APIConfig, endpoint: &str, context: &Option<String>, inputs: &Vec<(String, String)>) {
        for (k, v) in inputs.iter() {
            self.resolver.add_context(k.clone(), v)
        }
        let maybe_context = context
            .as_ref()
            .map(|c| api_config.get_api_context(c.as_str()))
            .flatten();
        if let Some(context_to_add) = maybe_context {
            for (k, v) in context_to_add.iter() {
                self.resolver.add_context(k.clone(), v.as_str())
            }
        }
        let api_endpoint = api_config.get_api_endpoint(endpoint).unwrap();
        let url = self.resolver.resolve(&api_endpoint.url);
        let resolved_headers = api_endpoint
            .headers
            .as_ref()
            .map(|h| self.resolve_headers(h))
            .unwrap_or_else(|| HeaderMap::new());
        let request = match &api_endpoint.method {
            config::APIMethod::GET => self.http_client.get(&url),
            config::APIMethod::POST => self.http_client.post(&url),
            config::APIMethod::DELETE => self.http_client.delete(&url),
            config::APIMethod::PATCH => self.http_client.patch(&url),
        };
        let request = request.headers(resolved_headers);
        let request = self.add_body(request, api_endpoint.body.as_ref());
        request.send().unwrap();
    }
}

#[cfg(test)]
#[path = "./executor_test.rs"]
mod executor_test;
