use std::fmt;

#[derive(Debug)]
pub enum BrowserError {
    Launch(String),
    Navigation(String),
    ElementNotFound(String),
    Script(String),
    IO(std::io::Error),
    Serialization(serde_json::Error),
}

impl fmt::Display for BrowserError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BrowserError::Launch(msg) => write!(f, "Browser launch error: {}", msg),
            BrowserError::Navigation(msg) => write!(f, "Navigation error: {}", msg),
            BrowserError::ElementNotFound(selector) => write!(f, "Element not found: {}", selector),
            BrowserError::Script(msg) => write!(f, "Script error: {}", msg),
            BrowserError::IO(e) => write!(f, "IO error: {}", e),
            BrowserError::Serialization(e) => write!(f, "Serialization error: {}", e),
        }
    }
}

impl std::error::Error for BrowserError {}

impl From<std::io::Error> for BrowserError {
    fn from(error: std::io::Error) -> Self {
        BrowserError::IO(error)
    }
}

impl From<serde_json::Error> for BrowserError {
    fn from(error: serde_json::Error) -> Self {
        BrowserError::Serialization(error)
    }
}

pub type BrowserResult<T> = std::result::Result<T, BrowserError>;
