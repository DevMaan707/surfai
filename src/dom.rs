use crate::errors::{BrowserError, Result};
use crate::types::ElementRect;
use headless_chrome::Tab;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomElement {
    pub id: String,
    pub tag_name: String,
    pub element_id: Option<String>,
    pub class_name: Option<String>,
    pub text_content: Option<String>,
    pub attributes: HashMap<String, String>,
    pub rect: Option<ElementRect>,
    pub is_clickable: bool,
    pub is_visible: bool,
    pub is_interactable: bool,
    pub xpath: String,
    pub css_selector: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomState {
    pub url: String,
    pub title: String,
    pub elements: Vec<DomElement>,
    pub clickable_elements: Vec<DomElement>,
    pub input_elements: Vec<DomElement>,
    pub text_elements: Vec<DomElement>,
    pub screenshot_base64: Option<String>,
}

pub struct DomProcessor {
    tab: Arc<Tab>,
}

impl DomProcessor {
    pub fn new(tab: Arc<Tab>) -> Self {
        Self { tab }
    }

    pub async fn extract_dom_state(&self, include_screenshot: bool) -> Result<DomState> {
        let url = self.get_current_url().await?;
        let title = self.get_page_title().await?;

        // Get HTML content
        let html_content = self.get_html_content().await?;

        // Parse and extract elements
        let elements = self.extract_elements(&html_content).await?;

        // Categorize elements
        let clickable_elements = self.filter_clickable_elements(&elements);
        let input_elements = self.filter_input_elements(&elements);
        let text_elements = self.filter_text_elements(&elements);

        let screenshot_base64 = if include_screenshot {
            Some(self.take_screenshot().await?)
        } else {
            None
        };

        Ok(DomState {
            url,
            title,
            elements,
            clickable_elements,
            input_elements,
            text_elements,
            screenshot_base64,
        })
    }

    async fn get_current_url(&self) -> Result<String> {
        Ok(self.tab.get_url())
    }

    async fn get_page_title(&self) -> Result<String> {
        let js_result = self
            .tab
            .evaluate("document.title", false)
            .map_err(|e| BrowserError::JavaScriptFailed(e.to_string()))?;

        Ok(js_result
            .value
            .and_then(|v| v.as_str().map(|s| s.to_string()))
            .unwrap_or_default())
    }

    async fn get_html_content(&self) -> Result<String> {
        let js_result = self
            .tab
            .evaluate("document.documentElement.outerHTML", false)
            .map_err(|e| BrowserError::JavaScriptFailed(e.to_string()))?;

        js_result
            .value
            .and_then(|v| v.as_str().map(|s| s.to_string()))
            .ok_or_else(|| {
                BrowserError::DomExtractionFailed("Failed to get HTML content".to_string())
            })
    }

    async fn extract_elements(&self, html_content: &str) -> Result<Vec<DomElement>> {
        let document = Html::parse_document(html_content);
        let mut elements = Vec::new();
        let mut element_counter = 0;

        // Define selectors for different element types
        let interactive_selectors = [
            "a",
            "button",
            "input",
            "select",
            "textarea",
            "label",
            "[onclick]",
            "[role='button']",
            "[tabindex]",
            "summary",
            "details",
        ];

        for selector_str in &interactive_selectors {
            if let Ok(selector) = Selector::parse(selector_str) {
                for element_ref in document.select(&selector) {
                    let element = element_ref.value();
                    let mut attributes = HashMap::new();

                    // Extract attributes using the correct API
                    for (name, value) in element.attrs() {
                        attributes.insert(name.to_string(), value.to_string());
                    }

                    // Extract text content
                    let text_content = element_ref.text().collect::<Vec<_>>().join(" ");
                    let text_content = if text_content.trim().is_empty() {
                        None
                    } else {
                        Some(text_content.trim().to_string())
                    };

                    // Generate unique ID for the element
                    element_counter += 1;
                    let id = format!("elem_{}", element_counter);

                    // Generate XPath and CSS selector
                    let xpath = self.generate_xpath(element.name(), element_counter);
                    let css_selector = self.generate_css_selector(element.name(), &attributes);

                    // Get element rectangle (would need JavaScript evaluation for accurate positioning)
                    let rect = self.get_element_rect(&css_selector).await.ok();

                    let dom_element = DomElement {
                        id,
                        tag_name: element.name().to_string(),
                        element_id: attributes.get("id").cloned(),
                        class_name: attributes.get("class").cloned(),
                        text_content,
                        attributes,
                        rect,
                        is_clickable: self.is_clickable_element(element.name()),
                        is_visible: true, // Would need JavaScript evaluation for accurate visibility
                        is_interactable: self.is_interactable_element(element.name()),
                        xpath,
                        css_selector,
                    };

                    elements.push(dom_element);
                }
            }
        }

        // Also extract text elements (p, h1-h6, span, div with text)
        let text_selectors = ["p", "h1", "h2", "h3", "h4", "h5", "h6", "span", "div"];
        for selector_str in &text_selectors {
            if let Ok(selector) = Selector::parse(selector_str) {
                for element_ref in document.select(&selector) {
                    let element = element_ref.value();
                    let text_content = element_ref.text().collect::<Vec<_>>().join(" ");

                    if !text_content.trim().is_empty() && text_content.trim().len() > 5 {
                        let mut attributes = HashMap::new();
                        for (name, value) in element.attrs() {
                            attributes.insert(name.to_string(), value.to_string());
                        }

                        element_counter += 1;
                        let id = format!("elem_{}", element_counter);
                        let xpath = self.generate_xpath(element.name(), element_counter);
                        let css_selector = self.generate_css_selector(element.name(), &attributes);

                        let dom_element = DomElement {
                            id,
                            tag_name: element.name().to_string(),
                            element_id: attributes.get("id").cloned(),
                            class_name: attributes.get("class").cloned(),
                            text_content: Some(text_content.trim().to_string()),
                            attributes,
                            rect: None,
                            is_clickable: false,
                            is_visible: true,
                            is_interactable: false,
                            xpath,
                            css_selector,
                        };

                        elements.push(dom_element);
                    }
                }
            }
        }

        Ok(elements)
    }

    async fn get_element_rect(&self, css_selector: &str) -> Result<ElementRect> {
        let js_code = format!(
            r#"
            (function() {{
                const element = document.querySelector('{}');
                if (!element) return null;
                const rect = element.getBoundingClientRect();
                return {{
                    x: rect.x,
                    y: rect.y,
                    width: rect.width,
                    height: rect.height
                }};
            }})()
        "#,
            css_selector.replace("'", "\\'")
        );

        let js_result = self
            .tab
            .evaluate(&js_code, false)
            .map_err(|e| BrowserError::JavaScriptFailed(e.to_string()))?;

        if let Some(value) = js_result.value {
            if let Ok(rect) = serde_json::from_value::<ElementRect>(value) {
                return Ok(rect);
            }
        }

        Err(BrowserError::ElementNotFound(
            "Element rect not found".to_string(),
        ))
    }

    fn generate_xpath(&self, tag_name: &str, counter: usize) -> String {
        format!("//{}[{}]", tag_name, counter)
    }

    fn generate_css_selector(
        &self,
        tag_name: &str,
        attributes: &HashMap<String, String>,
    ) -> String {
        let mut selector = tag_name.to_string();

        if let Some(id) = attributes.get("id") {
            selector.push_str(&format!("#{}", id));
        } else if let Some(class) = attributes.get("class") {
            let classes = class.split_whitespace().collect::<Vec<_>>();
            if !classes.is_empty() {
                selector.push_str(&format!(".{}", classes.join(".")));
            }
        }

        selector
    }

    fn is_clickable_element(&self, tag_name: &str) -> bool {
        matches!(tag_name, "a" | "button" | "input" | "select" | "summary")
    }

    fn is_interactable_element(&self, tag_name: &str) -> bool {
        matches!(tag_name, "input" | "textarea" | "select" | "button" | "a")
    }

    fn filter_clickable_elements(&self, elements: &[DomElement]) -> Vec<DomElement> {
        elements
            .iter()
            .filter(|elem| elem.is_clickable)
            .cloned()
            .collect()
    }

    fn filter_input_elements(&self, elements: &[DomElement]) -> Vec<DomElement> {
        elements
            .iter()
            .filter(|elem| matches!(elem.tag_name.as_str(), "input" | "textarea" | "select"))
            .cloned()
            .collect()
    }

    fn filter_text_elements(&self, elements: &[DomElement]) -> Vec<DomElement> {
        elements
            .iter()
            .filter(|elem| {
                elem.text_content.is_some() && !elem.text_content.as_ref().unwrap().is_empty()
            })
            .cloned()
            .collect()
    }

    async fn take_screenshot(&self) -> Result<String> {
        let screenshot = self
            .tab
            .capture_screenshot(
                headless_chrome::protocol::cdp::Page::CaptureScreenshotFormatOption::Png,
                None,
                None,
                true,
            )
            .map_err(|e| BrowserError::JavaScriptFailed(e.to_string()))?;

        Ok(base64::encode::<_>(screenshot))
    }

    pub async fn label_elements(&self, elements: &mut [DomElement]) -> Result<()> {
        for element in elements.iter_mut() {
            element
                .attributes
                .insert("ai_label".to_string(), self.generate_ai_label(element));
        }
        Ok(())
    }

    fn generate_ai_label(&self, element: &DomElement) -> String {
        let mut label_parts = vec![];
        label_parts.push(format!("{} element", element.tag_name));

        if let Some(text) = &element.text_content {
            if !text.is_empty() {
                label_parts.push(format!("with text '{}'", text.trim()));
            }
        }
        if let Some(id) = &element.element_id {
            label_parts.push(format!("with id '{}'", id));
        }
        if let Some(class) = &element.class_name {
            label_parts.push(format!("with class '{}'", class));
        }
        if element.is_clickable {
            label_parts.push("(clickable)".to_string());
        }

        if matches!(element.tag_name.as_str(), "input" | "textarea") {
            if let Some(input_type) = element.attributes.get("type") {
                label_parts.push(format!("(input type: {})", input_type));
            }
        }

        label_parts.join(" ")
    }
}
