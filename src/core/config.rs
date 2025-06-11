use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub browser: BrowserConfig,
    pub dom: DomConfig,
    pub session: SessionConfig,
    pub features: FeatureFlags,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserConfig {
    pub browser_type: BrowserType,
    pub headless: bool,
    pub viewport: Viewport,
    pub user_agent: Option<String>,
    pub disable_images: bool,
    pub disable_javascript: bool,
    pub args: Vec<String>,
    pub timeout_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomConfig {
    pub extract_all_elements: bool,
    pub include_hidden_elements: bool,
    pub max_text_length: usize,
    pub enable_ai_labels: bool,
    pub screenshot_quality: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionConfig {
    pub navigation_timeout_ms: u64,
    pub element_timeout_ms: u64,
    pub retry_attempts: u32,
    pub enable_logging: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureFlags {
    pub enable_highlighting: bool,
    pub enable_action_registry: bool,
    pub enable_state_tracking: bool,
    pub enable_ai_integration: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BrowserType {
    Chrome,
    Firefox,
    Safari,
    Edge,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Viewport {
    pub width: u32,
    pub height: u32,
    pub device_scale_factor: f64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            browser: BrowserConfig::default(),
            dom: DomConfig::default(),
            session: SessionConfig::default(),
            features: FeatureFlags::default(),
        }
    }
}

impl Default for BrowserConfig {
    fn default() -> Self {
        Self {
            browser_type: BrowserType::Chrome,
            headless: true,
            viewport: Viewport::default(),
            user_agent: None,
            disable_images: false,
            disable_javascript: false,
            args: vec![],
            timeout_ms: 30000,
        }
    }
}

impl Default for DomConfig {
    fn default() -> Self {
        Self {
            extract_all_elements: true,
            include_hidden_elements: false,
            max_text_length: 1000,
            enable_ai_labels: false,
            screenshot_quality: 80,
        }
    }
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            navigation_timeout_ms: 5000,
            element_timeout_ms: 2000,
            retry_attempts: 3,
            enable_logging: true,
        }
    }
}

impl Default for FeatureFlags {
    fn default() -> Self {
        Self {
            enable_highlighting: false,
            enable_action_registry: false,
            enable_state_tracking: false,
            enable_ai_integration: false,
        }
    }
}

impl Default for Viewport {
    fn default() -> Self {
        Self {
            width: 1280,
            height: 720,
            device_scale_factor: 1.0,
        }
    }
}
