# surf-ai 🏄‍♂️

A modern, AI-enhanced browser automation framework built in Rust. Designed for AI agents, automated testing, and web scraping with intelligent element detection and interaction.

## 🌟 Features

- **Intelligent Navigation**: Dynamic page load detection and smart waiting strategies
- **AI-Enhanced Element Detection**: Auto-labeling and classification of web elements
- **Reactive DOM Monitoring**: Real-time tracking of dynamic page changes
- **Session Management**: Sophisticated cookie and state management
- **Smart Interactions**: Context-aware clicking and typing with automatic retries
- **Cross-Browser Support**: Modular design for multiple browser implementations
- **Screenshot & Visual Analysis**: Advanced screenshot capabilities with comparison

## 🚀 Quick Start

```rust
use surf_ai::{BrowserSession, SessionTrait};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a demo session with visual feedback
    let mut session = BrowserSession::demo_mode().await?;

    // Smart navigation with dynamic completion detection
    session.navigate_smart("https://www.example.com").await?;

    // Get AI-analyzed elements
    let elements = session.get_ai_elements().await?;

    // Interact with elements using AI-generated selectors
    session.type_with_refresh(&elements[0].selector, "Hello world").await?;

    session.close().await?;
    Ok(())
}
```

## 🔧 Installation

Add to your Cargo.toml:
```toml
[dependencies]
surf-ai = "0.1.0"
```

## 🎯 Use Cases

1. **AI Agents**: Perfect for AI-powered web automation
2. **Web Testing**: Intelligent element detection and interaction
3. **Web Scraping**: Smart navigation and state management
4. **Automated Workflows**: Session management and cookie handling
5. **Visual Testing**: Screenshot comparison and analysis

## 🔬 Development Status

Currently in active development. Key areas of focus:

- [ ] Enhanced AI element analysis
- [ ] More browser implementations
- [ ] Advanced session management
- [ ] Performance optimizations
- [ ] Extended action registry
- [ ] Enhanced visual analysis tools

## 🛠️ Examples

Check out the examples folder for demonstrations:

- `dynamic_monitoring_demo.rs`: Shows reactive DOM monitoring
- `google_search_demo.rs`: Demonstrates AI-enhanced interactions

## 🎯 Future Scope

1. **Advanced AI Integration**
   - Natural language processing for element interaction
   - Learning from user interactions
   - Predictive navigation

2. **Enhanced Browser Support**
   - Firefox implementation
   - Safari implementation
   - Mobile browser support

3. **Performance & Reliability**
   - Parallel execution support
   - Enhanced error recovery
   - Performance profiling

4. **Developer Tools**
   - Visual element inspector
   - Session recording/replay
   - Advanced debugging tools

## 🤝 Contributing

Contributions are welcome! Please check out our contributing guidelines.

## 📝 License

This project is licensed under the MIT License - see the LICENSE file for details.

## 👤 Author

DevMaan707 - [GitHub Profile](https://github.com/DevMaan707)

## 🙏 Acknowledgments

Special thanks to the Rust community and contributors to the core technologies used in this project.
