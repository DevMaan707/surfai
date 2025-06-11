use crate::core::BrowserTrait;
use crate::errors::Result;
use serde_json::Value;

pub struct JavaScriptRunner;

impl JavaScriptRunner {
    pub async fn execute<B: BrowserTrait>(
        browser: &B,
        tab: &B::TabHandle,
        script: &str,
    ) -> Result<Value> {
        browser.execute_script(tab, script).await
    }
    pub async fn execute_with_timeout<B: BrowserTrait>(
        browser: &B,
        tab: &B::TabHandle,
        script: &str,
        timeout_ms: u64,
    ) -> Result<Value> {
        let execution = browser.execute_script(tab, script);

        tokio::time::timeout(tokio::time::Duration::from_millis(timeout_ms), execution)
            .await
            .map_err(|_| crate::errors::BrowserAgentError::JavaScriptTimeout)?
    }
    pub async fn wait_for_condition<B: BrowserTrait>(
        browser: &B,
        tab: &B::TabHandle,
        condition: &str,
        timeout_ms: u64,
        poll_interval_ms: u64,
    ) -> Result<bool> {
        let start_time = std::time::Instant::now();
        let timeout = tokio::time::Duration::from_millis(timeout_ms);
        let poll_interval = tokio::time::Duration::from_millis(poll_interval_ms);

        while start_time.elapsed() < timeout {
            let result = browser.execute_script(tab, condition).await?;
            if let Some(boolean_result) = result.as_bool() {
                if boolean_result {
                    return Ok(true);
                }
            }

            tokio::time::sleep(poll_interval).await;
        }

        Ok(false)
    }
    pub async fn inject_css<B: BrowserTrait>(
        browser: &B,
        tab: &B::TabHandle,
        css: &str,
    ) -> Result<()> {
        let script = format!(
            r#"
            (function() {{
                const style = document.createElement('style');
                style.textContent = `{}`;
                document.head.appendChild(style);
                return true;
            }})()
            "#,
            css.replace("`", "\\`")
        );

        browser.execute_script(tab, &script).await?;
        Ok(())
    }
    pub async fn get_element<B: BrowserTrait>(
        browser: &B,
        tab: &B::TabHandle,
        selector: &str,
        selector_type: &str,
    ) -> Result<Value> {
        let script = match selector_type {
            "css" => format!("document.querySelector('{}')", selector.replace("'", "\\'")),
            "xpath" => format!(
                "document.evaluate('{}', document, null, XPathResult.FIRST_ORDERED_NODE_TYPE, null).singleNodeValue",
                selector.replace("'", "\\'")
            ),
            "id" => format!(
                "document.getElementById('{}')",
                selector.replace("'", "\\'")
            ),
            _ => {
                return Err(crate::errors::BrowserAgentError::InvalidSelector(
                    selector_type.to_string(),
                ));
            }
        };

        browser.execute_script(tab, &script).await
    }
}
