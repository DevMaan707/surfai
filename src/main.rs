use browser_ragent::{BrowserConfig, BrowserSession};
use tokio;
use tracing::{error, info};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    info!("Starting browser automation with element highlighting");

    let config = BrowserConfig {
        headless: false, // Must be false to see highlights
        viewport: browser_ragent::Viewport {
            width: 1280,
            height: 720,
        },
        user_agent: Some("Mozilla/5.0 (compatible; RustBot/1.0)".to_string()),
        disable_images: false,
        disable_javascript: false,
    };

    let browser = BrowserSession::new(config).await?;

    // Navigate to a test page
    info!("Navigating to example.com");
    browser.navigate("https://example.com").await?;
    browser.wait_for_page_load(5000).await?;

    info!("Page loaded, highlighting interactive elements...");

    // Highlight all interactive elements with numbers
    let highlights = browser.highlight_elements_batch().await?;

    info!("Highlighted {} interactive elements:", highlights.len());
    for highlight in &highlights {
        info!(
            "  #{}: {} ({})",
            highlight.element_number, highlight.element_type, highlight.color
        );
    }

    // Wait to see the highlights
    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

    // Test clicking by number
    if !highlights.is_empty() {
        let first_element = &highlights[0];
        info!(
            "Demonstrating click on element #{}",
            first_element.element_number
        );

        // Highlight the specific element
        browser
            .highlight_element_by_number(first_element.element_number, &highlights)
            .await?;
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

        // Get element info
        if let Some(element_info) = browser
            .get_element_info_by_number(first_element.element_number, &highlights)
            .await?
        {
            info!(
                "Element #{} info: tag={}, text={:?}, id={:?}",
                first_element.element_number,
                element_info.tag_name,
                element_info.text_content,
                element_info.element_id
            );
        }
    }

    // Navigate to Google for more complex interaction
    info!("Navigating to Google...");
    browser.navigate("https://www.google.com").await?;
    browser.wait_for_page_load(5000).await?;

    // Highlight elements on Google
    let google_highlights = browser.highlight_elements_batch().await?;
    info!(
        "Google page has {} interactive elements",
        google_highlights.len()
    );

    // Find search input by looking for input elements
    if let Some(search_input) = google_highlights.iter().find(|h| h.element_type == "input") {
        info!(
            "Found search input at element #{}",
            search_input.element_number
        );

        // Type in the search box
        browser
            .type_in_element_by_number(
                search_input.element_number,
                "Rust programming",
                &google_highlights,
            )
            .await?;

        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

        // Look for search button
        if let Some(search_button) = google_highlights
            .iter()
            .find(|h| h.element_type == "button" || h.element_type == "input")
        {
            info!("Clicking search button #{}", search_button.element_number);
            browser
                .click_element_by_number(search_button.element_number, &google_highlights)
                .await?;
        }
    }

    // Wait to see results
    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

    // Clear highlights before finishing
    browser.clear_element_highlights().await?;
    info!("Demo completed successfully!");

    Ok(())
}
