use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Viewport {
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserConfig {
    pub headless: bool,
    pub viewport: Viewport,
    pub user_agent: Option<String>,
    pub disable_images: bool,
    pub disable_javascript: bool,
}

impl Default for BrowserConfig {
    fn default() -> Self {
        Self {
            headless: true,
            viewport: Viewport {
                width: 1280,
                height: 720,
            },
            user_agent: None,
            disable_images: false,
            disable_javascript: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElementRect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElementInfo {
    pub tag_name: String,
    pub element_id: Option<String>,
    pub class_name: Option<String>,
    pub text_content: Option<String>,
    pub attributes: HashMap<String, String>,
    pub rect: Option<ElementRect>,
}
