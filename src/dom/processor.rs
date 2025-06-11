use crate::core::config::DomConfig;
use crate::core::{BrowserTrait, DomProcessorTrait, ElementFilter, SelectorType};
use crate::dom::{DomElement, DomState};
use crate::errors::Result;
use async_trait::async_trait;
use scraper::{ElementRef, Html, Selector};
use std::collections::HashMap;

pub struct DomProcessor {
    config: DomConfig,
}

impl DomProcessor {
    pub fn new(config: DomConfig) -> Self {
        Self { config }
    }
}

#[async_trait]
impl DomProcessorTrait for DomProcessor {
    async fn extract_dom_state<B: BrowserTrait>(
        &self,
        browser: &B,
        tab: &B::TabHandle,
        include_screenshot: bool,
    ) -> Result<DomState> {
        let url = browser.get_url(tab).await?;
        let title = browser.get_title(tab).await?;

        // Get HTML content
        let html_content = browser
            .execute_script(tab, "document.documentElement.outerHTML")
            .await?;
        let html_str = html_content.as_str().unwrap_or("");

        let mut dom_state = DomState::new(url, title);

        // Extract elements using multiple methods
        let mut elements = self.extract_all_interactive_elements(html_str).await?;

        // Add AI labels if enabled
        if self.config.enable_ai_labels {
            self.add_ai_labels(&mut elements).await?;
        }

        // Add elements to state
        for element in elements {
            dom_state.add_element(element);
        }

        if include_screenshot {
            let screenshot_bytes = browser.take_screenshot(tab).await?;
            let screenshot_base64 = base64::encode(screenshot_bytes);
            dom_state.set_screenshot(screenshot_base64);
        }

        Ok(dom_state)
    }

    async fn extract_interactive_elements<B: BrowserTrait>(
        &self,
        browser: &B,
        tab: &B::TabHandle,
    ) -> Result<Vec<DomElement>> {
        let dom_state = self.extract_dom_state(browser, tab, false).await?;
        Ok(dom_state.clickable_elements)
    }

    async fn add_ai_labels(&self, elements: &mut Vec<DomElement>) -> Result<()> {
        for element in elements.iter_mut() {
            element.generate_ai_label();
        }
        Ok(())
    }

    fn filter_elements(
        &self,
        elements: &[DomElement],
        criteria: &ElementFilter,
    ) -> Vec<DomElement> {
        elements
            .iter()
            .filter(|element| {
                if let Some(ref tag_names) = criteria.tag_names {
                    if !tag_names.contains(&element.tag_name) {
                        return false;
                    }
                }

                if let Some(ref text) = criteria.has_text {
                    if let Some(ref element_text) = element.text_content {
                        if !element_text.to_lowercase().contains(&text.to_lowercase()) {
                            return false;
                        }
                    } else {
                        return false;
                    }
                }

                if let Some(visible) = criteria.is_visible {
                    if element.is_visible != visible {
                        return false;
                    }
                }

                if let Some(interactive) = criteria.is_interactive {
                    if element.is_interactable != interactive {
                        return false;
                    }
                }

                if let Some((ref attr_name, ref attr_value)) = criteria.has_attribute {
                    if let Some(element_attr_value) = element.attributes.get(attr_name) {
                        if let Some(ref expected_value) = attr_value {
                            if element_attr_value != expected_value {
                                return false;
                            }
                        }
                    } else {
                        return false;
                    }
                }

                true
            })
            .cloned()
            .collect()
    }

    fn generate_selector(&self, element: &DomElement, selector_type: SelectorType) -> String {
        match selector_type {
            SelectorType::Css => element.css_selector.clone(),
            SelectorType::XPath => element.xpath.clone(),
            SelectorType::TestId => {
                if let Some(test_id) = element.attributes.get("data-testid") {
                    format!("[data-testid='{}']", test_id)
                } else {
                    element.css_selector.clone()
                }
            }
        }
    }
}

impl DomProcessor {
    async fn extract_all_interactive_elements(&self, html: &str) -> Result<Vec<DomElement>> {
        let document = Html::parse_document(html);
        let mut elements = Vec::new();
        let mut element_counter = 0;
        let mut processed_elements = std::collections::HashSet::new();

        // Comprehensive list of interactive element selectors
        let interactive_selectors = vec![
            // Standard form elements
            "input",
            "button",
            "select",
            "textarea",
            "label",
            "fieldset",
            "legend",
            "optgroup",
            "option",
            "datalist",
            // Links and navigation
            "a",
            "area",
            // Interactive content
            "details",
            "summary",
            "dialog",
            "menu",
            "menuitem",
            // Media controls
            "audio[controls]",
            "video[controls]",
            // Custom interactive elements
            "[onclick]",
            "[onchange]",
            "[onsubmit]",
            "[onkeydown]",
            "[onkeyup]",
            "[onfocus]",
            "[onblur]",
            // ARIA roles
            "[role='button']",
            "[role='link']",
            "[role='checkbox']",
            "[role='radio']",
            "[role='textbox']",
            "[role='searchbox']",
            "[role='combobox']",
            "[role='listbox']",
            "[role='tab']",
            "[role='tabpanel']",
            "[role='menuitem']",
            "[role='menubar']",
            "[role='menu']",
            "[role='dialog']",
            "[role='alertdialog']",
            "[role='tooltip']",
            "[role='slider']",
            "[role='spinbutton']",
            "[role='progressbar']",
            "[role='switch']",
            "[role='tree']",
            "[role='grid']",
            "[role='gridcell']",
            // Accessibility attributes
            "[tabindex]",
            "[aria-expanded]",
            "[aria-haspopup]",
            "[aria-controls]",
            "[aria-owns]",
            "[draggable='true']",
            "[contenteditable='true']",
            // Google-specific and common website patterns
            "[data-ved]",
            "[jsaction]",
            "[data-testid]",
            "[data-cy]",
            "[data-test]",
            "[data-automation]",
            "[id*='search']",
            "[name*='search']",
            "[class*='search']",
            "[placeholder*='search']",
            "[aria-label*='search']",
            "[title*='search']",
            // Common interactive classes
            ".btn",
            ".button",
            ".link",
            ".clickable",
            ".interactive",
            ".control",
            ".input",
            ".field",
            ".search",
            // Elements that might contain clickable children
            "[data-href]",
            "[data-url]",
            "[data-link]",
        ];

        // Process each selector
        for selector_str in &interactive_selectors {
            if let Ok(selector) = Selector::parse(selector_str) {
                for element_ref in document.select(&selector) {
                    let element = element_ref.value();

                    // Create a unique identifier for this element to avoid duplicates
                    let element_id = format!(
                        "{}_{}",
                        element.name(),
                        element
                            .attrs()
                            .map(|(k, v)| format!("{}={}", k, v))
                            .collect::<Vec<_>>()
                            .join("_")
                    );

                    if processed_elements.contains(&element_id) {
                        continue;
                    }
                    processed_elements.insert(element_id);

                    let mut attributes = HashMap::new();
                    for (name, value) in element.attrs() {
                        attributes.insert(name.to_string(), value.to_string());
                    }

                    // Get text content (both direct text and inner text)
                    let text_content = element_ref.text().collect::<Vec<_>>().join(" ");
                    let text_content = if text_content.trim().is_empty() {
                        None
                    } else {
                        Some(text_content.trim().to_string())
                    };

                    element_counter += 1;
                    let id = format!("elem_{}", element_counter);

                    let mut dom_element = DomElement::new(element.name().to_string(), id);

                    if let Some(text) = text_content {
                        dom_element = dom_element.with_text_content(text);
                    }

                    // Set all attributes
                    for (key, value) in &attributes {
                        if key == "id" {
                            dom_element.element_id = Some(value.clone());
                        } else if key == "class" {
                            dom_element.class_name = Some(value.clone());
                        }
                        dom_element = dom_element.with_attribute(key.clone(), value.clone());
                    }

                    // Generate comprehensive selectors
                    dom_element.xpath = self.generate_xpath_for_element(&element_ref, &attributes);
                    dom_element.css_selector =
                        self.generate_css_selector_for_element(&element_ref, &attributes);

                    // Determine interaction capabilities
                    dom_element = dom_element
                        .set_clickable(self.is_clickable_element(&element_ref))
                        .set_interactable(self.is_interactable_element(&element_ref));

                    // Set visibility (basic check)
                    dom_element.is_visible = !self.is_hidden_element(&attributes);

                    elements.push(dom_element);
                }
            }
        }

        // Also extract text elements if configured
        if self.config.extract_all_elements {
            let text_selectors = [
                "p", "h1", "h2", "h3", "h4", "h5", "h6", "span", "div", "li", "td", "th",
            ];
            for selector_str in &text_selectors {
                if let Ok(selector) = Selector::parse(selector_str) {
                    for element_ref in document.select(&selector) {
                        let element = element_ref.value();
                        let text_content = element_ref.text().collect::<Vec<_>>().join(" ");

                        if !text_content.trim().is_empty() && text_content.trim().len() > 3 {
                            let element_id = format!(
                                "{}_{}",
                                element.name(),
                                element
                                    .attrs()
                                    .map(|(k, v)| format!("{}={}", k, v))
                                    .collect::<Vec<_>>()
                                    .join("_")
                            );

                            if processed_elements.contains(&element_id) {
                                continue;
                            }
                            processed_elements.insert(element_id);

                            let mut attributes = HashMap::new();
                            for (name, value) in element.attrs() {
                                attributes.insert(name.to_string(), value.to_string());
                            }

                            element_counter += 1;
                            let id = format!("elem_{}", element_counter);

                            let mut dom_element = DomElement::new(element.name().to_string(), id)
                                .with_text_content(text_content.trim().to_string());

                            for (key, value) in &attributes {
                                if key == "id" {
                                    dom_element.element_id = Some(value.clone());
                                } else if key == "class" {
                                    dom_element.class_name = Some(value.clone());
                                }
                                dom_element =
                                    dom_element.with_attribute(key.clone(), value.clone());
                            }

                            dom_element.xpath =
                                self.generate_xpath_for_element(&element_ref, &attributes);
                            dom_element.css_selector =
                                self.generate_css_selector_for_element(&element_ref, &attributes);
                            dom_element.is_visible = !self.is_hidden_element(&attributes);

                            elements.push(dom_element);
                        }
                    }
                }
            }
        }

        Ok(elements)
    }

    fn generate_xpath_for_element(
        &self,
        element_ref: &ElementRef,
        attributes: &HashMap<String, String>,
    ) -> String {
        let tag_name = element_ref.value().name();

        // Priority order for XPath generation
        if let Some(id) = attributes.get("id") {
            format!("//{}[@id='{}']", tag_name, id)
        } else if let Some(name) = attributes.get("name") {
            format!("//{}[@name='{}']", tag_name, name)
        } else if let Some(class) = attributes.get("class") {
            format!("//{}[@class='{}']", tag_name, class)
        } else if let Some(role) = attributes.get("role") {
            format!("//{}[@role='{}']", tag_name, role)
        } else if let Some(aria_label) = attributes.get("aria-label") {
            format!("//{}[@aria-label='{}']", tag_name, aria_label)
        } else {
            // Generate position-based XPath as fallback
            format!("//{}", tag_name)
        }
    }

    fn generate_css_selector_for_element(
        &self,
        element_ref: &ElementRef,
        attributes: &HashMap<String, String>,
    ) -> String {
        let tag_name = element_ref.value().name();

        // Priority order for CSS selector generation
        if let Some(id) = attributes.get("id") {
            format!("{}#{}", tag_name, css_escape(id))
        } else if let Some(name) = attributes.get("name") {
            format!("{}[name='{}']", tag_name, name)
        } else if let Some(class) = attributes.get("class") {
            let classes: Vec<&str> = class.split_whitespace().collect();
            if !classes.is_empty() {
                format!("{}.{}", tag_name, classes.join("."))
            } else {
                tag_name.to_string()
            }
        } else if let Some(role) = attributes.get("role") {
            format!("{}[role='{}']", tag_name, role)
        } else if let Some(data_testid) = attributes.get("data-testid") {
            format!("{}[data-testid='{}']", tag_name, data_testid)
        } else if let Some(aria_label) = attributes.get("aria-label") {
            format!("{}[aria-label='{}']", tag_name, aria_label)
        } else {
            tag_name.to_string()
        }
    }

    fn is_clickable_element(&self, element_ref: &ElementRef) -> bool {
        let tag_name = element_ref.value().name();
        let attributes = element_ref.value().attrs().collect::<HashMap<_, _>>();

        // Standard clickable elements
        if matches!(tag_name, "a" | "button" | "summary" | "area" | "menuitem") {
            return true;
        }

        // Input elements (most types are clickable)
        if tag_name == "input" {
            let input_type = attributes.get("type").unwrap_or(&"text");
            return !matches!(*input_type, "hidden");
        }

        // Elements with click handlers
        if attributes.contains_key("onclick")
            || attributes.contains_key("onchange")
            || attributes.contains_key("onsubmit")
        {
            return true;
        }

        // Elements with clickable roles
        if let Some(role) = attributes.get("role") {
            if matches!(
                *role,
                "button"
                    | "link"
                    | "checkbox"
                    | "radio"
                    | "tab"
                    | "menuitem"
                    | "option"
                    | "switch"
                    | "slider"
            ) {
                return true;
            }
        }

        // Elements that typically indicate clickability
        if attributes.contains_key("tabindex")
            || attributes.contains_key("aria-expanded")
            || attributes.contains_key("aria-haspopup")
            || attributes.get("draggable") == Some(&"true")
        {
            return true;
        }

        false
    }

    fn is_interactable_element(&self, element_ref: &ElementRef) -> bool {
        let tag_name = element_ref.value().name();
        let attributes = element_ref.value().attrs().collect::<HashMap<_, _>>();

        // Standard form elements
        if matches!(tag_name, "input" | "textarea" | "select" | "button") {
            let input_type = attributes.get("type").unwrap_or(&"text");
            return !matches!(*input_type, "hidden");
        }

        // Content editable elements
        if attributes.get("contenteditable") == Some(&"true") {
            return true;
        }

        // Elements with interactive roles
        if let Some(role) = attributes.get("role") {
            if matches!(
                *role,
                "textbox"
                    | "searchbox"
                    | "combobox"
                    | "listbox"
                    | "slider"
                    | "spinbutton"
                    | "switch"
            ) {
                return true;
            }
        }

        // Elements that can receive keyboard input
        if attributes.contains_key("tabindex")
            || attributes.contains_key("onfocus")
            || attributes.contains_key("onblur")
            || attributes.contains_key("onkeydown")
            || attributes.contains_key("onkeyup")
        {
            return true;
        }

        false
    }

    fn is_hidden_element(&self, attributes: &HashMap<String, String>) -> bool {
        // Check for hidden input
        if attributes.get("type") == Some(&"hidden".to_string()) {
            return true;
        }

        // Check for style attributes that hide elements
        if let Some(style) = attributes.get("style") {
            let style_lower = style.to_lowercase();
            if style_lower.contains("display:none")
                || style_lower.contains("display: none")
                || style_lower.contains("visibility:hidden")
                || style_lower.contains("visibility: hidden")
            {
                return true;
            }
        }

        // Check for hidden attribute
        if attributes.contains_key("hidden") {
            return true;
        }

        // Check for common hidden classes
        if let Some(class) = attributes.get("class") {
            let class_lower = class.to_lowercase();
            if class_lower.contains("hidden")
                || class_lower.contains("invisible")
                || class_lower.contains("d-none")
            {
                return true;
            }
        }

        false
    }
}

// Helper function to escape CSS selectors
fn css_escape(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            ' ' => "\\ ".to_string(),
            '.' => "\\.".to_string(),
            '#' => "\\#".to_string(),
            ':' => "\\:".to_string(),
            '[' => "\\[".to_string(),
            ']' => "\\]".to_string(),
            '(' => "\\(".to_string(),
            ')' => "\\)".to_string(),
            '\'' => "\\'".to_string(),
            '"' => "\\\"".to_string(),
            _ => c.to_string(),
        })
        .collect()
}
