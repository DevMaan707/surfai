use clap::{Arg, Command};
use std::io::{self, Write};
use surfai::{browser::session::ElementHighlight, BrowserSession, SessionTrait};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = Command::new("Smart Google Search Demo")
        .version("1.0")
        .about("Google search with smart navigation and element highlighting")
        .arg(
            Arg::new("headless")
                .long("headless")
                .help("Run browser in headless mode")
                .action(clap::ArgAction::SetTrue),
        )
        .get_matches();

    let headless = matches.get_flag("headless");

    println!("🚀 Smart Google Search Demo");

    let mut session = if headless {
        BrowserSession::quick_start().await?
    } else {
        BrowserSession::demo_mode().await?
    };

    let nav_result = session.navigate_smart("https://www.google.com").await?;
    println!("✅ Google loaded in {}ms", nav_result.duration_ms);
    let highlights = session.highlight_interactive_elements().await?;
    println!("🎯 Found {} interactive elements", highlights.len());

    if let Some(search_elem) = highlights.iter().find(|h| h.element_type == "input") {
        println!("🔍 Using search element #{}", search_elem.element_number);
        session
            .type_with_refresh(&search_elem.css_selector, "rust programming")
            .await?;
        let enter_script = format!(
            "document.querySelector('{}').dispatchEvent(new KeyboardEvent('keydown', {{key: 'Enter', bubbles: true}}))",
            search_elem.css_selector.replace("'", "\\'")
        );
        session.execute_script(&enter_script).await?;

        println!("✅ Search submitted");
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    }
    if !headless {
        interactive_mode(&mut session, &highlights).await?;
    }
    session.close().await?;
    println!("👋 Demo completed!");
    Ok(())
}

async fn interactive_mode(
    session: &mut BrowserSession<surfai::ChromeBrowser>,
    highlights: &[ElementHighlight],
) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🎮 Interactive Mode");
    println!("Commands: <number> (click/type) | 'list' (show elements) | 'quit'");

    loop {
        print!("\n> ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();

        match input {
            "quit" => break,
            "list" => {
                for (i, h) in highlights.iter().enumerate().take(10) {
                    println!("  {}. #{}: {}", i + 1, h.element_number, h.element_type);
                }
            }
            _ => {
                if let Ok(num) = input.parse::<usize>() {
                    if let Some(elem) = highlights.iter().find(|h| h.element_number == num) {
                        if elem.element_type == "input" || elem.element_type == "textarea" {
                            print!("Text: ");
                            io::stdout().flush()?;
                            let mut text = String::new();
                            io::stdin().read_line(&mut text)?;

                            match session.type_in_element_by_number(num, text.trim()).await {
                                Ok(_) => println!("✅ Typed successfully"),
                                Err(e) => println!("❌ Failed: {}", e),
                            }
                        } else {
                            match session.click_element_by_number_with_refresh(num).await {
                                Ok(_) => println!("✅ Clicked successfully"),
                                Err(e) => println!("❌ Failed: {}", e),
                            }
                        }
                    } else {
                        println!("❌ Element #{} not found", num);
                    }
                }
            }
        }
    }

    Ok(())
}
