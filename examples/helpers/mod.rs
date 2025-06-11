use browser_ragent::core::SessionTrait;
use browser_ragent::errors::Result;
use browser_ragent::{Config, DefaultBrowser, DefaultSession, DomState};

pub struct TestHelper;

impl TestHelper {
    pub async fn create_test_session() -> Result<DefaultSession> {
        let browser = DefaultBrowser::new();
        let config = Config {
            browser: browser_ragent::core::config::BrowserConfig {
                headless: true,
                ..Default::default()
            },
            ..Default::default()
        };

        DefaultSession::new(browser, config).await
    }

    pub async fn create_test_session_with_config(config: Config) -> Result<DefaultSession> {
        let browser = DefaultBrowser::new();
        DefaultSession::new(browser, config).await
    }

    pub async fn navigate_and_extract(session: &mut DefaultSession, url: &str) -> Result<DomState> {
        session.navigate_and_wait(url).await?;
        session.get_page_state(false).await
    }

    pub fn count_elements_by_type(dom_state: &DomState, element_type: &str) -> usize {
        dom_state
            .elements
            .iter()
            .filter(|e| e.tag_name == element_type)
            .count()
    }

    pub fn find_elements_with_text(
        dom_state: &DomState,
        text: &str,
    ) -> Vec<browser_ragent::dom::DomElement> {
        dom_state
            .find_elements_by_text(text)
            .into_iter()
            .cloned()
            .collect()
    }

    pub fn get_page_stats(dom_state: &DomState) -> PageStats {
        PageStats {
            total_elements: dom_state.elements.len(),
            clickable_elements: dom_state.clickable_elements.len(),
            input_elements: dom_state.input_elements.len(),
            text_elements: dom_state.text_elements.len(),
            images: Self::count_elements_by_type(dom_state, "img"),
            links: Self::count_elements_by_type(dom_state, "a"),
            buttons: Self::count_elements_by_type(dom_state, "button"),
            forms: Self::count_elements_by_type(dom_state, "form"),
        }
    }

    /// Enhanced element waiting using the session's execute_script method
    pub async fn wait_for_element(
        session: &DefaultSession,
        selector: &str,
        timeout_ms: u64,
    ) -> Result<bool> {
        let condition = format!(
            "document.querySelector('{}') !== null",
            selector.replace("'", "\\'")
        );

        Self::wait_for_condition(session, &condition, timeout_ms, 100).await
    }

    /// Wait for a JavaScript condition to be true
    pub async fn wait_for_condition(
        session: &DefaultSession,
        condition: &str,
        timeout_ms: u64,
        poll_interval_ms: u64,
    ) -> Result<bool> {
        let start_time = std::time::Instant::now();
        let timeout = tokio::time::Duration::from_millis(timeout_ms);
        let poll_interval = tokio::time::Duration::from_millis(poll_interval_ms);

        while start_time.elapsed() < timeout {
            match session.execute_script(condition).await {
                Ok(result) => {
                    if let Some(boolean_result) = result.as_bool() {
                        if boolean_result {
                            return Ok(true);
                        }
                    }
                }
                Err(_) => {
                    // Continue waiting if script execution fails
                }
            }

            tokio::time::sleep(poll_interval).await;
        }

        Ok(false)
    }

    /// Assert element exists using the session interface
    pub async fn assert_element_exists(session: &DefaultSession, selector: &str) -> Result<()> {
        let exists = Self::wait_for_element(session, selector, 5000).await?;
        if !exists {
            return Err(browser_ragent::errors::BrowserAgentError::ElementNotFound(
                selector.to_string(),
            ));
        }
        Ok(())
    }

    /// Assert page title contains text
    pub async fn assert_title_contains(session: &DefaultSession, text: &str) -> Result<()> {
        let title_script = "document.title";
        let title_result = session.execute_script(title_script).await?;
        let title = title_result.as_str().unwrap_or("");

        if !title.to_lowercase().contains(&text.to_lowercase()) {
            return Err(
                browser_ragent::errors::BrowserAgentError::ConfigurationError(format!(
                    "Title '{}' does not contain '{}'",
                    title, text
                )),
            );
        }
        Ok(())
    }

    /// Assert URL contains text
    pub async fn assert_url_contains(session: &DefaultSession, text: &str) -> Result<()> {
        let url = session.current_url().await?;
        if !url.to_lowercase().contains(&text.to_lowercase()) {
            return Err(
                browser_ragent::errors::BrowserAgentError::ConfigurationError(format!(
                    "URL '{}' does not contain '{}'",
                    url, text
                )),
            );
        }
        Ok(())
    }

    /// Wait for element to be visible
    pub async fn wait_for_element_visible(
        session: &DefaultSession,
        selector: &str,
        timeout_ms: u64,
    ) -> Result<bool> {
        let condition = format!(
            r#"
            (function() {{
                const element = document.querySelector('{}');
                if (!element) return false;
                const rect = element.getBoundingClientRect();
                const style = window.getComputedStyle(element);
                return rect.width > 0 &&
                       rect.height > 0 &&
                       style.visibility !== 'hidden' &&
                       style.display !== 'none' &&
                       parseFloat(style.opacity) > 0;
            }})()
            "#,
            selector.replace("'", "\\'")
        );

        Self::wait_for_condition(session, &condition, timeout_ms, 100).await
    }

    /// Wait for element to be clickable
    pub async fn wait_for_element_clickable(
        session: &DefaultSession,
        selector: &str,
        timeout_ms: u64,
    ) -> Result<bool> {
        let condition = format!(
            r#"
            (function() {{
                const element = document.querySelector('{}');
                if (!element) return false;
                const rect = element.getBoundingClientRect();
                const style = window.getComputedStyle(element);
                return rect.width > 0 &&
                       rect.height > 0 &&
                       style.visibility !== 'hidden' &&
                       style.display !== 'none' &&
                       parseFloat(style.opacity) > 0 &&
                       !element.disabled &&
                       style.pointerEvents !== 'none';
            }})()
            "#,
            selector.replace("'", "\\'")
        );

        Self::wait_for_condition(session, &condition, timeout_ms, 100).await
    }

    /// Get element text content
    pub async fn get_element_text(
        session: &DefaultSession,
        selector: &str,
    ) -> Result<Option<String>> {
        let script = format!(
            r#"
            (function() {{
                const element = document.querySelector('{}');
                if (element) {{
                    return element.textContent || element.innerText;
                }}
                return null;
            }})()
            "#,
            selector.replace("'", "\\'")
        );

        let result = session.execute_script(&script).await?;
        Ok(result.as_str().map(|s| s.to_string()))
    }

    /// Get element attribute
    pub async fn get_element_attribute(
        session: &DefaultSession,
        selector: &str,
        attribute: &str,
    ) -> Result<Option<String>> {
        let script = format!(
            r#"
            (function() {{
                const element = document.querySelector('{}');
                if (element) {{
                    return element.getAttribute('{}');
                }}
                return null;
            }})()
            "#,
            selector.replace("'", "\\'"),
            attribute.replace("'", "\\'")
        );

        let result = session.execute_script(&script).await?;
        Ok(result.as_str().map(|s| s.to_string()))
    }

    /// Check if element exists
    pub async fn element_exists(session: &DefaultSession, selector: &str) -> Result<bool> {
        let script = format!(
            "document.querySelector('{}') !== null",
            selector.replace("'", "\\'")
        );

        let result = session.execute_script(&script).await?;
        Ok(result.as_bool().unwrap_or(false))
    }

    /// Get element count
    pub async fn get_element_count(session: &DefaultSession, selector: &str) -> Result<usize> {
        let script = format!(
            "document.querySelectorAll('{}').length",
            selector.replace("'", "\\'")
        );

        let result = session.execute_script(&script).await?;
        Ok(result.as_u64().unwrap_or(0) as usize)
    }

    /// Scroll element into view
    pub async fn scroll_to_element(session: &DefaultSession, selector: &str) -> Result<()> {
        let script = format!(
            r#"
            (function() {{
                const element = document.querySelector('{}');
                if (element) {{
                    element.scrollIntoView({{ behavior: 'smooth', block: 'center' }});
                    return true;
                }}
                return false;
            }})()
            "#,
            selector.replace("'", "\\'")
        );

        let result = session.execute_script(&script).await?;
        if result.as_bool().unwrap_or(false) {
            Ok(())
        } else {
            Err(browser_ragent::errors::BrowserAgentError::ElementNotFound(
                selector.to_string(),
            ))
        }
    }

    /// Enhanced page load waiting
    pub async fn wait_for_page_load(session: &DefaultSession, timeout_ms: u64) -> Result<()> {
        let condition = r#"
            document.readyState === 'complete' &&
            (!window.jQuery || jQuery.active === 0)
        "#;

        Self::wait_for_condition(session, condition, timeout_ms, 100).await?;
        Ok(())
    }

    /// Wait for network to be idle (no pending requests)
    pub async fn wait_for_network_idle(session: &DefaultSession, timeout_ms: u64) -> Result<()> {
        let condition = r#"
            (function() {
                // Basic check for common loading indicators
                const loaders = document.querySelectorAll('.loading, .spinner, [class*="load"]');
                const hasVisibleLoaders = Array.from(loaders).some(el => {
                    const style = window.getComputedStyle(el);
                    return style.display !== 'none' && style.visibility !== 'hidden';
                });
                return !hasVisibleLoaders && document.readyState === 'complete';
            })()
        "#;

        Self::wait_for_condition(session, condition, timeout_ms, 500).await?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct PageStats {
    pub total_elements: usize,
    pub clickable_elements: usize,
    pub input_elements: usize,
    pub text_elements: usize,
    pub images: usize,
    pub links: usize,
    pub buttons: usize,
    pub forms: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio_test;

    #[tokio::test]
    async fn test_session_creation() {
        let session = TestHelper::create_test_session().await;
        assert!(session.is_ok());
    }

    #[tokio::test]
    async fn test_navigation_and_extraction() {
        let session = TestHelper::create_test_session().await.unwrap();
        let result = TestHelper::navigate_and_extract(&mut session, "https://example.com").await;

        assert!(result.is_ok());
        let dom_state = result.unwrap();
        assert!(!dom_state.elements.is_empty());
        assert!(!dom_state.url.is_empty());
        assert!(!dom_state.title.is_empty());
    }

    #[tokio::test]
    async fn test_element_finding() {
        let session = TestHelper::create_test_session().await.unwrap();
        let dom_state = TestHelper::navigate_and_extract(&mut session, "https://example.com")
            .await
            .unwrap();

        let text_elements = TestHelper::find_elements_with_text(&dom_state, "example");
        let stats = TestHelper::get_page_stats(&dom_state);

        assert!(stats.total_elements > 0);
        println!("Page stats: {:?}", stats);
    }

    #[tokio::test]
    async fn test_element_waiting() {
        let session = TestHelper::create_test_session().await.unwrap();
        session
            .navigate_and_wait("https://example.com")
            .await
            .unwrap();

        let found = TestHelper::wait_for_element(&session, "h1", 5000)
            .await
            .unwrap();
        assert!(found);
    }

    #[tokio::test]
    async fn test_element_visibility() {
        let session = TestHelper::create_test_session().await.unwrap();
        session
            .navigate_and_wait("https://example.com")
            .await
            .unwrap();

        let visible = TestHelper::wait_for_element_visible(&session, "h1", 5000)
            .await
            .unwrap();
        assert!(visible);
    }

    #[tokio::test]
    async fn test_element_text_extraction() {
        let session = TestHelper::create_test_session().await.unwrap();
        session
            .navigate_and_wait("https://example.com")
            .await
            .unwrap();

        let text = TestHelper::get_element_text(&session, "h1").await.unwrap();
        assert!(text.is_some());
        println!("H1 text: {:?}", text);
    }

    #[tokio::test]
    async fn test_element_existence() {
        let session = TestHelper::create_test_session().await.unwrap();
        session
            .navigate_and_wait("https://example.com")
            .await
            .unwrap();

        let exists = TestHelper::element_exists(&session, "h1").await.unwrap();
        assert!(exists);

        let not_exists = TestHelper::element_exists(&session, "non-existent-element")
            .await
            .unwrap();
        assert!(!not_exists);
    }
}
