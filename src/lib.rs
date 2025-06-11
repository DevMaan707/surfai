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
pub use actions::{ActionRegistry, ActionResult};
pub use browser::{BrowserSession, ChromeBrowser};
pub use core::{BrowserTrait, Config, DomProcessorTrait, SessionTrait};
pub use dom::{DomElement, DomProcessor, DomState};
pub use errors::{BrowserAgentError, Result};

pub type DefaultBrowser = browser::ChromeBrowser;
pub type DefaultSession = browser::BrowserSession<DefaultBrowser>;
