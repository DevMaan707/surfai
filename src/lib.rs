pub mod browser;
pub mod cdp;
pub mod elements;
pub mod error;
pub mod human;
pub mod mcp;
pub mod scraper;
pub use browser::BrowserManager;
pub use cdp::{CDPExecutor, CDPScript};
pub use elements::{AllElement, ElementInteractor, InteractableElement, LabeledElement};
pub use error::{BrowserError, BrowserResult};
pub use human::HumanInteractor;
pub use scraper::{FormData, PageData, PageScraper};

pub use chromiumoxide::{Browser, BrowserConfig, Page};
pub use mcp::*;
