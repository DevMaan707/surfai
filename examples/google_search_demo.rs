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

    let ai_elements = session.get_ai_elements().await?;
    display_ai_elements(&ai_elements);

    if !headless {
        ai_interactive_mode(&mut session).await?;
    } else {
        auto_search_demo(&mut session, &ai_elements).await?;
    }

    session.close().await?;
    println!("👋 Demo completed!");
    Ok(())
}

fn display_ai_elements(ai_elements: &[surfai::browser::session::AIElement]) {
    println!("\n🤖 AI-Analyzed Interactive Elements:");
    for (i, element) in ai_elements.iter().enumerate() {
        println!("{}. Element #{}", i + 1, element.element_number);
        println!(
            "   🏷️  AI Label: {}",
            element.label.as_ref().unwrap_or(&"No label".to_string())
        );
        println!("   📝 Description: {}", element.description);
        println!("   🎯 Type: {}", element.element_type);
        println!("   🔧 Capabilities: {:?}", element.capabilities);
        println!("   📋 Instructions: {}", element.ai_instructions);
        println!("   ---");
    }
}

async fn auto_search_demo(
    session: &mut BrowserSession<surfai::ChromeBrowser>,
    ai_elements: &[surfai::browser::session::AIElement],
) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(search_elem) = ai_elements.iter().find(|e| {
        e.ai_instructions.to_lowercase().contains("search")
            || e.label
                .as_ref()
                .map(|l| l.to_lowercase().contains("search"))
                .unwrap_or(false)
    }) {
        println!(
            "🔍 Auto-selecting: {}",
            search_elem
                .label
                .as_ref()
                .unwrap_or(&"Search element".to_string())
        );

        session
            .type_with_refresh(&search_elem.selector, "rust programming")
            .await?;

        let enter_script = format!(
            "document.querySelector('{}').dispatchEvent(new KeyboardEvent('keydown', {{key: 'Enter', bubbles: true}}))",
            search_elem.selector.replace("'", "\\'")
        );
        session.execute_script(&enter_script).await?;

        println!("✅ Search submitted automatically");
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    }

    Ok(())
}

async fn ai_interactive_mode(
    session: &mut BrowserSession<surfai::ChromeBrowser>,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🎮 AI-Powered Interactive Mode");
    println!("Commands:");
    println!("  - Type part of an AI label to select element");
    println!("  - 'list' - Show all elements again");
    println!("  - 'refresh' - Refresh elements");
    println!("  - 'quit' - Exit");

    // Initialize current elements
    let mut current_ai_elements = session.get_ai_elements().await?;

    loop {
        print!("\n🤖 AI> ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim().to_lowercase();

        match input.as_str() {
            "quit" => break,
            "list" => {
                println!("\n🤖 Available AI-Labeled Elements:");
                for (i, element) in current_ai_elements.iter().enumerate() {
                    println!(
                        "{}. {}",
                        i + 1,
                        element.label.as_ref().unwrap_or(&"No label".to_string())
                    );
                    println!(
                        "   Type: {} | Instructions: {}",
                        element.element_type, element.ai_instructions
                    );
                }
            }
            "refresh" => {
                session.highlight_interactive_elements().await?;
                current_ai_elements = session.get_ai_elements().await?;
                println!(
                    "🔄 Refreshed - found {} elements",
                    current_ai_elements.len()
                );
                display_ai_elements(&current_ai_elements);
            }
            _ => {
                // Find element by label matching
                let matching_elements: Vec<&surfai::browser::session::AIElement> =
                    current_ai_elements
                        .iter()
                        .filter(|e| {
                            e.label
                                .as_ref()
                                .map(|l| l.to_lowercase().contains(&input))
                                .unwrap_or(false)
                                || e.description.to_lowercase().contains(&input)
                                || e.element_type.to_lowercase().contains(&input)
                        })
                        .collect();

                if matching_elements.is_empty() {
                    println!("❌ No elements found matching '{}'", input);
                    continue;
                }

                if matching_elements.len() == 1 {
                    let element = matching_elements[0];
                    println!(
                        "🎯 Selected: {}",
                        element.label.as_ref().unwrap_or(&"No label".to_string())
                    );
                    println!("📋 Instructions: {}", element.ai_instructions);

                    interact_with_ai_element(session, element).await?;
                } else {
                    println!("🔍 Multiple matches found:");
                    for (i, element) in matching_elements.iter().enumerate() {
                        println!(
                            "{}. {}",
                            i + 1,
                            element.label.as_ref().unwrap_or(&"No label".to_string())
                        );
                    }

                    print!("Choose number (1-{}): ", matching_elements.len());
                    io::stdout().flush()?;

                    let mut choice_input = String::new();
                    io::stdin().read_line(&mut choice_input)?;

                    if let Ok(choice) = choice_input.trim().parse::<usize>() {
                        if choice > 0 && choice <= matching_elements.len() {
                            let element = matching_elements[choice - 1];
                            println!(
                                "🎯 Selected: {}",
                                element.label.as_ref().unwrap_or(&"No label".to_string())
                            );
                            interact_with_ai_element(session, element).await?;
                        } else {
                            println!("❌ Invalid choice");
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

async fn interact_with_ai_element(
    session: &mut BrowserSession<surfai::ChromeBrowser>,
    element: &surfai::browser::session::AIElement,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("🔧 Element capabilities: {:?}", element.capabilities);

    if element
        .capabilities
        .contains(&"can_receive_text_input".to_string())
    {
        print!("⌨️  Enter text to type (or press Enter for default): ");
        io::stdout().flush()?;

        let mut text_input = String::new();
        io::stdin().read_line(&mut text_input)?;
        let text = text_input.trim();

        let final_text = if text.is_empty() {
            match element.element_type.as_str() {
                s if s.contains("search") => "rust programming",
                s if s.contains("email") => "test@example.com",
                s if s.contains("password") => "password123",
                _ => "test input",
            }
        } else {
            text
        };

        println!("⌨️  Typing: '{}'", final_text);
        match session
            .type_with_refresh(&element.selector, final_text)
            .await
        {
            Ok(_) => println!(
                "✅ Successfully typed in: {}",
                element.label.as_ref().unwrap_or(&"element".to_string())
            ),
            Err(e) => println!("❌ Failed to type: {}", e),
        }

        if element.ai_instructions.to_lowercase().contains("search")
            || element.ai_instructions.to_lowercase().contains("enter")
        {
            print!("🔍 Press Enter in this field? (y/n): ");
            io::stdout().flush()?;

            let mut confirm = String::new();
            io::stdin().read_line(&mut confirm)?;

            if confirm.trim().to_lowercase().starts_with('y') {
                let enter_script = format!(
                    "document.querySelector('{}').dispatchEvent(new KeyboardEvent('keydown', {{key: 'Enter', bubbles: true}}))",
                    element.selector.replace("'", "\\'")
                );
                session.execute_script(&enter_script).await?;
                println!("✅ Pressed Enter");
            }
        }
    } else if element.capabilities.contains(&"clickable".to_string()) {
        println!("🖱️  Clicking element...");
        match session
            .click_element_by_number_with_refresh(element.element_number)
            .await
        {
            Ok(_) => println!(
                "✅ Successfully clicked: {}",
                element.label.as_ref().unwrap_or(&"element".to_string())
            ),
            Err(e) => println!("❌ Failed to click: {}", e),
        }
    } else {
        println!(
            "ℹ️  This element is for information only: {}",
            element.description
        );
    }

    tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
    Ok(())
}
