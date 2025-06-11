use clap::{Arg, Command};
use surfai::{BrowserSession, SessionTrait};
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = Command::new("Dynamic Navigation Showcase")
        .version("1.0")
        .about("Showcases truly dynamic navigation detection")
        .arg(
            Arg::new("headless")
                .long("headless")
                .help("Run browser in headless mode")
                .action(clap::ArgAction::SetTrue),
        )
        .get_matches();

    let headless = matches.get_flag("headless");

    println!("⚡ Dynamic Navigation Showcase - Truly Responsive Detection");
    println!("🔧 Headless: {}", headless);

    let mut session = if headless {
        BrowserSession::quick_start().await?
    } else {
        BrowserSession::demo_mode().await?
    };
    let test_sites = vec![
        ("https://www.google.com", "Google (Fast Loading)"),
        ("https://www.github.com", "GitHub (Medium Loading)"),
        ("https://www.wikipedia.org", "Wikipedia (Content Heavy)"),
    ];

    println!("\n🎯 Testing Dynamic Navigation Detection on Various Sites");
    println!("Each navigation will complete as soon as the page is truly ready - no waiting!");

    for (i, (url, description)) in test_sites.iter().enumerate() {
        println!("\n--- Test {}: {} ---", i + 1, description);

        let start_time = std::time::Instant::now();

        match session.navigate_smart(url).await {
            Ok(result) => {
                let total_time = start_time.elapsed().as_millis();

                println!("✅ Navigation Success!");
                println!("   🚀 Total Time: {}ms", total_time);
                println!("   ⚡ Page Load Time: {}ms", result.actual_load_time);

                if result.is_fast_load() {
                    println!("   ⚡ FAST LOAD detected!");
                }

                if result.is_complete_load() {
                    println!("   🎉 COMPLETE LOAD detected!");
                }

                let element_count = session.get_highlighted_elements().len();
                println!("   🎯 Interactive Elements: {}", element_count);
                let efficiency = if result.actual_load_time > 0 {
                    (result.actual_load_time as f64 / total_time as f64) * 100.0
                } else {
                    100.0
                };
                println!("   📈 Detection Efficiency: {:.1}%", efficiency);
            }
            Err(e) => {
                let total_time = start_time.elapsed().as_millis();
                println!("❌ Navigation Failed after {}ms: {}", total_time, e);
            }
        }
        sleep(Duration::from_millis(500)).await;
    }
    session.close().await?;
    println!("👋 Dynamic navigation showcase completed!");

    Ok(())
}
