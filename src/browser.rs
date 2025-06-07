use crate::dom::{DomProcessor, DomState};
use crate::errors::{BrowserError, Result};
use crate::types::BrowserConfig;
use headless_chrome::{Browser, LaunchOptions, Tab};
use std::ffi::OsStr;
use std::sync::Arc;
use std::time::Duration;
#[derive(Debug, Clone)]
pub struct ElementHighlight {
    pub element_id: String,
    pub element_number: usize,
    pub color: String,
    pub element_type: String,
}
pub struct BrowserSession {
    browser: Browser,
    tab: Arc<Tab>,
    config: BrowserConfig,
    dom_processor: DomProcessor,
}

impl BrowserSession {
    pub async fn new(config: BrowserConfig) -> Result<Self> {
        // Create strings first to ensure they live long enough
        let window_size_arg = format!(
            "--window-size={},{}",
            config.viewport.width, config.viewport.height
        );
        let user_agent_arg = config
            .user_agent
            .as_ref()
            .map(|ua| format!("--user-agent={}", ua));

        // Create base args
        let mut args = vec![
            OsStr::new("--no-sandbox"),
            OsStr::new("--disable-dev-shm-usage"),
            OsStr::new(&window_size_arg),
        ];

        // Add user agent if provided
        if let Some(ref ua_arg) = user_agent_arg {
            args.push(OsStr::new(ua_arg));
        }

        // Add image disabling if requested
        if config.disable_images {
            args.push(OsStr::new("--blink-settings=imagesEnabled=false"));
        }

        let launch_options = LaunchOptions::default_builder()
            .headless(config.headless)
            .args(args)
            .build()
            .map_err(|e| BrowserError::LaunchFailed(e.to_string()))?;

        let browser =
            Browser::new(launch_options).map_err(|e| BrowserError::LaunchFailed(e.to_string()))?;

        let tab = browser
            .new_tab()
            .map_err(|e| BrowserError::LaunchFailed(e.to_string()))?;

        // tab is already Arc<Tab> from new_tab()
        let tab_arc = tab;

        // Set viewport using emulation
        let viewport_js = format!(
            r#"
            Object.defineProperty(window, 'innerWidth', {{ value: {}, configurable: true }});
            Object.defineProperty(window, 'innerHeight', {{ value: {}, configurable: true }});
            Object.defineProperty(window, 'outerWidth', {{ value: {}, configurable: true }});
            Object.defineProperty(window, 'outerHeight', {{ value: {}, configurable: true }});
        "#,
            config.viewport.width,
            config.viewport.height,
            config.viewport.width,
            config.viewport.height
        );

        tab_arc
            .evaluate(&viewport_js, false)
            .map_err(|e| BrowserError::LaunchFailed(e.to_string()))?;

        let dom_processor = DomProcessor::new(tab_arc.clone());

        Ok(Self {
            browser,
            tab: tab_arc,
            config,
            dom_processor,
        })
    }

    pub async fn navigate(&self, url: &str) -> Result<()> {
        self.tab
            .navigate_to(url)
            .map_err(|e| BrowserError::NavigationFailed(e.to_string()))?;

        // Wait for navigation to complete
        self.tab
            .wait_until_navigated()
            .map_err(|e| BrowserError::NavigationFailed(e.to_string()))?;

        Ok(())
    }

    pub async fn get_dom_state(&self, include_screenshot: bool) -> Result<DomState> {
        self.dom_processor
            .extract_dom_state(include_screenshot)
            .await
    }

    pub async fn get_dom_state_with_labels(&self, include_screenshot: bool) -> Result<DomState> {
        let mut dom_state = self
            .dom_processor
            .extract_dom_state(include_screenshot)
            .await?;

        // Add AI-friendly labels to elements
        self.dom_processor
            .label_elements(&mut dom_state.elements)
            .await?;
        self.dom_processor
            .label_elements(&mut dom_state.clickable_elements)
            .await?;
        self.dom_processor
            .label_elements(&mut dom_state.input_elements)
            .await?;
        self.dom_processor
            .label_elements(&mut dom_state.text_elements)
            .await?;

        Ok(dom_state)
    }

    pub async fn click_element(&self, css_selector: &str) -> Result<()> {
        self.tab
            .find_element(css_selector)
            .map_err(|e| BrowserError::ElementNotFound(e.to_string()))?
            .click()
            .map_err(|e| BrowserError::JavaScriptFailed(e.to_string()))?;

        Ok(())
    }

    pub async fn click_element_by_xpath(&self, xpath: &str) -> Result<()> {
        let js_code = format!(
            r#"
            (function() {{
                const element = document.evaluate('{}', document, null, XPathResult.FIRST_ORDERED_NODE_TYPE, null).singleNodeValue;
                if (element) {{
                    element.click();
                    return true;
                }}
                return false;
            }})()
        "#,
            xpath.replace("'", "\\'")
        );

        let result = self
            .tab
            .evaluate(&js_code, false)
            .map_err(|e| BrowserError::JavaScriptFailed(e.to_string()))?;

        if let Some(value) = result.value {
            if value.as_bool() == Some(true) {
                return Ok(());
            }
        }

        Err(BrowserError::ElementNotFound(format!(
            "Element with xpath '{}' not found",
            xpath
        )))
    }

    pub async fn type_text(&self, css_selector: &str, text: &str) -> Result<()> {
        let element = self
            .tab
            .find_element(css_selector)
            .map_err(|e| BrowserError::ElementNotFound(e.to_string()))?;

        element
            .click()
            .map_err(|e| BrowserError::JavaScriptFailed(e.to_string()))?;

        element
            .type_into(text)
            .map_err(|e| BrowserError::JavaScriptFailed(e.to_string()))?;

        Ok(())
    }

    pub async fn type_text_by_xpath(&self, xpath: &str, text: &str) -> Result<()> {
        let js_code = format!(
            r#"
            (function() {{
                const element = document.evaluate('{}', document, null, XPathResult.FIRST_ORDERED_NODE_TYPE, null).singleNodeValue;
                if (element) {{
                    element.focus();
                    element.value = '{}';
                    element.dispatchEvent(new Event('input', {{ bubbles: true }}));
                    element.dispatchEvent(new Event('change', {{ bubbles: true }}));
                    return true;
                }}
                return false;
            }})()
        "#,
            xpath.replace("'", "\\'"),
            text.replace("'", "\\'")
        );

        let result = self
            .tab
            .evaluate(&js_code, false)
            .map_err(|e| BrowserError::JavaScriptFailed(e.to_string()))?;

        if let Some(value) = result.value {
            if value.as_bool() == Some(true) {
                return Ok(());
            }
        }

        Err(BrowserError::ElementNotFound(format!(
            "Element with xpath '{}' not found",
            xpath
        )))
    }

    pub async fn clear_input(&self, css_selector: &str) -> Result<()> {
        let js_code = format!(
            r#"
            (function() {{
                const element = document.querySelector('{}');
                if (element) {{
                    element.value = '';
                    element.dispatchEvent(new Event('input', {{ bubbles: true }}));
                    element.dispatchEvent(new Event('change', {{ bubbles: true }}));
                    return true;
                }}
                return false;
            }})()
        "#,
            css_selector.replace("'", "\\'")
        );

        let result = self
            .tab
            .evaluate(&js_code, false)
            .map_err(|e| BrowserError::JavaScriptFailed(e.to_string()))?;

        if let Some(value) = result.value {
            if value.as_bool() == Some(true) {
                return Ok(());
            }
        }

        Err(BrowserError::ElementNotFound(format!(
            "Element with selector '{}' not found",
            css_selector
        )))
    }

    pub async fn scroll_to_element(&self, css_selector: &str) -> Result<()> {
        let js_code = format!(
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
            css_selector.replace("'", "\\'")
        );

        let result = self
            .tab
            .evaluate(&js_code, false)
            .map_err(|e| BrowserError::JavaScriptFailed(e.to_string()))?;

        if let Some(value) = result.value {
            if value.as_bool() == Some(true) {
                return Ok(());
            }
        }

        Err(BrowserError::ElementNotFound(format!(
            "Element with selector '{}' not found",
            css_selector
        )))
    }

    pub async fn scroll_page(&self, direction: &str, amount: i32) -> Result<()> {
        let js_code = match direction {
            "up" => format!("window.scrollBy(0, -{})", amount),
            "down" => format!("window.scrollBy(0, {})", amount),
            "left" => format!("window.scrollBy(-{}, 0)", amount),
            "right" => format!("window.scrollBy({}, 0)", amount),
            _ => {
                return Err(BrowserError::JavaScriptFailed(
                    "Invalid scroll direction".to_string(),
                ));
            }
        };

        self.tab
            .evaluate(&js_code, false)
            .map_err(|e| BrowserError::JavaScriptFailed(e.to_string()))?;

        Ok(())
    }

    pub async fn get_page_source(&self) -> Result<String> {
        let js_result = self
            .tab
            .evaluate("document.documentElement.outerHTML", false)
            .map_err(|e| BrowserError::JavaScriptFailed(e.to_string()))?;

        js_result
            .value
            .and_then(|v| v.as_str().map(|s| s.to_string()))
            .ok_or_else(|| {
                BrowserError::DomExtractionFailed("Failed to get page source".to_string())
            })
    }

    pub fn get_current_url(&self) -> String {
        self.tab.get_url()
    }

    pub async fn get_page_title(&self) -> Result<String> {
        let js_result = self
            .tab
            .evaluate("document.title", false)
            .map_err(|e| BrowserError::JavaScriptFailed(e.to_string()))?;

        Ok(js_result
            .value
            .and_then(|v| v.as_str().map(|s| s.to_string()))
            .unwrap_or_default())
    }

    pub async fn wait_for_element(&self, css_selector: &str, timeout_ms: u64) -> Result<()> {
        self.tab
            .wait_for_element_with_custom_timeout(css_selector, Duration::from_millis(timeout_ms))
            .map_err(|e| BrowserError::ElementNotFound(e.to_string()))?;

        Ok(())
    }

    pub async fn wait_for_navigation(&self, timeout_ms: u64) -> Result<()> {
        tokio::time::sleep(Duration::from_millis(timeout_ms)).await;
        Ok(())
    }

    pub async fn execute_javascript(&self, script: &str) -> Result<serde_json::Value> {
        let result = self
            .tab
            .evaluate(script, false)
            .map_err(|e| BrowserError::JavaScriptFailed(e.to_string()))?;

        Ok(result.value.unwrap_or(serde_json::Value::Null))
    }

    pub async fn take_screenshot(&self) -> Result<Vec<u8>> {
        let screenshot = self
            .tab
            .capture_screenshot(
                headless_chrome::protocol::cdp::Page::CaptureScreenshotFormatOption::Png,
                None,
                None,
                true,
            )
            .map_err(|e| BrowserError::JavaScriptFailed(e.to_string()))?;

        Ok(screenshot)
    }

    pub async fn take_screenshot_base64(&self) -> Result<String> {
        let screenshot = self.take_screenshot().await?;
        Ok(base64::encode(screenshot))
    }

    pub async fn go_back(&self) -> Result<()> {
        self.tab
            .evaluate("window.history.back()", false)
            .map_err(|e| BrowserError::JavaScriptFailed(e.to_string()))?;
        Ok(())
    }

    pub async fn go_forward(&self) -> Result<()> {
        self.tab
            .evaluate("window.history.forward()", false)
            .map_err(|e| BrowserError::JavaScriptFailed(e.to_string()))?;
        Ok(())
    }

    pub async fn refresh(&self) -> Result<()> {
        self.tab
            .reload(false, None)
            .map_err(|e| BrowserError::NavigationFailed(e.to_string()))?;
        Ok(())
    }

    pub async fn close(&self) -> Result<()> {
        Ok(())
    }

    pub async fn get_cookies(&self) -> Result<Vec<serde_json::Value>> {
        let js_result = self
            .tab
            .evaluate("document.cookie", false)
            .map_err(|e| BrowserError::JavaScriptFailed(e.to_string()))?;

        let cookies_str = js_result
            .value
            .and_then(|v| v.as_str().map(|s| s.to_string()))
            .unwrap_or_default();

        let cookies: Vec<serde_json::Value> = cookies_str
            .split(';')
            .filter_map(|cookie| {
                let parts: Vec<&str> = cookie.trim().splitn(2, '=').collect();
                if parts.len() == 2 {
                    Some(serde_json::json!({
                        "name": parts[0].trim(),
                        "value": parts[1].trim()
                    }))
                } else {
                    None
                }
            })
            .collect();

        Ok(cookies)
    }

    pub async fn set_cookie(&self, name: &str, value: &str, domain: Option<&str>) -> Result<()> {
        let domain_part = domain
            .map(|d| format!("; domain={}", d))
            .unwrap_or_default();
        let js_code = format!("document.cookie = '{}={}{}';", name, value, domain_part);

        self.tab
            .evaluate(&js_code, false)
            .map_err(|e| BrowserError::JavaScriptFailed(e.to_string()))?;

        Ok(())
    }

    pub async fn get_element_attribute(
        &self,
        css_selector: &str,
        attribute: &str,
    ) -> Result<Option<String>> {
        let js_code = format!(
            r#"
            (function() {{
                const element = document.querySelector('{}');
                if (element) {{
                    return element.getAttribute('{}');
                }}
                return null;
            }})()
        "#,
            css_selector.replace("'", "\\'"),
            attribute.replace("'", "\\'")
        );

        let result = self
            .tab
            .evaluate(&js_code, false)
            .map_err(|e| BrowserError::JavaScriptFailed(e.to_string()))?;

        Ok(result.value.and_then(|v| v.as_str().map(|s| s.to_string())))
    }

    pub async fn get_element_text(&self, css_selector: &str) -> Result<Option<String>> {
        let js_code = format!(
            r#"
            (function() {{
                const element = document.querySelector('{}');
                if (element) {{
                    return element.textContent || element.innerText;
                }}
                return null;
            }})()
        "#,
            css_selector.replace("'", "\\'")
        );

        let result = self
            .tab
            .evaluate(&js_code, false)
            .map_err(|e| BrowserError::JavaScriptFailed(e.to_string()))?;

        Ok(result.value.and_then(|v| v.as_str().map(|s| s.to_string())))
    }

    pub async fn is_element_visible(&self, css_selector: &str) -> Result<bool> {
        let js_code = format!(
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
            css_selector.replace("'", "\\'")
        );

        let result = self
            .tab
            .evaluate(&js_code, false)
            .map_err(|e| BrowserError::JavaScriptFailed(e.to_string()))?;

        Ok(result.value.and_then(|v| v.as_bool()).unwrap_or(false))
    }

    pub async fn wait_for_page_load(&self, timeout_ms: u64) -> Result<()> {
        let js_code = r#"
            (function() {
                return document.readyState === 'complete';
            })()
        "#;

        let start_time = std::time::Instant::now();
        let timeout = Duration::from_millis(timeout_ms);

        while start_time.elapsed() < timeout {
            let result = self
                .tab
                .evaluate(js_code, false)
                .map_err(|e| BrowserError::JavaScriptFailed(e.to_string()))?;

            if let Some(value) = result.value {
                if value.as_bool() == Some(true) {
                    return Ok(());
                }
            }

            tokio::time::sleep(Duration::from_millis(100)).await;
        }

        Err(BrowserError::NavigationFailed(
            "Page load timeout".to_string(),
        ))
    }
    pub async fn highlight_interactable_elements(&self) -> Result<Vec<ElementHighlight>> {
        // Get DOM state first
        let dom_state = self.get_dom_state_with_labels(false).await?;

        // Clear any existing overlays
        self.clear_element_highlights().await?;

        let mut highlights = Vec::new();
        let mut element_counter = 1;

        // Define colors for different element types
        let colors = [
            ("#FF0000", "clickable"), // Red for clickable elements
            ("#00FF00", "input"),     // Green for input elements
            ("#0000FF", "button"),    // Blue for buttons
            ("#FF6600", "select"),    // Orange for select elements
            ("#9900FF", "textarea"),  // Purple for text areas
            ("#00FFFF", "link"),      // Cyan for links
        ];

        // Process clickable elements
        for element in &dom_state.clickable_elements {
            let color = match element.tag_name.as_str() {
                "button" => "#0000FF",
                "input" => "#00FF00",
                "select" => "#FF6600",
                "textarea" => "#9900FF",
                "a" => "#00FFFF",
                _ => "#FF0000",
            };

            if let Ok(_) = self
                .draw_element_highlight(&element.css_selector, color, element_counter)
                .await
            {
                highlights.push(ElementHighlight {
                    element_id: element.id.clone(),
                    element_number: element_counter,
                    color: color.to_string(),
                    element_type: element.tag_name.clone(),
                });
                element_counter += 1;
            }
        }

        // Process input elements that might not be in clickable elements
        for element in &dom_state.input_elements {
            if !highlights.iter().any(|h| h.element_id == element.id) {
                let color = match element.tag_name.as_str() {
                    "input" => "#00FF00",
                    "textarea" => "#9900FF",
                    "select" => "#FF6600",
                    _ => "#FFFF00",
                };

                if let Ok(_) = self
                    .draw_element_highlight(&element.css_selector, color, element_counter)
                    .await
                {
                    highlights.push(ElementHighlight {
                        element_id: element.id.clone(),
                        element_number: element_counter,
                        color: color.to_string(),
                        element_type: element.tag_name.clone(),
                    });
                    element_counter += 1;
                }
            }
        }

        Ok(highlights)
    }

    async fn draw_element_highlight(
        &self,
        css_selector: &str,
        color: &str,
        number: usize,
    ) -> Result<()> {
        let js_code = format!(
            r#"
                (function() {{
                    const element = document.querySelector('{}');
                    if (!element) return false;

                    const rect = element.getBoundingClientRect();
                    if (rect.width === 0 || rect.height === 0) return false;

                    // Create overlay div
                    const overlay = document.createElement('div');
                    overlay.className = 'browser-automation-highlight-{}';
                    overlay.style.position = 'fixed';
                    overlay.style.left = rect.left + 'px';
                    overlay.style.top = rect.top + 'px';
                    overlay.style.width = rect.width + 'px';
                    overlay.style.height = rect.height + 'px';
                    overlay.style.border = '3px solid {}';
                    overlay.style.backgroundColor = 'transparent';
                    overlay.style.pointerEvents = 'none';
                    overlay.style.zIndex = '999999';
                    overlay.style.boxSizing = 'border-box';

                    // Create number label
                    const label = document.createElement('div');
                    label.style.position = 'absolute';
                    label.style.top = '-25px';
                    label.style.left = '-3px';
                    label.style.backgroundColor = '{}';
                    label.style.color = 'white';
                    label.style.padding = '2px 6px';
                    label.style.fontSize = '12px';
                    label.style.fontWeight = 'bold';
                    label.style.borderRadius = '3px';
                    label.style.fontFamily = 'Arial, sans-serif';
                    label.style.minWidth = '20px';
                    label.style.textAlign = 'center';
                    label.textContent = '{}';

                    overlay.appendChild(label);
                    document.body.appendChild(overlay);

                    return true;
                }})()
            "#,
            css_selector.replace("'", "\\'"),
            number,
            color,
            color,
            number
        );

        let result = self
            .tab
            .evaluate(&js_code, false)
            .map_err(|e| BrowserError::JavaScriptFailed(e.to_string()))?;

        if let Some(value) = result.value {
            if value.as_bool() == Some(true) {
                return Ok(());
            }
        }

        Err(BrowserError::ElementNotFound(format!(
            "Could not highlight element: {}",
            css_selector
        )))
    }

    pub async fn clear_element_highlights(&self) -> Result<()> {
        let js_code = r#"
                (function() {
                    const highlights = document.querySelectorAll('[class*="browser-automation-highlight-"]');
                    highlights.forEach(highlight => highlight.remove());
                    return highlights.length;
                })()
            "#;

        self.tab
            .evaluate(js_code, false)
            .map_err(|e| BrowserError::JavaScriptFailed(e.to_string()))?;

        Ok(())
    }

    pub async fn highlight_element_by_number(
        &self,
        element_number: usize,
        highlights: &[ElementHighlight],
    ) -> Result<()> {
        if let Some(highlight) = highlights
            .iter()
            .find(|h| h.element_number == element_number)
        {
            // Clear existing highlights
            self.clear_element_highlights().await?;

            // Highlight just this element with a special color
            let js_code = format!(
                r#"
                    (function() {{
                        // Find element by its highlight data
                        const elements = document.querySelectorAll('*');
                        for (let element of elements) {{
                            const rect = element.getBoundingClientRect();
                            if (rect.width > 0 && rect.height > 0) {{
                                // This is a simplified approach - in practice you'd want to match
                                // elements more precisely using the stored CSS selector

                                // Create pulsing highlight
                                const overlay = document.createElement('div');
                                overlay.style.position = 'fixed';
                                overlay.style.left = rect.left + 'px';
                                overlay.style.top = rect.top + 'px';
                                overlay.style.width = rect.width + 'px';
                                overlay.style.height = rect.height + 'px';
                                overlay.style.border = '5px solid #FFD700';
                                overlay.style.backgroundColor = 'rgba(255, 215, 0, 0.2)';
                                overlay.style.pointerEvents = 'none';
                                overlay.style.zIndex = '999999';
                                overlay.style.animation = 'pulse 1s infinite';
                                overlay.className = 'browser-automation-highlight-selected';

                                // Add pulse animation
                                const style = document.createElement('style');
                                style.textContent = `
                                    @keyframes pulse {{
                                        0% {{ opacity: 1; }}
                                        50% {{ opacity: 0.5; }}
                                        100% {{ opacity: 1; }}
                                    }}
                                `;
                                document.head.appendChild(style);

                                const label = document.createElement('div');
                                label.style.position = 'absolute';
                                label.style.top = '-30px';
                                label.style.left = '-5px';
                                label.style.backgroundColor = '#FFD700';
                                label.style.color = 'black';
                                label.style.padding = '4px 8px';
                                label.style.fontSize = '14px';
                                label.style.fontWeight = 'bold';
                                label.style.borderRadius = '5px';
                                label.textContent = 'SELECTED: {}';

                                overlay.appendChild(label);
                                document.body.appendChild(overlay);
                                break;
                            }}
                        }}
                        return true;
                    }})()
                "#,
                element_number
            );

            self.tab
                .evaluate(&js_code, false)
                .map_err(|e| BrowserError::JavaScriptFailed(e.to_string()))?;

            Ok(())
        } else {
            Err(BrowserError::ElementNotFound(format!(
                "Element number {} not found",
                element_number
            )))
        }
    }

    pub async fn click_element_by_number(
        &self,
        element_number: usize,
        highlights: &[ElementHighlight],
    ) -> Result<()> {
        if let Some(highlight) = highlights
            .iter()
            .find(|h| h.element_number == element_number)
        {
            // Find the actual DOM element and click it
            let dom_state = self.get_dom_state(false).await?;

            if let Some(element) = dom_state
                .elements
                .iter()
                .find(|e| e.id == highlight.element_id)
            {
                self.click_element(&element.css_selector).await
            } else {
                Err(BrowserError::ElementNotFound(format!(
                    "Element {} not found in DOM",
                    element_number
                )))
            }
        } else {
            Err(BrowserError::ElementNotFound(format!(
                "Element number {} not found",
                element_number
            )))
        }
    }

    pub async fn type_in_element_by_number(
        &self,
        element_number: usize,
        text: &str,
        highlights: &[ElementHighlight],
    ) -> Result<()> {
        if let Some(highlight) = highlights
            .iter()
            .find(|h| h.element_number == element_number)
        {
            let dom_state = self.get_dom_state(false).await?;

            if let Some(element) = dom_state
                .elements
                .iter()
                .find(|e| e.id == highlight.element_id)
            {
                self.type_text(&element.css_selector, text).await
            } else {
                Err(BrowserError::ElementNotFound(format!(
                    "Element {} not found in DOM",
                    element_number
                )))
            }
        } else {
            Err(BrowserError::ElementNotFound(format!(
                "Element number {} not found",
                element_number
            )))
        }
    }

    pub async fn get_element_info_by_number(
        &self,
        element_number: usize,
        highlights: &[ElementHighlight],
    ) -> Result<Option<crate::dom::DomElement>> {
        if let Some(highlight) = highlights
            .iter()
            .find(|h| h.element_number == element_number)
        {
            let dom_state = self.get_dom_state_with_labels(false).await?;

            Ok(dom_state
                .elements
                .iter()
                .find(|e| e.id == highlight.element_id)
                .cloned())
        } else {
            Ok(None)
        }
    }

    // Fast batch highlighting without delays
    pub async fn highlight_elements_batch(&self) -> Result<Vec<ElementHighlight>> {
        let dom_state = self.get_dom_state_with_labels(false).await?;
        self.clear_element_highlights().await?;

        // Build all highlights in a single JavaScript execution
        let mut js_commands: Vec<String> = Vec::new();
        let mut highlights = Vec::new();
        let mut element_counter = 1;

        // Collect all elements to highlight
        let mut elements_to_highlight = Vec::new();

        // Add clickable elements
        for element in &dom_state.clickable_elements {
            let color = match element.tag_name.as_str() {
                "button" => "#0000FF",
                "input" => "#00FF00",
                "select" => "#FF6600",
                "textarea" => "#9900FF",
                "a" => "#00FFFF",
                _ => "#FF0000",
            };

            elements_to_highlight.push((element, color, element_counter));
            highlights.push(ElementHighlight {
                element_id: element.id.clone(),
                element_number: element_counter,
                color: color.to_string(),
                element_type: element.tag_name.clone(),
            });
            element_counter += 1;
        }

        // Add unique input elements
        for element in &dom_state.input_elements {
            if !highlights.iter().any(|h| h.element_id == element.id) {
                let color = match element.tag_name.as_str() {
                    "input" => "#00FF00",
                    "textarea" => "#9900FF",
                    "select" => "#FF6600",
                    _ => "#FFFF00",
                };

                elements_to_highlight.push((element, color, element_counter));
                highlights.push(ElementHighlight {
                    element_id: element.id.clone(),
                    element_number: element_counter,
                    color: color.to_string(),
                    element_type: element.tag_name.clone(),
                });
                element_counter += 1;
            }
        }

        // Build single JavaScript command for all highlights
        let mut batch_js = String::from("(function() { const results = [];");

        for (element, color, number) in elements_to_highlight {
            batch_js.push_str(&format!(
                r#"
                    try {{
                        const element = document.querySelector('{}');
                        if (element) {{
                            const rect = element.getBoundingClientRect();
                            if (rect.width > 0 && rect.height > 0) {{
                                const overlay = document.createElement('div');
                                overlay.className = 'browser-automation-highlight-{}';
                                overlay.style.position = 'fixed';
                                overlay.style.left = rect.left + 'px';
                                overlay.style.top = rect.top + 'px';
                                overlay.style.width = rect.width + 'px';
                                overlay.style.height = rect.height + 'px';
                                overlay.style.border = '3px solid {}';
                                overlay.style.backgroundColor = 'transparent';
                                overlay.style.pointerEvents = 'none';
                                overlay.style.zIndex = '999999';
                                overlay.style.boxSizing = 'border-box';

                                const label = document.createElement('div');
                                label.style.position = 'absolute';
                                label.style.top = '-25px';
                                label.style.left = '-3px';
                                label.style.backgroundColor = '{}';
                                label.style.color = 'white';
                                label.style.padding = '2px 6px';
                                label.style.fontSize = '12px';
                                label.style.fontWeight = 'bold';
                                label.style.borderRadius = '3px';
                                label.style.fontFamily = 'Arial, sans-serif';
                                label.style.minWidth = '20px';
                                label.style.textAlign = 'center';
                                label.textContent = '{}';

                                overlay.appendChild(label);
                                document.body.appendChild(overlay);
                                results.push({});
                            }}
                        }}
                    }} catch(e) {{ console.error('Highlight error:', e); }}
                "#,
                element.css_selector.replace("'", "\\'"),
                number,
                color,
                color,
                number,
                number
            ));
        }

        batch_js.push_str(" return results.length; })()");

        // Execute all highlights in one batch
        self.tab
            .evaluate(&batch_js, false)
            .map_err(|e| BrowserError::JavaScriptFailed(e.to_string()))?;

        Ok(highlights)
    }
}

impl Drop for BrowserSession {
    fn drop(&mut self) {
        // Browser will be automatically closed when dropped
    }
}
