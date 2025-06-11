use crate::core::{BrowserCapabilities, BrowserTrait, Config};
use crate::errors::{BrowserAgentError, Result};
use async_trait::async_trait;
use headless_chrome::{Browser, LaunchOptions, Tab};
use serde_json::Value;
use std::ffi::OsStr;
use std::sync::Arc;

/// Chrome browser implementation
pub struct ChromeBrowser {
    browser: Option<Browser>,
    capabilities: BrowserCapabilities,
}

impl ChromeBrowser {
    pub fn new() -> Self {
        Self {
            browser: None,
            capabilities: BrowserCapabilities {
                supports_javascript: true,
                supports_screenshots: true,
                supports_network_interception: true,
                supports_mobile_emulation: true,
            },
        }
    }

    pub fn capabilities(&self) -> &BrowserCapabilities {
        &self.capabilities
    }
}

#[async_trait]
impl BrowserTrait for ChromeBrowser {
    type TabHandle = Arc<Tab>;

    async fn launch(&mut self, config: &Config) -> Result<()> {
        let window_size_arg = format!(
            "--window-size={},{}",
            config.browser.viewport.width, config.browser.viewport.height
        );

        let user_agent_arg = config
            .browser
            .user_agent
            .as_ref()
            .map(|ua| format!("--user-agent={}", ua));

        let mut args = vec![
            OsStr::new("--no-sandbox"),
            OsStr::new("--disable-dev-shm-usage"),
            OsStr::new(&window_size_arg),
        ];

        if let Some(ref ua_arg) = user_agent_arg {
            args.push(OsStr::new(ua_arg));
        }

        if config.browser.disable_images {
            args.push(OsStr::new("--blink-settings=imagesEnabled=false"));
        }

        // Add custom args
        for arg in &config.browser.args {
            args.push(OsStr::new(arg));
        }

        let launch_options = LaunchOptions::default_builder()
            .headless(config.browser.headless)
            .args(args)
            .build()
            .map_err(|e| BrowserAgentError::LaunchFailed(e.to_string()))?;

        let browser = Browser::new(launch_options)
            .map_err(|e| BrowserAgentError::LaunchFailed(e.to_string()))?;

        self.browser = Some(browser);
        Ok(())
    }

    async fn new_tab(&self) -> Result<Self::TabHandle> {
        let browser = self
            .browser
            .as_ref()
            .ok_or_else(|| BrowserAgentError::BrowserNotLaunched)?;

        let tab = browser
            .new_tab()
            .map_err(|e| BrowserAgentError::TabCreationFailed(e.to_string()))?;

        Ok(tab)
    }

    async fn navigate(&self, tab: &Self::TabHandle, url: &str) -> Result<()> {
        tab.navigate_to(url)
            .map_err(|e| BrowserAgentError::NavigationFailed(e.to_string()))?;

        tab.wait_until_navigated()
            .map_err(|e| BrowserAgentError::NavigationFailed(e.to_string()))?;

        Ok(())
    }

    async fn execute_script(&self, tab: &Self::TabHandle, script: &str) -> Result<Value> {
        let result = tab
            .evaluate(script, false)
            .map_err(|e| BrowserAgentError::JavaScriptFailed(e.to_string()))?;

        Ok(result.value.unwrap_or(Value::Null))
    }

    async fn take_screenshot(&self, tab: &Self::TabHandle) -> Result<Vec<u8>> {
        let screenshot = tab
            .capture_screenshot(
                headless_chrome::protocol::cdp::Page::CaptureScreenshotFormatOption::Png,
                None,
                None,
                true,
            )
            .map_err(|e| BrowserAgentError::ScreenshotFailed(e.to_string()))?;

        Ok(screenshot)
    }

    async fn get_url(&self, tab: &Self::TabHandle) -> Result<String> {
        Ok(tab.get_url())
    }

    async fn get_title(&self, tab: &Self::TabHandle) -> Result<String> {
        let result = self.execute_script(tab, "document.title").await?;
        Ok(result.as_str().unwrap_or("").to_string())
    }

    async fn wait_for_navigation(&self, tab: &Self::TabHandle, timeout_ms: u64) -> Result<()> {
        tokio::time::sleep(tokio::time::Duration::from_millis(timeout_ms)).await;
        Ok(())
    }

    fn is_running(&self) -> bool {
        self.browser.is_some()
    }

    async fn close(&mut self) -> Result<()> {
        self.browser = None;
        Ok(())
    }
}
