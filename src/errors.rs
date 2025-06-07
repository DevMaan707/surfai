use thiserror::Error;

#[derive(Error, Debug)]
pub enum BrowserError {
    #[error("Browser launch failed: {0}")]
    LaunchFailed(String),

    #[error("Navigation failed: {0}")]
    NavigationFailed(String),

    #[error("DOM extraction failed: {0}")]
    DomExtractionFailed(String),

    #[error("Element not found: {0}")]
    ElementNotFound(String),

    #[error("JavaScript execution failed: {0}")]
    JavaScriptFailed(String),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Chrome error: {0}")]
    ChromeError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Anyhow error: {0}")]
    AnyhowError(String),
}

pub type Result<T> = std::result::Result<T, BrowserError>;

// Convert anyhow::Error to BrowserError
impl From<anyhow::Error> for BrowserError {
    fn from(err: anyhow::Error) -> Self {
        BrowserError::AnyhowError(err.to_string())
    }
}

// Helper function to convert any error to BrowserError
impl BrowserError {
    pub fn from_any_error<E: std::fmt::Display>(err: E) -> Self {
        BrowserError::ChromeError(err.to_string())
    }
}
