use surfai::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting Google search automation...");

    let browser = BrowserManager::new(false).await?;
    let page = browser.new_page("https://google.com").await?;
    let interactor = ElementInteractor::new(page.clone());

    // Wait a bit for page to fully load
    tokio::time::sleep(std::time::Duration::from_secs(2)).await;

    // Get all elements first
    let all_elements = interactor.get_all_elements().await?;
    println!("Found {} total elements", all_elements.len());

    // Label elements to see what we're working with
    let labeled_elements = interactor.label_elements(None).await?;
    println!("Labeled {} elements", labeled_elements.len());

    // Show UI labels for debugging
    interactor.label_ui(&labeled_elements, false).await?;
    println!("UI labels shown! Check the browser to see labeled elements.");

    // Wait to see labels
    tokio::time::sleep(std::time::Duration::from_secs(3)).await;

    // Clear labels
    interactor.clear_label_ui().await?;

    // Search for a product
    let search_query = "iPhone 15 Pro";
    println!("Typing '{}' in search box...", search_query);

    match interactor.type_in_search_box(search_query).await {
        Ok(()) => println!("Successfully typed in search box"),
        Err(e) => {
            println!("❌ Failed to type in search box: {}", e);
            // Let's try to find the search box manually
            let elements = interactor.get_interactable_elements().await?;
            println!("🔍 Available elements:");
            for element in elements.iter().take(10) {
                println!(
                    "  - {}: {} ({})",
                    element.unique_id, element.selector, element.tag_name
                );
            }
        }
    }

    // Wait a moment
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;

    // Click search button
    println!("Clicking search button...");
    match interactor.click_search_button().await {
        Ok(()) => println!("Successfully clicked search button"),
        Err(e) => println!("❌ Failed to click search button: {}", e),
    }

    // Wait for results to load
    tokio::time::sleep(std::time::Duration::from_secs(3)).await;

    // Extract search results
    println!("Extracting search results...");
    let results = interactor.extract_search_results().await?;

    println!("Found {} search results:", results.len());
    for (i, result) in results.iter().enumerate().take(5) {
        println!(
            "{}. {}",
            i + 1,
            result.get("title").unwrap_or(&"No title".to_string())
        );
        println!(
            "   Link: {}",
            result.get("link").unwrap_or(&"No link".to_string())
        );
        println!(
            "   Snippet: {}",
            result.get("snippet").unwrap_or(&"No snippet".to_string())
        );
        println!();
    }

    // Take a screenshot of results
    let scraper = PageScraper::new(page.clone());
    let screenshot = scraper.take_screenshot().await?;
    std::fs::write("google_search_results.png", screenshot)?;
    println!("📸 Screenshot saved as 'google_search_results.png'");

    // Get page data
    let page_data = scraper.scrape_page().await?;
    println!("Page title: {}", page_data.title);
    println!("Found {} links on results page", page_data.links.len());

    browser.close().await?;
    println!("Google search automation completed successfully!");

    Ok(())
}
