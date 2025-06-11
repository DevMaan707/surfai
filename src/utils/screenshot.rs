use crate::core::BrowserTrait;
use crate::errors::Result;
use base64;
pub struct ScreenshotManager;

impl ScreenshotManager {
    pub async fn take_base64<B: BrowserTrait>(browser: &B, tab: &B::TabHandle) -> Result<String> {
        let screenshot_bytes = browser.take_screenshot(tab).await?;
        Ok(base64::encode(screenshot_bytes))
    }
    pub async fn save_to_file<B: BrowserTrait>(
        browser: &B,
        tab: &B::TabHandle,
        file_path: &str,
    ) -> Result<()> {
        let screenshot_bytes = browser.take_screenshot(tab).await?;
        tokio::fs::write(file_path, screenshot_bytes)
            .await
            .map_err(|e| crate::errors::BrowserAgentError::IoError(e))?;
        Ok(())
    }
    pub async fn take_element_screenshot<B: BrowserTrait>(
        browser: &B,
        tab: &B::TabHandle,
        selector: &str,
    ) -> Result<Vec<u8>> {
        let script = format!(
            r#"
            (function() {{
                const element = document.querySelector('{}');
                if (!element) return null;

                element.scrollIntoView({{ block: 'center' }});
                const rect = element.getBoundingClientRect();

                return {{
                    x: rect.x,
                    y: rect.y,
                    width: rect.width,
                    height: rect.height
                }};
            }})()
            "#,
            selector.replace("'", "\\'")
        );

        let rect_result = browser.execute_script(tab, &script).await?;

        if rect_result.is_null() {
            return Err(crate::errors::BrowserAgentError::ElementNotFound(
                selector.to_string(),
            ));
        }
        browser.take_screenshot(tab).await
    }
    pub fn compare_screenshots(screenshot1: &[u8], screenshot2: &[u8]) -> f64 {
        if screenshot1.len() != screenshot2.len() {
            return 0.0;
        }

        let total_pixels = screenshot1.len();
        let different_pixels = screenshot1
            .iter()
            .zip(screenshot2.iter())
            .filter(|(a, b)| a != b)
            .count();

        1.0 - (different_pixels as f64 / total_pixels as f64)
    }
}
