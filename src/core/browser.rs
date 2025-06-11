use crate::errors::Result;
use async_trait::async_trait;
use serde_json::Value;

#[async_trait]
pub trait BrowserTrait: Send + Sync {
    type TabHandle: Send + Sync;

    /// Launch a new browser instance
    async fn launch(&mut self, config: &crate::core::Config) -> Result<()>;

    /// Create a new tab/page
    async fn new_tab(&self) -> Result<Self::TabHandle>;

    /// Navigate to a URL
    async fn navigate(&self, tab: &Self::TabHandle, url: &str) -> Result<()>;

    /// Execute JavaScript in the browser
    async fn execute_script(&self, tab: &Self::TabHandle, script: &str) -> Result<Value>;

    /// Take a screenshot
    async fn take_screenshot(&self, tab: &Self::TabHandle) -> Result<Vec<u8>>;

    /// Get current URL
    async fn get_url(&self, tab: &Self::TabHandle) -> Result<String>;

    /// Get page title
    async fn get_title(&self, tab: &Self::TabHandle) -> Result<String>;

    /// Wait for navigation to complete
    async fn wait_for_navigation(&self, tab: &Self::TabHandle, timeout_ms: u64) -> Result<()>;

    /// Check if browser is still running
    fn is_running(&self) -> bool;

    /// Close the browser
    async fn close(&mut self) -> Result<()>;
}

/// Browser capabilities that can be queried
#[derive(Debug, Clone)]
pub struct BrowserCapabilities {
    pub supports_javascript: bool,
    pub supports_screenshots: bool,
    pub supports_network_interception: bool,
    pub supports_mobile_emulation: bool,
}
