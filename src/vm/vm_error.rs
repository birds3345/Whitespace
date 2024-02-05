use std::fmt;

#[derive(Debug)]
pub struct VMError {
    message: String,
}

impl VMError {
    pub fn new(message: &str) -> Self {
        Self {
            message: message.to_string(),
        }
    }
}

impl fmt::Display for VMError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for VMError {
    fn description(&self) -> &str {
        &self.message
    }
}