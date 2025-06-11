use thiserror::Error;

#[derive(Error, Debug)]
pub enum BrowserAgentError {
    #[error("Browser launch failed: {0}")]
    LaunchFailed(String),

    #[error("Browser not launched")]
    BrowserNotLaunched,

    #[error("Tab creation failed: {0}")]
    TabCreationFailed(String),

    #[error("No active tab")]
    NoActiveTab,

    #[error("Navigation failed: {0}")]
    NavigationFailed(String),

    #[error("DOM extraction failed: {0}")]
    DomExtractionFailed(String),

    #[error("Element not found: {0}")]
    ElementNotFound(String),

    #[error("JavaScript execution failed: {0}")]
    JavaScriptFailed(String),

    #[error("JavaScript execution timeout")]
    JavaScriptTimeout,

    #[error("Screenshot failed: {0}")]
    ScreenshotFailed(String),

    #[error("Invalid selector type: {0}")]
    InvalidSelector(String),

    #[error("Action error: {0}")]
    ActionError(#[from] crate::actions::ActionError),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Configuration error: {0}")]
    ConfigurationError(String),

    #[error("Timeout error: {0}")]
    TimeoutError(String),

    #[error("Chrome error: {0}")]
    ChromeError(String),

    #[error("Anyhow error: {0}")]
    AnyhowError(String),
}

pub type Result<T> = std::result::Result<T, BrowserAgentError>;

// Convert anyhow::Error to BrowserAgentError
impl From<anyhow::Error> for BrowserAgentError {
    fn from(err: anyhow::Error) -> Self {
        BrowserAgentError::AnyhowError(err.to_string())
    }
}
