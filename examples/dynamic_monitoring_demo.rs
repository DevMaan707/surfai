use browser_ragent::{core::SessionTrait, Config, DefaultSession};
use clap::{Arg, Command};
use std::io::{self, Write};
use tokio::time::{sleep, Duration};

mod helpers;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = Command::new("Dynamic Monitoring Showcase")
        .version("1.0")
        .about("Showcases built-in navigation detection and dynamic element discovery")
        .arg(
            Arg::new("headless")
                .long("headless")
                .help("Run browser in headless mode")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("mode")
                .long("mode")
                .value_name("MODE")
                .help("Demo mode: auto, interactive, or showcase")
                .default_value("showcase"),
        )
        .get_matches();

    let headless = matches.get_flag("headless");
    let mode = matches.get_one::<String>("mode").unwrap();

    println!("🚀 Dynamic Monitoring Showcase using Built-in Methods");
    println!("🔧 Mode: {} | Headless: {}", mode, headless);

    // Configure session with all dynamic features enabled
    let mut config = Config::default();
    config.browser.headless = headless;
    config.browser.viewport.width = 1920;
    config.browser.viewport.height = 1080;
    config.dom.enable_ai_labels = true;
    config.dom.extract_all_elements = true;
    config.features.enable_highlighting = true;
    config.features.enable_state_tracking = true;

    let mut session = helpers::TestHelper::create_test_session_with_config(config).await?;

    // Enable auto-refresh for dynamic monitoring
    session.set_auto_refresh(true);
    println!("✅ Auto-refresh enabled for dynamic element detection");

    match mode.as_str() {
        "auto" => run_automated_showcase(&mut session).await?,
        "interactive" => run_interactive_showcase(&mut session).await?,
        _ => run_comprehensive_showcase(&mut session).await?,
    }

    session.close().await?;
    println!("👋 Showcase completed!");

    Ok(())
}

async fn run_comprehensive_showcase(
    session: &mut DefaultSession,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🎯 === COMPREHENSIVE DYNAMIC MONITORING SHOWCASE ===");

    // Demo 1: Advanced Navigation with Built-in Detection
    println!("\n📍 DEMO 1: Advanced Navigation Detection");
    println!("Using navigate_and_wait_reactive() with built-in NavigationManager...");

    let nav_result = session
        .navigate_and_wait_reactive("https://www.google.com")
        .await?;
    println!("✅ Navigation completed successfully!");
    println!("   📊 Navigation details:");
    println!("      Reason: {}", nav_result.reason);
    println!("      URL: {}", nav_result.url);
    println!("      Ready state: {}", nav_result.ready_state);
    println!("      Duration: {}ms", nav_result.duration_ms);

    // Get element count (avoid holding borrow)
    let initial_element_count = session.get_highlighted_elements().len();
    println!(
        "   🎯 Auto-highlighted {} interactive elements",
        initial_element_count
    );

    // Demo 2: Built-in AI Element Analysis
    println!("\n🤖 DEMO 2: Built-in AI Element Analysis");
    println!("Using get_ai_elements() with automatic labeling...");

    let ai_elements = session.get_ai_elements().await?;
    println!("✅ Analyzed {} elements with AI labels", ai_elements.len());

    // Show AI-enhanced element information
    for (i, element) in ai_elements.iter().take(5).enumerate() {
        println!(
            "   {}. Element #{}: {}",
            i + 1,
            element.element_number,
            element.element_type
        );
        println!(
            "      🏷️  AI Label: {}",
            element.label.as_ref().unwrap_or(&"No label".to_string())
        );
        println!("      📝 Description: {}", element.description);
        println!("      🎯 Instructions: {}", element.ai_instructions);
        println!("      🔧 Capabilities: {:?}", element.capabilities);
        println!();
    }

    // Demo 3: Dynamic Element Interaction with Auto-refresh
    println!("\n⚡ DEMO 3: Dynamic Element Interaction with Auto-refresh");

    // Find search element using AI analysis and store needed info
    let search_element_info = ai_elements
        .iter()
        .find(|e| {
            e.element_type.contains("input")
                && (e.ai_instructions.to_lowercase().contains("search")
                    || e.description.to_lowercase().contains("search"))
        })
        .map(|e| {
            (
                e.selector.clone(),
                e.description.clone(),
                e.ai_instructions.clone(),
            )
        });

    if let Some((search_selector, search_description, search_instructions)) = search_element_info {
        println!("🔍 Found search element: {}", search_description);
        println!("🎯 AI Instructions: {}", search_instructions);

        // Use the built-in type_with_refresh method for dynamic monitoring
        println!("⌨️  Typing with auto-refresh monitoring...");
        session
            .type_with_refresh(&search_selector, "dynamic web development")
            .await?;

        println!("✅ Typing completed - auto-refresh handled DOM changes automatically");

        // Check if new elements appeared after typing
        let new_element_count = session.get_highlighted_elements().len();
        if new_element_count != initial_element_count {
            println!(
                "   📈 Element count changed: {} → {}",
                initial_element_count, new_element_count
            );
            println!("   🔄 New elements automatically detected and highlighted!");
        }

        sleep(Duration::from_secs(2)).await;

        // Demonstrate waiting for specific elements (like autocomplete)
        println!("⏳ Using wait_for_elements() to detect dynamic content...");
        let found_suggestions = session
            .wait_for_elements("[role='listbox'], [role='option'], .suggestion", 3000)
            .await?;

        if found_suggestions {
            println!("✅ Dynamic suggestions detected and elements refreshed!");
        } else {
            println!("⏰ No suggestions appeared within timeout");
        }
    } else {
        println!("⚠️ No search element found, skipping interaction demo");
    }

    // Demo 4: Navigation Change Detection
    println!("\n🚀 DEMO 4: Navigation Change Detection");
    println!("Navigating to a different site to test navigation monitoring...");

    let nav_result = session
        .navigate_and_wait_reactive("https://www.wikipedia.org")
        .await?;
    println!("✅ Navigation to Wikipedia completed!");
    println!("   📊 Navigation analysis:");
    println!("      Reason: {}", nav_result.reason);
    println!("      Duration: {}ms", nav_result.duration_ms);

    // Session automatically refreshed elements and restarted monitoring
    let wiki_element_count = session.get_highlighted_elements().len();
    println!(
        "   🎯 Auto-detected {} new interactive elements on Wikipedia",
        wiki_element_count
    );

    // Show element type distribution
    show_element_distribution(session).await;

    // Demo 5: Continuous DOM Monitoring
    println!("\n🔄 DEMO 5: Continuous DOM Monitoring");

    // Find Wikipedia search box and store selector
    let wiki_ai_elements = session.get_ai_elements().await?;
    let wiki_search_info = wiki_ai_elements
        .iter()
        .find(|e| {
            e.ai_instructions.to_lowercase().contains("search")
                || e.description.to_lowercase().contains("search")
        })
        .map(|e| (e.selector.clone(), e.description.clone()));

    if let Some((selector, description)) = wiki_search_info {
        println!("🔍 Found Wikipedia search: {}", description);

        // Type gradually to trigger multiple DOM changes
        println!("⌨️  Typing gradually to demonstrate continuous monitoring...");

        session.type_with_refresh(&selector, "artificial").await?;
        sleep(Duration::from_millis(1000)).await;

        // Clear and type new text
        session
            .type_with_refresh(&selector, "artificial intelligence")
            .await?;
        sleep(Duration::from_millis(1000)).await;

        session
            .type_with_refresh(&selector, "artificial intelligence machine learning")
            .await?;

        println!("✅ Each typing action automatically monitored for DOM changes!");
    }

    // Demo 6: Element State Tracking
    println!("\n📊 DEMO 6: Element State Tracking");

    // Get current interactive elements for comparison
    let current_elements = session.get_current_interactive_elements().await?;
    println!("📈 Current page analysis:");
    println!("   🎯 Interactive elements: {}", current_elements.len());

    // Show element capabilities analysis
    let mut capability_stats = std::collections::HashMap::new();
    for element in &current_elements {
        for capability in &element.capabilities {
            *capability_stats.entry(capability.clone()).or_insert(0) += 1;
        }
    }

    println!("   🔧 Element capabilities distribution:");
    for (capability, count) in capability_stats {
        println!("      {}: {}", capability, count);
    }

    // Demo 7: Return Navigation Test
    println!("\n🔙 DEMO 7: Return Navigation with State Restoration");

    let nav_result = session
        .navigate_and_wait_reactive("https://www.google.com")
        .await?;
    println!("✅ Returned to Google!");
    println!("   ⏱️  Navigation took {}ms", nav_result.duration_ms);

    let final_element_count = session.get_highlighted_elements().len();
    println!(
        "   🎯 Re-highlighted {} elements on return",
        final_element_count
    );

    // Final Statistics
    println!("\n📈 === FINAL SHOWCASE STATISTICS ===");
    println!("🔍 All demonstrations used built-in package methods:");
    println!("   ✅ navigate_and_wait_reactive() - Advanced navigation detection");
    println!("   ✅ get_ai_elements() - AI-powered element analysis");
    println!("   ✅ type_with_refresh() - Dynamic DOM monitoring during interaction");
    println!("   ✅ wait_for_elements() - Smart element waiting");
    println!("   ✅ Auto-refresh system - Continuous background monitoring");
    println!("   ✅ Built-in highlighting - Real-time visual feedback");

    Ok(())
}

async fn run_automated_showcase(
    session: &mut DefaultSession,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🤖 === AUTOMATED SHOWCASE SEQUENCE ===");

    // Quick automated demo of key features
    let sites = vec![
        ("https://www.google.com", "Google Search"),
        ("https://www.github.com", "GitHub"),
        ("https://www.stackoverflow.com", "Stack Overflow"),
    ];

    for (i, (url, name)) in sites.iter().enumerate() {
        println!("\n--- Site {}: {} ---", i + 1, name);

        let nav_result = session.navigate_and_wait_reactive(url).await?;
        println!("✅ Navigated to {} ({}ms)", name, nav_result.duration_ms);

        let element_count = session.get_highlighted_elements().len();
        println!("🎯 Found {} interactive elements", element_count);

        let ai_elements = session.get_ai_elements().await?;
        let search_elements: Vec<_> = ai_elements
            .iter()
            .filter(|e| e.ai_instructions.to_lowercase().contains("search"))
            .collect();

        println!(
            "🔍 Detected {} search-related elements",
            search_elements.len()
        );

        if let Some(search_elem) = search_elements.first() {
            println!("⌨️  Testing search interaction...");
            let selector = search_elem.selector.clone(); // Clone to avoid borrowing issues
            session.type_with_refresh(&selector, "test query").await?;
            sleep(Duration::from_millis(1500)).await;
        }

        sleep(Duration::from_secs(1)).await;
    }

    println!("\n✅ Automated showcase completed!");
    Ok(())
}

async fn run_interactive_showcase(
    session: &mut DefaultSession,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🎮 === INTERACTIVE SHOWCASE ===");
    println!("Available commands:");
    println!("  'nav <url>' - Navigate using navigate_and_wait_reactive()");
    println!("  'analyze' - Run get_ai_elements() analysis");
    println!("  'type <num> <text>' - Type in element by number");
    println!("  'click <num>' - Click element by number with refresh");
    println!("  'wait <selector>' - Use wait_for_elements() with selector");
    println!("  'refresh' - Manually refresh highlights");
    println!("  'elements' - Show current interactive elements");
    println!("  'quit' - Exit");

    // Start at Google
    session
        .navigate_and_wait_reactive("https://www.google.com")
        .await?;

    loop {
        print!("\n🎯 Showcase> ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();
        let parts: Vec<&str> = input.split_whitespace().collect();

        if parts.is_empty() {
            continue;
        }

        match parts[0] {
            "quit" => break,
            "nav" => {
                if parts.len() > 1 {
                    let url = parts[1..].join(" ");
                    println!("🚀 Using navigate_and_wait_reactive()...");
                    match session.navigate_and_wait_reactive(&url).await {
                        Ok(result) => {
                            println!("✅ Navigation successful!");
                            println!("   Reason: {}", result.reason);
                            println!("   Duration: {}ms", result.duration_ms);
                            let element_count = session.get_highlighted_elements().len();
                            println!("   Elements auto-highlighted: {}", element_count);
                        }
                        Err(e) => println!("❌ Navigation failed: {}", e),
                    }
                }
            }
            "analyze" => {
                println!("🤖 Running AI element analysis...");
                match session.get_ai_elements().await {
                    Ok(elements) => {
                        println!("✅ Analyzed {} elements", elements.len());
                        for (i, elem) in elements.iter().take(10).enumerate() {
                            println!(
                                "  {}. #{}: {} - {}",
                                i + 1,
                                elem.element_number,
                                elem.element_type,
                                elem.description
                            );
                        }
                    }
                    Err(e) => println!("❌ Analysis failed: {}", e),
                }
            }
            "type" => {
                if parts.len() > 2 {
                    if let Ok(num) = parts[1].parse::<usize>() {
                        let text = parts[2..].join(" ");
                        println!(
                            "⌨️  Using type_in_element_by_number() on element #{}...",
                            num
                        );
                        match session.type_in_element_by_number(num, &text).await {
                            Ok(_) => {
                                println!("✅ Typed successfully!");
                                // Check for changes after typing
                                sleep(Duration::from_millis(500)).await;
                            }
                            Err(e) => println!("❌ Failed: {}", e),
                        }
                    }
                }
            }
            "click" => {
                if parts.len() > 1 {
                    if let Ok(num) = parts[1].parse::<usize>() {
                        println!(
                            "🖱️  Using click_element_by_number_with_refresh() on element #{}...",
                            num
                        );
                        match session.click_element_by_number_with_refresh(num).await {
                            Ok(_) => println!("✅ Clicked successfully with auto-refresh!"),
                            Err(e) => println!("❌ Failed: {}", e),
                        }
                    }
                }
            }
            "wait" => {
                if parts.len() > 1 {
                    let selector = parts[1..].join(" ");
                    println!(
                        "⏳ Using wait_for_elements() with selector '{}'...",
                        selector
                    );
                    match session.wait_for_elements(&selector, 5000).await {
                        Ok(found) => {
                            if found {
                                println!("✅ Elements found and highlighted!");
                            } else {
                                println!("⏰ Elements not found within timeout");
                            }
                        }
                        Err(e) => println!("❌ Wait failed: {}", e),
                    }
                }
            }
            "refresh" => {
                println!("🔄 Using highlight_interactive_elements()...");
                match session.highlight_interactive_elements().await {
                    Ok(highlights) => {
                        println!("✅ Refreshed {} element highlights", highlights.len());
                        show_element_distribution(session).await;
                    }
                    Err(e) => println!("❌ Refresh failed: {}", e),
                }
            }
            "elements" => {
                let elements = session.get_highlighted_elements();
                println!("🎯 Current {} interactive elements:", elements.len());
                for (i, elem) in elements.iter().take(15).enumerate() {
                    println!(
                        "  {}. #{}: {} ({})",
                        i + 1,
                        elem.element_number,
                        elem.element_type,
                        elem.color
                    );
                }
            }
            _ => {
                println!("❓ Unknown command. Type 'quit' to exit.");
            }
        }
    }

    Ok(())
}

async fn show_element_distribution(session: &mut DefaultSession) {
    let highlights = session.get_highlighted_elements().to_vec(); // Clone to avoid borrowing issues
    let mut type_counts = std::collections::HashMap::new();
    for highlight in highlights {
        *type_counts
            .entry(highlight.element_type.clone())
            .or_insert(0) += 1;
    }

    println!("   📊 Element type distribution:");
    for (element_type, count) in type_counts {
        println!("      {}: {}", element_type, count);
    }
}
