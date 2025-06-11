use crate::core::{BrowserTrait, Config};
use crate::dom::DomState;
use crate::errors::Result;
use async_trait::async_trait;

#[async_trait]
pub trait SessionTrait<B: BrowserTrait>: Send + Sync {
    async fn new(browser: B, config: Config) -> Result<Self>
    where
        Self: Sized;

    async fn navigate_and_wait(&mut self, url: &str) -> Result<()>;

    async fn get_page_state(&self, include_screenshot: bool) -> Result<DomState>;

    async fn click(&self, selector: &str) -> Result<()>;

    async fn type_text(&self, selector: &str, text: &str) -> Result<()>;

    async fn execute_script(&self, script: &str) -> Result<serde_json::Value>;

    async fn screenshot(&self) -> Result<Vec<u8>>;

    async fn current_url(&self) -> Result<String>;

    async fn close(&self) -> Result<()>;
}
