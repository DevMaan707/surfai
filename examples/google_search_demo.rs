use browser_ragent::{core::SessionTrait, Config};
use clap::{Arg, Command};
use std::io::{self, Write};
mod helpers;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = Command::new("Enhanced Google Search Demo")
        .version("1.0")
        .about("Interactive Google search with element highlighting")
        .arg(
            Arg::new("headless")
                .long("headless")
                .help("Run browser in headless mode")
                .action(clap::ArgAction::SetTrue),
        )
        .get_matches();

    let headless = matches.get_flag("headless");

    println!("🚀 Starting Enhanced Google Search Demo");
    println!("🔧 Headless mode: {}", if headless { "ON" } else { "OFF" });

    let mut config = Config::default();
    config.browser.headless = headless;
    config.browser.viewport.width = 1920;
    config.browser.viewport.height = 1080;
    config.browser.disable_images = false; // Enable images for better debugging
    config.dom.enable_ai_labels = true;
    config.dom.extract_all_elements = true;

    let mut session = helpers::TestHelper::create_test_session_with_config(config).await?;

    println!("📍 Navigating to Google...");
    session.navigate_and_wait("https://www.google.com").await?;
    println!("✅ Google loaded successfully!");

    // Wait a moment for page to fully load
    tokio::time::sleep(std::time::Duration::from_millis(2000)).await;

    // Highlight all interactive elements
    println!("🎯 Highlighting interactive elements...");
    let highlights = session.highlight_interactive_elements().await?;

    println!("✨ Found {} interactive elements:", highlights.len());
    for (i, highlight) in highlights.iter().enumerate().take(10) {
        println!(
            "  {}. #{} - {} ({})",
            i + 1,
            highlight.element_number,
            highlight.element_type,
            highlight.color
        );
    }

    if !headless {
        println!("\n👀 You should now see numbered overlays on the webpage!");
        println!("🔍 Look for green-highlighted input elements (these are likely search boxes)");

        print!("Press Enter to continue...");
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
    }

    // Find search input (look for input elements)
    let search_element = highlights
        .iter()
        .find(|h| h.element_type == "input")
        .or_else(|| highlights.iter().find(|h| h.element_type == "textarea"));

    if let Some(search_elem) = search_element {
        println!(
            "🔍 Found search element #{} ({})",
            search_elem.element_number, search_elem.element_type
        );

        println!("⌨️  Typing search query...");
        match session
            .type_in_element_by_number(search_elem.element_number, "rust programming")
            .await
        {
            Ok(_) => {
                println!("✅ Successfully typed in search box!");

                // Wait a moment to see the typed text
                tokio::time::sleep(std::time::Duration::from_millis(1000)).await;

                // Try to find and click search button
                let search_button = highlights
                    .iter()
                    .find(|h| h.element_type == "button" || (h.element_type == "input")); // Google search button is often an input

                if let Some(button) = search_button {
                    println!("🔘 Found search button #{}", button.element_number);
                    match session.click_element_by_number(button.element_number).await {
                        Ok(_) => {
                            println!("✅ Search submitted!");
                            tokio::time::sleep(std::time::Duration::from_secs(3)).await;
                        }
                        Err(e) => {
                            println!("⚠️  Button click failed: {}", e);

                            // Try pressing Enter instead
                            println!("🔄 Trying Enter key...");
                            let enter_script = format!(
                                r#"
                                const searchBox = document.querySelector('{}');
                                if (searchBox) {{
                                    const event = new KeyboardEvent('keydown', {{ key: 'Enter', bubbles: true }});
                                    searchBox.dispatchEvent(event);
                                }}
                                "#,
                                search_elem.css_selector.replace("'", "\\'")
                            );
                            session.execute_script(&enter_script).await?;
                        }
                    }
                } else {
                    println!("🔍 No search button found, trying Enter key...");
                    let enter_script = format!(
                        r#"
                        const searchBox = document.querySelector('{}');
                        if (searchBox) {{
                            const event = new KeyboardEvent('keydown', {{ key: 'Enter', bubbles: true }});
                            searchBox.dispatchEvent(event);
                        }}
                        "#,
                        search_elem.css_selector.replace("'", "\\'")
                    );
                    session.execute_script(&enter_script).await?;
                }
            }
            Err(e) => {
                println!("❌ Failed to type in search box: {}", e);
                println!("🔧 Search element details:");
                println!("   Number: {}", search_elem.element_number);
                println!("   Type: {}", search_elem.element_type);
                println!("   Selector: {}", search_elem.css_selector);
            }
        }
    } else {
        println!("❌ No search input found!");
        println!("🔍 Available elements:");
        for highlight in highlights.iter().take(5) {
            println!(
                "  #{}: {} ({})",
                highlight.element_number, highlight.element_type, highlight.css_selector
            );
        }
    }

    if !headless {
        println!("\n🎯 Interactive mode:");
        println!("Enter element number to interact with it, or 'quit' to exit");

        loop {
            print!("Element number (or 'quit'): ");
            io::stdout().flush()?;

            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            let input = input.trim();

            if input == "quit" {
                break;
            }

            if let Ok(num) = input.parse::<usize>() {
                if let Some(highlight) = highlights.iter().find(|h| h.element_number == num) {
                    println!(
                        "🎯 Element #{}: {} ({})",
                        num, highlight.element_type, highlight.css_selector
                    );

                    if highlight.element_type == "input" || highlight.element_type == "textarea" {
                        print!("Text to type: ");
                        io::stdout().flush()?;
                        let mut text_input = String::new();
                        io::stdin().read_line(&mut text_input)?;
                        let text = text_input.trim();

                        match session.type_in_element_by_number(num, text).await {
                            Ok(_) => println!("✅ Text typed successfully!"),
                            Err(e) => println!("❌ Failed to type: {}", e),
                        }
                    } else {
                        match session.click_element_by_number(num).await {
                            Ok(_) => println!("✅ Element clicked successfully!"),
                            Err(e) => println!("❌ Failed to click: {}", e),
                        }
                    }
                } else {
                    println!("❌ Element #{} not found", num);
                }
            }
        }
    }

    session.clear_element_highlights().await?;
    session.close().await?;
    println!("👋 Demo completed!");

    Ok(())
}
