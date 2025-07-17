# Surf-New: Advanced Browser Automation Framework

A powerful Rust-based browser automation framework with human-like behavior simulation and stealth capabilities. Built on top of `chromiumoxide` with advanced scripting, element interaction, and MCP (Model Context Protocol) server integration.

## Features

### 🚀 Core Capabilities
- **Stealth Browser Management**: Anti-detection capabilities with realistic browser fingerprinting
- **Human-like Interactions**: Natural mouse movements, typing patterns, and browsing behavior
- **Advanced Element Detection**: Intelligent element labeling and interaction systems
- **CDP Script Execution**: Custom Chrome DevTools Protocol script execution
- **Page Scraping**: Comprehensive data extraction from web pages
- **MCP Server Integration**: Model Context Protocol server for AI agent integration

### 🛡️ Stealth Features
- User agent spoofing and browser fingerprint masking
- Realistic timing patterns and random delays
- CAPTCHA detection and avoidance
- Session management with cookies and browsing history simulation
- Mouse movement with natural curves and variations

### 🎯 Element Interaction
- Smart element detection and labeling
- Google Search automation with human-like patterns
- Form filling and button clicking
- Screenshot capture and visual debugging
- DOM manipulation and event simulation

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
surf-new = "0.1.0"
tokio = { version = "1.0", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

## Quick Start

### Basic Browser Automation

```rust
use surf_new::*;

#[tokio::main]
async fn main() -> Result> {
    // Create a new browser instance
    let browser = BrowserManager::new(false).await?;
    let page = browser.new_page("https://google.com").await?;

    // Initialize components
    let scraper = PageScraper::new(page.clone());
    let interactor = ElementInteractor::new(page.clone());

    // Scrape page data
    let page_data = scraper.scrape_page().await?;
    println!("Page title: {}", page_data.title);

    // Find interactive elements
    let elements = interactor.find_interactable_elements().await?;
    println!("Found {} interactive elements", elements.len());

    // Take screenshot
    let screenshot = scraper.take_screenshot().await?;
    std::fs::write("screenshot.png", screenshot)?;

    browser.close().await?;
    Ok(())
}
```

### Human-like Google Search

```rust
use surf_new::*;

#[tokio::main]
async fn main() -> Result> {
    // Create stealth browser
    let browser = BrowserManager::new_stealth_browser(false).await?;
    let page = browser.new_page("https://google.com").await?;

    let human = HumanInteractor::new(page.clone());

    // Apply stealth settings
    human.apply_stealth().await?;
    human.setup_session().await?;

    // Perform human-like search
    let search_query = "rust programming language";
    human.search_with_realistic_pattern(search_query).await?;
    human.submit_search().await?;

    // Wait for results and extract
    human.wait_for_results(30).await?;
    let results = human.extract_search_results().await?;

    println!("Found {} results", results.len());
    for (i, result) in results.iter().enumerate().take(5) {
        println!("{}. {}", i + 1, result.get("title").unwrap_or(&"No title".to_string()));
        println!("   Link: {}", result.get("link").unwrap_or(&"No link".to_string()));
    }

    browser.close().await?;
    Ok(())
}
```

## MCP Server Integration

The framework includes a built-in MCP (Model Context Protocol) server for seamless AI agent integration.

### Starting the MCP Server

```rust
use surf_new::mcp::MCPBrowserServer;

#[tokio::main]
async fn main() -> Result> {
    let server = MCPBrowserServer::new();

    // Handle MCP requests
    let request = MCPRequest {
        id: "1".to_string(),
        method: "tools/list".to_string(),
        params: serde_json::json!({}),
    };

    let response = server.handle_request(request).await;
    println!("MCP Response: {:?}", response);

    Ok(())
}
```

### Available MCP Tools

The MCP server provides the following tools for AI agents:

#### 1. `create_browser_session`
Creates a new browser session with optional stealth mode.

**Parameters:**
- `headless` (boolean, default: true): Run browser in headless mode
- `stealth` (boolean, default: false): Enable stealth anti-detection features

**Example:**
```json
{
  "name": "create_browser_session",
  "arguments": {
    "headless": false,
    "stealth": true
  }
}
```

#### 2. `navigate_to_url`
Navigate to a specific URL in an existing session.

**Parameters:**
- `session_id` (string, required): Browser session ID
- `url` (string, required): Target URL

**Example:**
```json
{
  "name": "navigate_to_url",
  "arguments": {
    "session_id": "uuid-session-id",
    "url": "https://google.com"
  }
}
```

#### 3. `search_google`
Perform a Google search with human-like behavior patterns.

**Parameters:**
- `session_id` (string, required): Browser session ID
- `query` (string, required): Search query

**Example:**
```json
{
  "name": "search_google",
  "arguments": {
    "session_id": "uuid-session-id",
    "query": "rust programming language"
  }
}
```

#### 4. `get_page_elements`
Extract all interactable elements from the current page.

**Parameters:**
- `session_id` (string, required): Browser session ID
- `element_types` (array, optional): Specific element types to filter

**Example:**
```json
{
  "name": "get_page_elements",
  "arguments": {
    "session_id": "uuid-session-id",
    "element_types": ["input", "button", "a"]
  }
}
```

#### 5. `click_element`
Click on an element with human-like mouse movements.

**Parameters:**
- `session_id` (string, required): Browser session ID
- `selector` (string, required): CSS selector for the element

**Example:**
```json
{
  "name": "click_element",
  "arguments": {
    "session_id": "uuid-session-id",
    "selector": "button[type='submit']"
  }
}
```

#### 6. `type_text`
Type text into an element with realistic human typing patterns.

**Parameters:**
- `session_id` (string, required): Browser session ID
- `selector` (string, required): CSS selector for the input element
- `text` (string, required): Text to type

**Example:**
```json
{
  "name": "type_text",
  "arguments": {
    "session_id": "uuid-session-id",
    "selector": "input[name='q']",
    "text": "Hello world"
  }
}
```

#### 7. `scrape_page`
Extract all data from the current page including title, content, links, images, and forms.

**Parameters:**
- `session_id` (string, required): Browser session ID

**Example:**
```json
{
  "name": "scrape_page",
  "arguments": {
    "session_id": "uuid-session-id"
  }
}
```

#### 8. `take_screenshot`
Capture a screenshot of the current page.

**Parameters:**
- `session_id` (string, required): Browser session ID

**Example:**
```json
{
  "name": "take_screenshot",
  "arguments": {
    "session_id": "uuid-session-id"
  }
}
```

#### 9. `close_session`
Close a browser session and clean up resources.

**Parameters:**
- `session_id` (string, required): Browser session ID

**Example:**
```json
{
  "name": "close_session",
  "arguments": {
    "session_id": "uuid-session-id"
  }
}
```

### MCP Usage Flow

1. **Create Session**: Use `create_browser_session` to initialize a browser
2. **Navigate**: Use `navigate_to_url` to go to target websites
3. **Interact**: Use `click_element`, `type_text`, or `search_google` for interactions
4. **Extract Data**: Use `scrape_page`, `get_page_elements`, or `take_screenshot` for data extraction
5. **Cleanup**: Use `close_session` to properly close the browser

### MCP Integration Example

```rust
use surf_new::mcp::*;

async fn ai_agent_workflow() -> Result> {
    let server = MCPBrowserServer::new();

    // Step 1: Create browser session
    let create_request = MCPRequest {
        id: "1".to_string(),
        method: "tools/call".to_string(),
        params: serde_json::json!({
            "name": "create_browser_session",
            "arguments": {
                "headless": false,
                "stealth": true
            }
        }),
    };

    let response = server.handle_request(create_request).await;
    let session_id = response.result.unwrap()["session_id"].as_str().unwrap();

    // Step 2: Perform Google search
    let search_request = MCPRequest {
        id: "2".to_string(),
        method: "tools/call".to_string(),
        params: serde_json::json!({
            "name": "search_google",
            "arguments": {
                "session_id": session_id,
                "query": "AI automation tools"
            }
        }),
    };

    let search_response = server.handle_request(search_request).await;
    println!("Search results: {:?}", search_response.result);

    // Step 3: Take screenshot
    let screenshot_request = MCPRequest {
        id: "3".to_string(),
        method: "tools/call".to_string(),
        params: serde_json::json!({
            "name": "take_screenshot",
            "arguments": {
                "session_id": session_id
            }
        }),
    };

    let screenshot_response = server.handle_request(screenshot_request).await;

    // Step 4: Close session
    let close_request = MCPRequest {
        id: "4".to_string(),
        method: "tools/call".to_string(),
        params: serde_json::json!({
            "name": "close_session",
            "arguments": {
                "session_id": session_id
            }
        }),
    };

    server.handle_request(close_request).await;

    Ok(())
}
```

## API Reference

### Core Components

#### `BrowserManager`
Main browser management interface.

**Methods:**
- `new(headless: bool) -> BrowserResult`: Create standard browser
- `new_stealth_browser(headless: bool) -> BrowserResult`: Create stealth browser
- `new_page(url: &str) -> BrowserResult`: Create new page
- `close(self) -> BrowserResult`: Close browser

#### `HumanInteractor`
Human-like interaction simulation.

**Key Methods:**
- `apply_stealth() -> BrowserResult`: Apply stealth settings
- `type_human(selector: &str, text: &str) -> BrowserResult`: Human-like typing
- `click_human(selector: &str) -> BrowserResult`: Human-like clicking
- `search_with_realistic_pattern(query: &str) -> BrowserResult`: Realistic search
- `extract_search_results() -> BrowserResult>>`: Extract results

#### `ElementInteractor`
Element detection and interaction.

**Key Methods:**
- `get_all_elements() -> BrowserResult>`: Get all elements
- `get_interactable_elements() -> BrowserResult>`: Get interactive elements
- `label_elements(tags: Option>) -> BrowserResult>`: Label elements
- `click_element(selector: &str) -> BrowserResult`: Click element
- `type_text(selector: &str, text: &str) -> BrowserResult`: Type text

#### `PageScraper`
Web page data extraction.

**Key Methods:**
- `scrape_page() -> BrowserResult`: Scrape all page data
- `take_screenshot() -> BrowserResult>`: Take screenshot
- `extract_links() -> BrowserResult>`: Extract links
- `extract_images() -> BrowserResult>`: Extract images

## Configuration

### Stealth Browser Settings

The stealth browser includes comprehensive anti-detection features:

```rust
let browser = BrowserManager::new_stealth_browser(false).await?;
```

**Stealth Features:**
- User agent spoofing
- Plugin and extension masking
- Canvas fingerprinting protection
- WebGL fingerprinting protection
- Timezone and language spoofing
- Screen resolution normalization

### Human Behavior Patterns

Customize human-like behavior:

```rust
let human = HumanInteractor::new(page);

// Configure timing patterns
human.random_delay(1000, 2000).await;  // Random delay between 1-2 seconds
human.simulate_reading_delay("Sample text").await;  // Reading-based delay
human.simulate_hesitation().await;  // Simulate user hesitation
```

## Examples

The `examples/` directory contains comprehensive usage examples:

- `basic_usage.rs`: Basic browser automation
- `google_search.rs`: Google search automation
- `human_google_search.rs`: Human-like Google search
- `label_elements.rs`: Element labeling and interaction

Run examples with:
```bash
cargo run --example basic_usage
cargo run --example google_search
cargo run --example human_google_search
```

## Error Handling

The framework uses a comprehensive error system:

```rust
use surf_new::BrowserResult;

match browser.new_page("https://example.com").await {
    Ok(page) => {
        // Success
    }
    Err(BrowserError::Navigation(msg)) => {
        println!("Navigation error: {}", msg);
    }
    Err(BrowserError::ElementNotFound(selector)) => {
        println!("Element not found: {}", selector);
    }
    Err(e) => {
        println!("Other error: {}", e);
    }
}
```

## Requirements

- Rust 1.70+
- Chrome or Chromium browser installed
- tokio runtime
- Network access for web automation

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests for new functionality
5. Submit a pull request



## Support

For issues and questions:
- Create an issue on GitHub
- Check the examples directory for usage patterns
- Review the API documentation

**Note**: This framework is designed for legitimate automation and testing purposes. Please respect website terms of service and implement appropriate rate limiting and error handling in production use.
