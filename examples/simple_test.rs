use browser_ragent::core::SessionTrait;
mod helpers;
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Starting simple browser test...");

    let mut session = helpers::TestHelper::create_test_session().await?;

    println!("📍 Navigating to example.com...");
    session.navigate_and_wait("https://google.com").await?;

    let url = session.current_url().await?;
    println!("✅ Current URL: {}", url);

    let dom_state = session.get_page_state(false).await?;
    println!("📊 Page has {} elements", dom_state.elements.len());

    session.close().await?;
    println!("✅ Test completed successfully!");

    Ok(())
}
