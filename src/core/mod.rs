pub mod browser;
pub mod config;
pub mod dom;
pub mod session;

pub use browser::{BrowserCapabilities, BrowserTrait}; // Added BrowserCapabilities
pub use config::Config;
pub use dom::{DomProcessorTrait, ElementFilter, SelectorType}; // Added exports
pub use session::SessionTrait;
