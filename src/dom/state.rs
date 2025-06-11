use crate::dom::DomElement;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomState {
    pub url: String,
    pub title: String,
    pub elements: Vec<DomElement>,
    pub clickable_elements: Vec<DomElement>,
    pub input_elements: Vec<DomElement>,
    pub text_elements: Vec<DomElement>,
    pub screenshot_base64: Option<String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl DomState {
    pub fn new(url: String, title: String) -> Self {
        Self {
            url,
            title,
            elements: Vec::new(),
            clickable_elements: Vec::new(),
            input_elements: Vec::new(),
            text_elements: Vec::new(),
            screenshot_base64: None,
            timestamp: chrono::Utc::now(),
        }
    }

    pub fn add_element(&mut self, element: DomElement) {
        if element.is_clickable {
            self.clickable_elements.push(element.clone());
        }

        if matches!(element.tag_name.as_str(), "input" | "textarea" | "select") {
            self.input_elements.push(element.clone());
        }

        if element.text_content.is_some() {
            self.text_elements.push(element.clone());
        }

        self.elements.push(element);
    }

    pub fn set_screenshot(&mut self, screenshot: String) {
        self.screenshot_base64 = Some(screenshot);
    }

    pub fn element_count(&self) -> usize {
        self.elements.len()
    }

    pub fn find_elements_by_tag(&self, tag_name: &str) -> Vec<&DomElement> {
        self.elements
            .iter()
            .filter(|e| e.tag_name == tag_name)
            .collect()
    }

    pub fn find_elements_by_text(&self, text: &str) -> Vec<&DomElement> {
        self.elements
            .iter()
            .filter(|e| {
                e.text_content
                    .as_ref()
                    .map(|t| t.to_lowercase().contains(&text.to_lowercase()))
                    .unwrap_or(false)
            })
            .collect()
    }
}
