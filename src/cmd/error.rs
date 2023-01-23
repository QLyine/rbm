use thiserror::Error;

#[derive(Error, Debug)]
pub enum ExecutorError {
    #[error("API not found: {0}")]
    APINotFound(String),
    #[error("Endpoint not found: {0}")]
    EndpointNotFound(String),
    #[error("HTTP Request error: {0}")]
    HTTPRequestError(String),
    #[error("Failed to parse header: {0}. Err: {1}")]
    FailedToParseHeader(String, String),
    #[error("Failed to read body: {0}")]
    FailedToReadBody(String),
    #[error("Failed to write body into stdout: {0}")]
    FailedToPrintBody(String),
}
