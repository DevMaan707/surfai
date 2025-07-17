use crate::error::{BrowserError, BrowserResult};
use chromiumoxide::Page;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InteractableElement {
    pub tag_name: String,
    pub element_type: String,
    pub text: Option<String>,
    pub selector: String,
    pub clickable: bool,
    pub input_type: Option<String>,
    pub attributes: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LabeledElement {
    pub unique_id: String,
    pub tag_name: String,
    pub text: Option<String>,
    pub selector: String,
    pub clickable: bool,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllElement {
    pub unique_id: String,
    pub tag_name: String,
    pub text: Option<String>,
    pub selector: String,
}

pub struct ElementInteractor {
    page: Page,
}

impl ElementInteractor {
    pub fn new(page: Page) -> Self {
        Self { page }
    }

    /// Get all elements on the page
    pub async fn get_all_elements(&self) -> BrowserResult<Vec<AllElement>> {
        let script = r#"
            Array.from(document.querySelectorAll('*')).map((el, index) => ({
                unique_id: 'elem_' + index,
                tag_name: el.tagName.toLowerCase(),
                text: el.textContent ? el.textContent.trim().substring(0, 50) : null,
                selector: el.id ? '#' + el.id : el.tagName.toLowerCase()
            })).filter(el => el.tag_name !== 'script' && el.tag_name !== 'style');
        "#;

        let result = self
            .page
            .evaluate(script)
            .await
            .map_err(|e| BrowserError::Script(e.to_string()))?;

        let elements: Vec<AllElement> = result
            .into_value()
            .map_err(|e| BrowserError::Script(e.to_string()))?;

        Ok(elements)
    }

    /// Get interactable elements with positions - FIXED for Google search box
    pub async fn get_interactable_elements(&self) -> BrowserResult<Vec<LabeledElement>> {
        let script = r#"
            (() => {
                const elements = [];
                // Include textarea, input, button, a, select elements
                const allElements = document.querySelectorAll('input, textarea, button, a, select');

                allElements.forEach((el, index) => {
                    const rect = el.getBoundingClientRect();

                    // Only include visible elements
                    if (rect.width > 0 && rect.height > 0) {
                        const tagName = el.tagName.toLowerCase();
                        const isClickable = tagName === 'button' || tagName === 'a' ||
                                          el.type === 'submit' || el.type === 'button';

                        // Better selector generation
                        let selector = '';
                        if (el.name) {
                            selector = `${tagName}[name="${el.name}"]`;
                        } else if (el.id) {
                            selector = `#${el.id}`;
                        } else if (el.className) {
                            selector = `.${el.className.split(' ')[0]}`;
                        } else {
                            selector = tagName;
                        }

                        elements.push({
                            unique_id: 'interact_' + index,
                            tag_name: tagName,
                            text: el.textContent ? el.textContent.trim().substring(0, 50) :
                                  (el.placeholder ? el.placeholder.substring(0, 50) : null),
                            selector: selector,
                            clickable: isClickable,
                            x: rect.x,
                            y: rect.y,
                            width: rect.width,
                            height: rect.height
                        });
                    }
                });

                return elements;
            })();
        "#;

        let result = self
            .page
            .evaluate(script)
            .await
            .map_err(|e| BrowserError::Script(e.to_string()))?;

        let elements: Vec<LabeledElement> = result
            .into_value()
            .map_err(|e| BrowserError::Script(e.to_string()))?;

        Ok(elements)
    }

    /// Label elements by tag - FIXED for Google search box
    pub async fn label_elements(
        &self,
        tags: Option<Vec<String>>,
    ) -> BrowserResult<Vec<LabeledElement>> {
        let selector = if let Some(tag_list) = tags {
            tag_list.join(", ")
        } else {
            "input, textarea, button, a, select".to_string()
        };

        let script = format!(
            r#"
            (() => {{
                const elements = [];
                const allElements = document.querySelectorAll('{}');

                allElements.forEach((el, index) => {{
                    const rect = el.getBoundingClientRect();

                    // Only include visible elements
                    if (rect.width > 0 && rect.height > 0) {{
                        const tagName = el.tagName.toLowerCase();
                        const uniqueId = tagName + '_' + index;

                        // Set data attribute for later reference
                        el.setAttribute('data-surf-id', uniqueId);

                        const isClickable = tagName === 'button' || tagName === 'a' ||
                                          el.type === 'submit' || el.type === 'button';

                        // Better selector generation
                        let selector = '';
                        if (el.name) {{
                            selector = `${{tagName}}[name="${{el.name}}"]`;
                        }} else if (el.id) {{
                            selector = `#${{el.id}}`;
                        }} else if (el.className) {{
                            selector = `.${{el.className.split(' ')[0]}}`;
                        }} else {{
                            selector = tagName;
                        }}

                        elements.push({{
                            unique_id: uniqueId,
                            tag_name: tagName,
                            text: el.textContent ? el.textContent.trim().substring(0, 50) :
                                  (el.placeholder ? el.placeholder.substring(0, 50) : null),
                            selector: selector,
                            clickable: isClickable,
                            x: rect.x,
                            y: rect.y,
                            width: rect.width,
                            height: rect.height
                        }});
                    }}
                }});

                return elements;
            }})();
        "#,
            selector
        );

        let result = self
            .page
            .evaluate(script.as_str())
            .await
            .map_err(|e| BrowserError::Script(e.to_string()))?;

        let elements: Vec<LabeledElement> = result
            .into_value()
            .map_err(|e| BrowserError::Script(e.to_string()))?;

        Ok(elements)
    }

    /// Type text into search box (Google-specific helper)
    pub async fn type_in_search_box(&self, text: &str) -> BrowserResult<()> {
        let script = format!(
            r#"
            (() => {{
                // Try multiple selectors for Google search box
                const selectors = [
                    'textarea[name="q"]',
                    'input[name="q"]',
                    'input[aria-label="Search"]',
                    'textarea[aria-label="Search"]',
                    'input[title="Search"]',
                    'textarea[title="Search"]'
                ];

                for (let selector of selectors) {{
                    const element = document.querySelector(selector);
                    if (element) {{
                        element.focus();
                        element.value = '{}';
                        element.dispatchEvent(new Event('input', {{ bubbles: true }}));
                        return true;
                    }}
                }}
                return false;
            }})();
        "#,
            text
        );

        let result = self
            .page
            .evaluate(script.as_str())
            .await
            .map_err(|e| BrowserError::Script(e.to_string()))?;

        let success: bool = result
            .into_value()
            .map_err(|e| BrowserError::Script(e.to_string()))?;

        if success {
            Ok(())
        } else {
            Err(BrowserError::ElementNotFound(
                "Google search box".to_string(),
            ))
        }
    }

    /// Click search button (Google-specific helper)
    pub async fn click_search_button(&self) -> BrowserResult<()> {
        let script = r#"
            (() => {
                // Try multiple selectors for Google search button
                const selectors = [
                    'button[aria-label="Google Search"]',
                    'input[name="btnK"]',
                    'input[value="Google Search"]',
                    'button[type="submit"]',
                    'input[type="submit"]'
                ];

                for (let selector of selectors) {
                    const element = document.querySelector(selector);
                    if (element) {
                        element.click();
                        return true;
                    }
                }

                // Fallback: press Enter on search box
                const searchBox = document.querySelector('textarea[name="q"]') ||
                                document.querySelector('input[name="q"]');
                if (searchBox) {
                    searchBox.dispatchEvent(new KeyboardEvent('keydown', { key: 'Enter' }));
                    return true;
                }

                return false;
            })();
        "#;

        let result = self
            .page
            .evaluate(script)
            .await
            .map_err(|e| BrowserError::Script(e.to_string()))?;

        let success: bool = result
            .into_value()
            .map_err(|e| BrowserError::Script(e.to_string()))?;

        if success {
            Ok(())
        } else {
            Err(BrowserError::ElementNotFound(
                "Google search button".to_string(),
            ))
        }
    }

    /// Extract search results from Google results page
    pub async fn extract_search_results(&self) -> BrowserResult<Vec<HashMap<String, String>>> {
        let script = r#"
            (() => {
                const results = [];

                // Google search results are typically in divs with class 'g'
                const resultElements = document.querySelectorAll('.g, .MjjYud');

                resultElements.forEach(element => {
                    const titleElement = element.querySelector('h3');
                    const linkElement = element.querySelector('a');
                    const snippetElement = element.querySelector('.VwiC3b, .s3v9rd');

                    if (titleElement && linkElement) {
                        results.push({
                            title: titleElement.textContent || '',
                            link: linkElement.href || '',
                            snippet: snippetElement ? snippetElement.textContent || '' : ''
                        });
                    }
                });

                return results;
            })();
        "#;

        let result = self
            .page
            .evaluate(script)
            .await
            .map_err(|e| BrowserError::Script(e.to_string()))?;

        let results: Vec<HashMap<String, String>> = result
            .into_value()
            .map_err(|e| BrowserError::Script(e.to_string()))?;

        Ok(results)
    }

    // Keep existing methods...
    pub async fn label_ui(
        &self,
        elements: &[LabeledElement],
        interactable_only: bool,
    ) -> BrowserResult<()> {
        let elements_json = serde_json::to_string(elements)?;

        let script = format!(
            r#"
            (() => {{
                const elements = {};
                const interactableOnly = {};

                // Remove existing labels
                document.querySelectorAll('.surf-label, .surf-box').forEach(el => el.remove());

                // Add styles if not exists
                if (!document.getElementById('surf-styles')) {{
                    const style = document.createElement('style');
                    style.id = 'surf-styles';
                    style.textContent = `
                        .surf-box {{ position: fixed; border: 2px solid red; background: rgba(255,0,0,0.1); z-index: 9999; pointer-events: none; }}
                        .surf-box.clickable {{ border-color: green; background: rgba(0,255,0,0.1); }}
                        .surf-label {{ position: fixed; background: red; color: white; padding: 2px 4px; font-size: 10px; z-index: 10000; pointer-events: none; }}
                        .surf-label.clickable {{ background: green; }}
                    `;
                    document.head.appendChild(style);
                }}

                // Create all elements in one go
                elements.forEach((element, index) => {{
                    if (interactableOnly && !element.clickable) return;

                    const boxEl = document.createElement('div');
                    boxEl.className = 'surf-box' + (element.clickable ? ' clickable' : '');
                    boxEl.style.left = element.x + 'px';
                    boxEl.style.top = element.y + 'px';
                    boxEl.style.width = element.width + 'px';
                    boxEl.style.height = element.height + 'px';
                    document.body.appendChild(boxEl);

                    const labelEl = document.createElement('div');
                    labelEl.className = 'surf-label' + (element.clickable ? ' clickable' : '');
                    labelEl.textContent = element.unique_id;
                    labelEl.style.left = element.x + 'px';
                    labelEl.style.top = (element.y - 15) + 'px';
                    document.body.appendChild(labelEl);
                }});

                return elements.length;
            }})();
        "#,
            elements_json, interactable_only
        );

        let result = self
            .page
            .evaluate(script.as_str())
            .await
            .map_err(|e| BrowserError::Script(e.to_string()))?;

        let _count: i32 = result
            .into_value()
            .map_err(|e| BrowserError::Script(e.to_string()))?;

        Ok(())
    }

    pub async fn clear_label_ui(&self) -> BrowserResult<()> {
        let script = r#"
            (() => {
                document.querySelectorAll('.surf-label, .surf-box').forEach(el => el.remove());
                const style = document.getElementById('surf-styles');
                if (style) style.remove();
                return true;
            })();
        "#;

        let result = self
            .page
            .evaluate(script)
            .await
            .map_err(|e| BrowserError::Script(e.to_string()))?;

        let _success: bool = result
            .into_value()
            .map_err(|e| BrowserError::Script(e.to_string()))?;

        Ok(())
    }

    pub async fn click_by_id(&self, unique_id: &str) -> BrowserResult<()> {
        let script = format!(
            r#"
            (() => {{
                const el = document.querySelector('[data-surf-id="{}"]');
                if (el) {{
                    el.click();
                    return true;
                }}
                return false;
            }})();
        "#,
            unique_id
        );

        let result = self
            .page
            .evaluate(script.as_str())
            .await
            .map_err(|e| BrowserError::Script(e.to_string()))?;

        let success: bool = result
            .into_value()
            .map_err(|e| BrowserError::Script(e.to_string()))?;

        if success {
            Ok(())
        } else {
            Err(BrowserError::ElementNotFound(unique_id.to_string()))
        }
    }

    pub async fn find_interactable_elements(&self) -> BrowserResult<Vec<InteractableElement>> {
        let script = r#"
            (() => {
                const elements = [];
                const allElements = document.querySelectorAll('*');

                allElements.forEach(el => {
                    const tagName = el.tagName.toLowerCase();
                    const isInteractable = ['input', 'button', 'select', 'textarea', 'a'].includes(tagName) ||
                                         el.onclick || el.getAttribute('role') === 'button' ||
                                         el.classList.contains('btn') || el.classList.contains('button');

                    if (isInteractable && el.offsetWidth > 0 && el.offsetHeight > 0) {
                        const attributes = {};
                        for (let attr of el.attributes) {
                            attributes[attr.name] = attr.value;
                        }

                        elements.push({
                            tag_name: tagName,
                            element_type: el.type || el.getAttribute('type') || 'unknown',
                            text: el.textContent ? el.textContent.trim().substring(0, 100) : null,
                            selector: el.id ? `#${el.id}` : (el.className ? `.${el.className.split(' ')[0]}` : tagName),
                            clickable: tagName === 'button' || tagName === 'a' || !!el.onclick,
                            input_type: tagName === 'input' ? el.type : null,
                            attributes: attributes
                        });
                    }
                });

                return elements;
            })();
        "#;

        let result = self
            .page
            .evaluate(script)
            .await
            .map_err(|e| BrowserError::Script(e.to_string()))?;

        let elements: Vec<InteractableElement> = result
            .into_value()
            .map_err(|e| BrowserError::Script(e.to_string()))?;

        Ok(elements)
    }

    pub async fn click_element(&self, selector: &str) -> BrowserResult<()> {
        let element = self
            .page
            .find_element(selector)
            .await
            .map_err(|_| BrowserError::ElementNotFound(selector.to_string()))?;

        element
            .click()
            .await
            .map_err(|e| BrowserError::Script(e.to_string()))?;

        Ok(())
    }

    pub async fn type_text(&self, selector: &str, text: &str) -> BrowserResult<()> {
        let element = self
            .page
            .find_element(selector)
            .await
            .map_err(|_| BrowserError::ElementNotFound(selector.to_string()))?;

        element
            .type_str(text)
            .await
            .map_err(|e| BrowserError::Script(e.to_string()))?;

        Ok(())
    }

    pub async fn get_element_text(&self, selector: &str) -> BrowserResult<String> {
        let element = self
            .page
            .find_element(selector)
            .await
            .map_err(|_| BrowserError::ElementNotFound(selector.to_string()))?;

        let text = element
            .inner_text()
            .await
            .map_err(|e| BrowserError::Script(e.to_string()))?;

        Ok(text.unwrap_or_default())
    }

    pub async fn get_element_attribute(
        &self,
        selector: &str,
        attribute: &str,
    ) -> BrowserResult<Option<String>> {
        let element = self
            .page
            .find_element(selector)
            .await
            .map_err(|_| BrowserError::ElementNotFound(selector.to_string()))?;

        let attr = element
            .attribute(attribute)
            .await
            .map_err(|e| BrowserError::Script(e.to_string()))?;

        Ok(attr)
    }
}
