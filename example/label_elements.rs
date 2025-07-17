use surfai::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let browser = BrowserManager::new(false).await?;
    let page = browser.new_page("https://google.com").await?;
    let interactor = ElementInteractor::new(page.clone());

    // Get all elements
    let all_elements = interactor.get_all_elements().await?;
    println!("Total elements: {}", all_elements.len());

    // Get interactable elements
    let interactive = interactor.get_interactable_elements().await?;
    println!("Interactive elements: {}", interactive.len());

    // Label elements
    let labeled = interactor.label_elements(None).await?;
    println!("Labeled elements: {}", labeled.len());

    // Show UI labels
    interactor.label_ui(&labeled, true).await?;
    println!("UI labels shown!");

    // Wait 5 seconds
    tokio::time::sleep(std::time::Duration::from_secs(5)).await;

    // Clear labels
    interactor.clear_label_ui().await?;
    println!("Labels cleared!");

    browser.close().await?;
    Ok(())
}
