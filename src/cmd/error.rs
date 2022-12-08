use std::fmt;

pub enum ExecutorError {
    ValidationError(String),
}

impl fmt::Display for ExecutorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExecutorError::ValidationError(msg) => write!(f, "{}", msg.as_str()),
        }
    }
}
