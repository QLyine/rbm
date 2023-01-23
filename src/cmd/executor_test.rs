#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use httpmock::Method::{GET, POST};
    use httpmock::MockServer;

    use crate::cmd::config::{APIBody, APIBodyType, APIConfig, APIContext, APIEndpoint, APIMethod};
    use crate::cmd::executor::{self, Engine};

    #[test]
    fn post_expansion_test() -> std::io::Result<()> {
        let data = r#"{
                    "name": "John Doe",
                    "age": 43,
                    "nick_names": [
                      "nick",
                      "boo"
                    ]
                  }"#;
        let server = MockServer::start();
        // Create a mock on the server.
        let hello_mock = server.mock(|when, then| {
            when.method(POST)
                .path("/foo/bar")
                .body(data)
                .header("Authorization", "foo");
            then.status(200)
                .body(data.as_bytes());
        });
        let mut engine = Engine::new();
        let api_config = APIConfig::new(create_context(&server.port(), data), create_endpoints());
        let context_to_use = Some("local".to_string());
        let inputs = vec![];
        let result = engine.run(&api_config, "test_endpoint", &context_to_use, &inputs);
        assert_eq!(result.is_ok(), true);
        let response = result.unwrap();
        assert_eq!(response.status, 200);
        let body = response.body;
        assert_eq!(body.is_empty(), false);
        hello_mock.assert();
        Ok(())
    }

    fn create_endpoints() -> HashMap<String, APIEndpoint> {
        let api_endpoint = APIEndpoint {
            method: APIMethod::POST,
            url: "http://{{url}}/foo/bar".to_string(),
            headers: Some(HashMap::from([(
                "Authorization".to_string(),
                "{{auth}}".to_string(),
            )])),
            body: Some(APIBody::new("{{body}}", APIBodyType::STRING)),
        };
        HashMap::from([("test_endpoint".to_string(), api_endpoint)])
    }

    fn create_context(port: &u16, req_body: &str) -> Option<HashMap<String, APIContext>> {
        let address = format!("localhost:{}", port);
        let api_context_local: APIContext = HashMap::from([
            ("body".to_string(), req_body.to_string()),
            ("url".to_string(), address.clone()),
            ("auth".to_string(), "foo".to_string()),
        ]);
        let api_context_remote: APIContext = HashMap::from([
            ("url".to_string(), address),
            ("auth".to_string(), "bar".to_string()),
        ]);
        let context = HashMap::from([
            ("local".to_string(), api_context_local),
            ("remote".to_string(), api_context_remote),
        ]);
        return Some(context);
    }
}
