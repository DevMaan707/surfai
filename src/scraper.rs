use crate::error::{BrowserError, BrowserResult};
use chromiumoxide::Page;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageData {
    pub title: String,
    pub url: String,
    pub content: String,
    pub links: Vec<String>,
    pub images: Vec<String>,
    pub forms: Vec<FormData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormData {
    pub action: String,
    pub method: String,
    pub inputs: Vec<String>,
}

pub struct PageScraper {
    page: Page,
}

impl PageScraper {
    pub fn new(page: Page) -> Self {
        Self { page }
    }

    pub async fn scrape_page(&self) -> BrowserResult<PageData> {
        let title = self
            .page
            .get_title()
            .await
            .map_err(|e| BrowserError::Script(e.to_string()))?;

        let url = self
            .page
            .url()
            .await
            .map_err(|e| BrowserError::Script(e.to_string()))?;

        let content = self
            .page
            .content()
            .await
            .map_err(|e| BrowserError::Script(e.to_string()))?;

        let links = self.extract_links().await?;
        let images = self.extract_images().await?;
        let forms = self.extract_forms().await?;

        Ok(PageData {
            title: title.unwrap_or_default(),
            url: url.unwrap_or_default(),
            content,
            links,
            images,
            forms,
        })
    }

    pub async fn take_screenshot(&self) -> BrowserResult<Vec<u8>> {
        let screenshot = self
            .page
            .screenshot(chromiumoxide::page::ScreenshotParams::builder().build())
            .await
            .map_err(|e| BrowserError::Script(e.to_string()))?;

        Ok(screenshot)
    }

    async fn extract_links(&self) -> BrowserResult<Vec<String>> {
        let script = r#"
            (() => {
                return Array.from(document.querySelectorAll('a[href]')).map(a => a.href);
            })();
        "#;

        let result = self
            .page
            .evaluate(script)
            .await
            .map_err(|e| BrowserError::Script(e.to_string()))?;

        let links: Vec<String> = result
            .into_value()
            .map_err(|e| BrowserError::Script(e.to_string()))?;

        Ok(links)
    }

    async fn extract_images(&self) -> BrowserResult<Vec<String>> {
        let script = r#"
            (() => {
                return Array.from(document.querySelectorAll('img[src]')).map(img => img.src);
            })();
        "#;

        let result = self
            .page
            .evaluate(script)
            .await
            .map_err(|e| BrowserError::Script(e.to_string()))?;

        let images: Vec<String> = result
            .into_value()
            .map_err(|e| BrowserError::Script(e.to_string()))?;

        Ok(images)
    }

    async fn extract_forms(&self) -> BrowserResult<Vec<FormData>> {
        let script = r#"
            (() => {
                return Array.from(document.querySelectorAll('form')).map(form => ({
                    action: form.action || '',
                    method: form.method || 'GET',
                    inputs: Array.from(form.querySelectorAll('input')).map(input => input.name || input.id || input.type)
                }));
            })();
        "#;

        let result = self
            .page
            .evaluate(script)
            .await
            .map_err(|e| BrowserError::Script(e.to_string()))?;

        let forms: Vec<FormData> = result
            .into_value()
            .map_err(|e| BrowserError::Script(e.to_string()))?;

        Ok(forms)
    }
}
