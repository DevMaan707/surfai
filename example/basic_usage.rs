use surfai::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let browser = BrowserManager::new(false).await?;
    let page = browser.new_page("https://google.com").await?;
    let scraper = PageScraper::new(page.clone());
    let interactor = ElementInteractor::new(page.clone());
    let cdp = CDPExecutor::new(page.clone());
    let page_data = scraper.scrape_page().await?;
    println!("Page title: {}", page_data.title);
    let elements = interactor.find_interactable_elements().await?;
    println!("Found {} interactable elements", elements.len());
    let custom_script = CDPScript {
        name: "get_page_info".to_string(),
        script: r#"
            return {
                url: window.location.href,
                title: document.title,
                readyState: document.readyState,
                elementCount: document.querySelectorAll('*').length
            };
        "#
        .to_string(),
        params: None,
    };

    let result = cdp.execute_cdp_script(custom_script).await?;
    println!("CDP Script result: {}", result);
    let metrics = cdp.get_performance_metrics().await?;
    println!("Performance metrics available: {:?}", metrics);
    let screenshot = scraper.take_screenshot().await?;
    std::fs::write("page_screenshot.png", screenshot)?;
    println!("Screenshot saved");

    browser.close().await?;
    Ok(())
}
