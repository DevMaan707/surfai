use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
    pub ai_label: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElementRect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

impl DomElement {
    pub fn new(tag_name: String, id: String) -> Self {
        Self {
            id,
            tag_name,
            element_id: None,
            class_name: None,
            text_content: None,
            attributes: HashMap::new(),
            rect: None,
            is_clickable: false,
            is_visible: true,
            is_interactable: false,
            xpath: String::new(),
            css_selector: String::new(),
            ai_label: None,
        }
    }

    pub fn with_text_content(mut self, text: String) -> Self {
        self.text_content = Some(text);
        self
    }

    pub fn with_attribute(mut self, key: String, value: String) -> Self {
        self.attributes.insert(key, value);
        self
    }

    pub fn with_rect(mut self, rect: ElementRect) -> Self {
        self.rect = Some(rect);
        self
    }

    pub fn set_clickable(mut self, clickable: bool) -> Self {
        self.is_clickable = clickable;
        self
    }

    pub fn set_interactable(mut self, interactable: bool) -> Self {
        self.is_interactable = interactable;
        self
    }

    pub fn generate_ai_label(&mut self) {
        let mut label_parts = vec![];

        // Start with element type
        match self.tag_name.as_str() {
            "input" => {
                if let Some(input_type) = self.attributes.get("type") {
                    label_parts.push(format!("{} input field", input_type));
                } else {
                    label_parts.push("text input field".to_string());
                }
            }
            "button" => label_parts.push("button".to_string()),
            "a" => label_parts.push("link".to_string()),
            "select" => label_parts.push("dropdown menu".to_string()),
            "textarea" => label_parts.push("text area".to_string()),
            _ => label_parts.push(format!("{} element", self.tag_name)),
        }

        // Add identifying information
        if let Some(name) = self.attributes.get("name") {
            label_parts.push(format!("named '{}'", name));
        }

        if let Some(id) = &self.element_id {
            label_parts.push(format!("with ID '{}'", id));
        }

        if let Some(placeholder) = self.attributes.get("placeholder") {
            label_parts.push(format!("placeholder '{}'", placeholder));
        }

        if let Some(aria_label) = self.attributes.get("aria-label") {
            label_parts.push(format!("labeled '{}'", aria_label));
        }

        if let Some(title) = self.attributes.get("title") {
            label_parts.push(format!("titled '{}'", title));
        }

        // Add text content if available and meaningful
        if let Some(text) = &self.text_content {
            let clean_text = text.trim();
            if !clean_text.is_empty() && clean_text.len() < 100 {
                label_parts.push(format!("containing '{}'", clean_text));
            }
        }

        // Add interaction information
        if self.is_clickable {
            label_parts.push("(clickable)".to_string());
        }

        if self.is_interactable && matches!(self.tag_name.as_str(), "input" | "textarea") {
            label_parts.push("(can type here)".to_string());
        }

        // Special handling for Google search elements
        if self.attributes.get("name") == Some(&"q".to_string()) {
            label_parts.clear();
            label_parts.push("Google search box (main search input)".to_string());
        }

        if self.attributes.get("role") == Some(&"searchbox".to_string()) {
            label_parts.push("(search box)".to_string());
        }

        self.ai_label = Some(label_parts.join(" "));
    }
}
