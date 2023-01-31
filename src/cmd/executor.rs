use std::{collections::HashMap, fs::File, path::PathBuf, str::FromStr};

use reqwest::{
    self,
    blocking::{Body, RequestBuilder, Response},
    header::{HeaderMap, HeaderName, HeaderValue},
};

use super::{
    config::{self, APIBody, APIConfig, AuthEndpoint},
    error::ExecutorError,
    resolver::{self, Resolver},
};

const CONTEXT_KEY: &str = "context";

pub struct Header {
    pub key: String,
    pub value: String,
}

pub struct HttpResponse {
    pub status: u16,
    pub version: String,
    pub headers: Vec<Header>,
    pub body: Vec<u8>,
}

pub struct Engine {
    resolver: Box<dyn Resolver>,
    http_client: reqwest::blocking::Client,
}

impl Engine {
    pub fn new() -> Self {
        let resolver = resolver::new();
        Engine {
            resolver,
            http_client: reqwest::blocking::Client::new(),
        }
    }

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

    fn map_response(response: Response) -> Result<HttpResponse, ExecutorError> {
        let status = response.status().as_u16();
        let mut headers: Vec<Header> = Vec::with_capacity(response.headers().capacity());
        for (hk, hv) in response.headers().iter() {
            let key = hk.to_string();
            let value = String::from_utf8(hv.as_bytes().into())
                .map_err(|e| ExecutorError::FailedToParseHeader(key.clone(), e.to_string()))?;
            headers.push(Header {
                key,
                value,
            })
        }
        let version = format!("{:?}", response.version());
        let body = response
            .bytes()
            .map(|b| b.to_vec())
            .map_err(|err| ExecutorError::FailedToReadBody(err.to_string()))?;
        Result::Ok(HttpResponse {
            status,
            version,
            headers,
            body,
        })
    }

    fn add_auth(&mut self, request: RequestBuilder, auth_endpoint: &AuthEndpoint) -> RequestBuilder {
        match auth_endpoint {
            AuthEndpoint::Basic { username, password } => 
                request.basic_auth(
                    self.resolver.resolve(username), 
                    Some(self.resolver.resolve(password))
                ),
            _ => request
        }
    }

    pub fn run(
        &mut self,
        api_config: &APIConfig,
        endpoint: &str,
        maybe_context: &Option<String>,
        inputs: &Vec<(String, String)>,
    ) -> Result<HttpResponse, ExecutorError> {
        if let Some(context) = maybe_context {
            self.resolver.add_context(CONTEXT_KEY.to_string(), context)
        }
        for (k, v) in inputs.iter() {
            self.resolver.add_context(k.clone(), v)
        }
        let maybe_context = maybe_context
            .as_ref()
            .and_then(|c| api_config.get_api_context(c.as_str()));
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
            .unwrap_or_else(HeaderMap::new);
        let request = match &api_endpoint.method {
            config::APIMethod::GET => self.http_client.get(&url),
            config::APIMethod::POST => self.http_client.post(&url),
            config::APIMethod::DELETE => self.http_client.delete(&url),
            config::APIMethod::PATCH => self.http_client.patch(&url),
        };
        let request = if let Some(auth_endpoint) = &api_endpoint.auth  {
            self.add_auth(request, auth_endpoint)
        } else {
            request
        }
        .headers(resolved_headers);

        let request = self.add_body(request, api_endpoint.body.as_ref());
        request
            .send()
            .map_err(|e| ExecutorError::HTTPRequestError(e.to_string()))
            .and_then(Self::map_response)
    }
}

#[cfg(test)]
#[path = "./executor_test.rs"]
mod executor_test;
