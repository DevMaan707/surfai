use crate::errors::Result;
use crate::{BrowserConfig, BrowserSession, DomState};

pub struct TestHelper;

impl TestHelper {
    pub async fn create_test_browser() -> Result<BrowserSession> {
        let config = BrowserConfig {
            headless: true,
            ..Default::default()
        };
        BrowserSession::new(config).await
    }

    pub async fn navigate_and_extract(browser: &BrowserSession, url: &str) -> Result<DomState> {
        browser.navigate(url).await?;
        browser.wait_for_page_load(5000).await?;
        browser.get_dom_state_with_labels(false).await
    }

    pub fn count_elements_by_type(dom_state: &DomState, element_type: &str) -> usize {
        dom_state
            .elements
            .iter()
            .filter(|e| e.tag_name == element_type)
            .count()
    }

    pub fn find_elements_with_text<'a>(
        dom_state: &'a DomState,
        text: &str,
    ) -> Vec<&'a crate::dom::DomElement> {
        dom_state
            .elements
            .iter()
            .filter(|e| {
                if let Some(element_text) = &e.text_content {
                    element_text.to_lowercase().contains(&text.to_lowercase())
                } else {
                    false
                }
            })
            .collect()
    }

    pub fn get_form_elements(dom_state: &DomState) -> Vec<&crate::dom::DomElement> {
        dom_state
            .elements
            .iter()
            .filter(|e| {
                matches!(
                    e.tag_name.as_str(),
                    "input" | "textarea" | "select" | "button"
                )
            })
            .collect()
    }

    pub fn find_clickable_elements(dom_state: &DomState) -> Vec<&crate::dom::DomElement> {
        dom_state.clickable_elements.iter().collect()
    }

    pub fn find_elements_by_attribute<'a>(
        dom_state: &'a DomState,
        attribute_name: &str,
        attribute_value: &str,
    ) -> Vec<&'a crate::dom::DomElement> {
        dom_state
            .elements
            .iter()
            .filter(|e| {
                e.attributes
                    .get(attribute_name)
                    .map(|v| v == attribute_value)
                    .unwrap_or(false)
            })
            .collect()
    }

    pub fn find_elements_by_id<'a>(
        dom_state: &'a DomState,
        id: &str,
    ) -> Vec<&'a crate::dom::DomElement> {
        dom_state
            .elements
            .iter()
            .filter(|e| {
                e.element_id
                    .as_ref()
                    .map(|element_id| element_id == id)
                    .unwrap_or(false)
            })
            .collect()
    }

    pub fn find_elements_by_class<'a>(
        dom_state: &'a DomState,
        class_name: &str,
    ) -> Vec<&'a crate::dom::DomElement> {
        dom_state
            .elements
            .iter()
            .filter(|e| {
                e.class_name
                    .as_ref()
                    .map(|classes| classes.split_whitespace().any(|c| c == class_name))
                    .unwrap_or(false)
            })
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

// Alternative approach: Return owned data instead of references
impl TestHelper {
    pub fn find_elements_with_text_owned(
        dom_state: &DomState,
        text: &str,
    ) -> Vec<crate::dom::DomElement> {
        dom_state
            .elements
            .iter()
            .filter(|e| {
                if let Some(element_text) = &e.text_content {
                    element_text.to_lowercase().contains(&text.to_lowercase())
                } else {
                    false
                }
            })
            .cloned()
            .collect()
    }

    pub fn get_form_elements_owned(dom_state: &DomState) -> Vec<crate::dom::DomElement> {
        dom_state
            .elements
            .iter()
            .filter(|e| {
                matches!(
                    e.tag_name.as_str(),
                    "input" | "textarea" | "select" | "button"
                )
            })
            .cloned()
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio_test;

    #[tokio::test]
    async fn test_browser_creation() {
        let browser = TestHelper::create_test_browser().await;
        assert!(browser.is_ok());
    }

    #[tokio::test]
    async fn test_navigation() {
        let browser = TestHelper::create_test_browser().await.unwrap();
        let result = browser.navigate("https://example.com").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_dom_extraction() {
        let browser = TestHelper::create_test_browser().await.unwrap();
        let dom_state = TestHelper::navigate_and_extract(&browser, "https://example.com").await;
        assert!(dom_state.is_ok());

        let dom = dom_state.unwrap();
        assert!(!dom.elements.is_empty());
        assert!(!dom.url.is_empty());
        assert!(!dom.title.is_empty());
    }

    #[tokio::test]
    async fn test_element_finding() {
        let browser = TestHelper::create_test_browser().await.unwrap();
        let dom_state = TestHelper::navigate_and_extract(&browser, "https://example.com").await;
        assert!(dom_state.is_ok());

        let dom = dom_state.unwrap();

        // Test finding elements with text
        let text_elements = TestHelper::find_elements_with_text(&dom, "example");
        // Should find some elements containing "example" text

        // Test form elements
        let form_elements = TestHelper::get_form_elements(&dom);
        // May or may not have form elements depending on the page

        // Test page stats
        let stats = TestHelper::get_page_stats(&dom);
        assert!(stats.total_elements > 0);

        println!("Page stats: {:?}", stats);
    }

    #[tokio::test]
    async fn test_element_filtering() {
        let browser = TestHelper::create_test_browser().await.unwrap();
        let dom_state = TestHelper::navigate_and_extract(&browser, "https://example.com").await;
        assert!(dom_state.is_ok());

        let dom = dom_state.unwrap();

        // Test finding by tag type
        let div_count = TestHelper::count_elements_by_type(&dom, "div");
        let p_count = TestHelper::count_elements_by_type(&dom, "p");

        println!(
            "Found {} div elements and {} p elements",
            div_count, p_count
        );

        // Test owned vs borrowed versions
        let text_elements_borrowed = TestHelper::find_elements_with_text(&dom, "domain");
        let text_elements_owned = TestHelper::find_elements_with_text_owned(&dom, "domain");

        assert_eq!(text_elements_borrowed.len(), text_elements_owned.len());
    }
}
