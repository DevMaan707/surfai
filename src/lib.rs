// src/lib.rs
//! Browser Agent Library
//!
//! A modular browser automation library designed for AI agents and automated testing.
//!
//! # Architecture
//!
//! - **Core**: Abstract traits and interfaces
//! - **Browser**: Browser implementation (Chrome, Firefox, etc.)
//! - **DOM**: DOM processing and state management
//! - **Actions**: Action registry and execution system
//! - **Utils**: Shared utilities
//! - **Errors**: Comprehensive error handling

pub mod actions;
pub mod browser;
pub mod core;
pub mod dom;
pub mod errors;
pub mod utils;

// Re-export commonly used types for convenience
pub use actions::{ActionRegistry, ActionResult};
pub use browser::{AIElement, BrowserSession, ChromeBrowser, LoginConfig, NavigationResult};
pub use core::{BrowserTrait, Config, DomProcessorTrait, SessionTrait};
pub use dom::{DomElement, DomProcessor, DomState};
pub use errors::{BrowserAgentError, Result};

// Type aliases for convenience
pub type DefaultBrowser = ChromeBrowser;
pub type DefaultSession = BrowserSession<ChromeBrowser>;
