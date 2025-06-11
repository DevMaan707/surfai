use crate::dom::{DomElement, DomState};
use crate::errors::Result;
use async_trait::async_trait;

/// Core DOM processing trait
///
/// This trait defines how DOM state is extracted and processed from web pages.
/// Different implementations can provide different levels of detail or optimization.
#[async_trait]
pub trait DomProcessorTrait: Send + Sync {
    /// Extract complete DOM state from a browser tab
    async fn extract_dom_state<B: crate::core::BrowserTrait>(
        &self,
        browser: &B,
        tab: &B::TabHandle,
        include_screenshot: bool,
    ) -> Result<DomState>;

    /// Extract only interactive elements
    async fn extract_interactive_elements<B: crate::core::BrowserTrait>(
        &self,
        browser: &B,
        tab: &B::TabHandle,
    ) -> Result<Vec<DomElement>>;

    /// Add AI-friendly labels to elements
    async fn add_ai_labels(&self, elements: &mut Vec<DomElement>) -> Result<()>;

    /// Filter elements by criteria
    fn filter_elements(&self, elements: &[DomElement], criteria: &ElementFilter)
    -> Vec<DomElement>;

    /// Generate element selectors
    fn generate_selector(&self, element: &DomElement, selector_type: SelectorType) -> String;
}

/// Criteria for filtering DOM elements
#[derive(Debug, Clone)]
pub struct ElementFilter {
    pub tag_names: Option<Vec<String>>,
    pub has_text: Option<String>,
    pub is_visible: Option<bool>,
    pub is_interactive: Option<bool>,
    pub has_attribute: Option<(String, Option<String>)>,
}

/// Types of selectors that can be generated
#[derive(Debug, Clone)]
pub enum SelectorType {
    Css,
    XPath,
    TestId,
}
