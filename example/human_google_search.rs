use std::collections::HashMap;
use surfai::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting human-like search test...");

    // Initialize stealth browser
    let browser = BrowserManager::new_stealth_browser(false).await?;

    // Add error handling for page creation
    let page = match browser.new_page("https://google.com").await {
        Ok(page) => page,
        Err(e) => {
            println!("Failed to create page: {}", e);
            return Ok(());
        }
    };

    let human = HumanInteractor::new(page.clone());

    // Setup stealth mode
    human.apply_stealth().await?;
    human.setup_session().await?;

    // Add small delay after setup
    human.random_delay(2000, 3000).await;

    // Check for blocking
    if human.check_for_captcha().await? {
        println!("Blocking detected, aborting");
        return Ok(());
    }

    // Execute search
    let search_query = "rust programming language";
    println!("Searching for: {}", search_query);

    // Try search with error handling
    match human.type_in_google_search(search_query).await {
        Ok(()) => println!("Successfully typed search query"),
        Err(e) => {
            println!("Failed to type search: {}", e);
            return Ok(());
        }
    }

    human.random_delay(1000, 2000).await;

    match human.submit_search().await {
        Ok(()) => println!("Search submitted"),
        Err(e) => {
            println!("Failed to submit search: {}", e);
            return Ok(());
        }
    }

    // Wait for results with timeout
    println!("Waiting for results...");
    human.random_delay(3000, 5000).await;

    // Check current URL
    let current_url = human.get_current_url().await?;
    println!("Current URL: {}", current_url);

    // Check for post-search blocking
    if human.check_for_captcha().await? {
        println!("Post-search blocking detected");
        return Ok(());
    }

    // Extract and process results
    if human.is_on_results_page().await? {
        println!("On results page");
        let results = human.extract_search_results().await?;

        if results.is_empty() {
            println!("No results found");
        } else {
            println!("Found {} results", results.len());

            // Process first 5 results
            for (i, result) in results.iter().enumerate().take(5) {
                let _title = "No title".to_string();
                let title = result.get("title").unwrap_or(&_title);
                let _link = "No link".to_string();
                let link = result.get("link").unwrap_or(&_link);
                let _snippet = "No snippet".to_string();
                let snippet = result.get("snippet").unwrap_or(&_snippet);

                println!("\n{}. {}", i + 1, title);
                println!("   Link: {}", link);

                if !snippet.is_empty() {
                    println!("   Snippet: {}", snippet);
                }

                // Human-like reading delay
                human.random_delay(500, 1500).await;
            }

            // Simulate browsing behavior
            println!("\nSimulating browsing behavior...");
            human.scroll_human(300, 3).await?;
            human.random_delay(2000, 4000).await;

            // Basic stats
            let with_snippets = results
                .iter()
                .filter(|r| !r.get("snippet").unwrap_or(&String::new()).is_empty())
                .count();

            println!("\nResults summary:");
            println!("Total results: {}", results.len());
            println!("With snippets: {}", with_snippets);

            // Domain analysis
            let mut domains = HashMap::new();
            for result in &results {
                if let Some(link) = result.get("link") {
                    if let Ok(url) = url::Url::parse(link) {
                        if let Some(domain) = url.domain() {
                            *domains.entry(domain.to_string()).or_insert(0) += 1;
                        }
                    }
                }
            }

            if !domains.is_empty() {
                println!("\nTop domains:");
                let mut domain_vec: Vec<_> = domains.into_iter().collect();
                domain_vec.sort_by(|a, b| b.1.cmp(&a.1));

                for (domain, count) in domain_vec.iter().take(3) {
                    println!("  {}: {}", domain, count);
                }
            }
        }
    } else {
        println!("Not on results page");
        let current_url = human.get_current_url().await?;
        println!("Current URL: {}", current_url);
    }

    // Cleanup
    human.idle_time().await;

    // Graceful browser close
    match browser.close().await {
        Ok(()) => println!("Browser closed successfully"),
        Err(e) => println!("Error closing browser: {}", e),
    }

    println!("Test completed");
    Ok(())
}
