use crate::error::{BrowserError, BrowserResult};
use chromiumoxide::{Browser, BrowserConfig, Page};
use futures::StreamExt;
use std::time::Duration;
use tokio::task::JoinHandle;

pub struct BrowserManager {
    browser: Browser,
    _handler: JoinHandle<()>,
}

impl BrowserManager {
    pub async fn new(headless: bool) -> BrowserResult<Self> {
        let mut config = BrowserConfig::builder();

        if !headless {
            config = config.with_head();
        }

        let (browser, mut handler) = Browser::launch(config.build().unwrap())
            .await
            .map_err(|e| BrowserError::Launch(e.to_string()))?;

        // In the task spawn section of all browser methods:
        let handle = tokio::task::spawn(async move {
            while let Some(event) = handler.next().await {
                match event {
                    Ok(_) => {}
                    Err(e) => {
                        let error_str = e.to_string();
                        // Filter out common harmless errors
                        if !error_str.contains("data did not match any variant")
                            && !error_str.contains("ResetWithoutClosingHandshake")
                            && !error_str.contains("Connection reset")
                        {
                            eprintln!("Browser event error: {:?}", e);
                        }
                    }
                }
            }
        });

        Ok(Self {
            browser,
            _handler: handle,
        })
    }
    pub async fn new_stealth_browser(headless: bool) -> BrowserResult<Self> {
        let args = vec![
            "--disable-blink-features=AutomationControlled",
            "--disable-infobars",
            "--disable-extensions",
            "--disable-plugins",
            "--disable-web-security",
            "--no-first-run",
            "--disable-default-apps",
            "--password-store=basic",
            "--use-mock-keychain",
            "--window-size=1366,768",
            "--user-agent=Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
            "--lang=en-US",
            "--accept-lang=en-US,en;q=0.9",
            "--memory-pressure-off",
            "--max_old_space_size=2048",
            "--js-flags=--max-old-space-size=2048",
            "--disable-background-timer-throttling",
            "--disable-renderer-backgrounding",
            "--disable-backgrounding-occluded-windows",
            "--disable-dev-shm-usage",
            "--no-sandbox",
            "--disable-gpu",
            "--disable-software-rasterizer",
            "--disable-hang-monitor",
            "--disable-prompt-on-repost",
            "--disable-sync",
            "--disable-translate",
            "--disable-domain-reliability",
            "--disable-component-extensions-with-background-pages",
            "--disable-background-networking",
            "--disable-client-side-phishing-detection",
            "--disable-popup-blocking",
            "--disable-notifications",
            "--disable-desktop-notifications",
            "--no-crash-upload",
            "--no-default-browser-check",
            "--no-pings",

            "--disable-features=TranslateUI,VizDisplayCompositor",
            "--disable-ipc-flooding-protection",
            "--aggressive-cache-discard",
            "--disable-backing-store-limit",

            "--ignore-certificate-errors",
            "--ignore-ssl-errors",
            "--ignore-certificate-errors-spki-list",
            "--disable-features=VizDisplayCompositor",
        ];

        let user_data_path = format!(
            "/tmp/browser_profile_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs()
        );

        let mut config = BrowserConfig::builder()
            .args(args)
            .user_data_dir(&user_data_path)
            .request_timeout(Duration::from_secs(30));

        if !headless {
            config = config.with_head();
        }

        let (browser, mut handler) = Browser::launch(config.build().unwrap())
            .await
            .map_err(|e| BrowserError::Launch(e.to_string()))?;
        let handle = tokio::task::spawn(async move {
            let mut consecutive_errors = 0;
            const MAX_CONSECUTIVE_ERRORS: u32 = 5;

            while let Some(event) = handler.next().await {
                match event {
                    Ok(_) => {
                        consecutive_errors = 0;
                    }
                    Err(e) => {
                        let error_str = e.to_string();
                        if !error_str.contains("data did not match any variant")
                            && !error_str.contains("ResetWithoutClosingHandshake")
                            && !error_str.contains("Connection reset")
                            && !error_str.contains("EOF")
                        {
                            consecutive_errors += 1;

                            if consecutive_errors >= MAX_CONSECUTIVE_ERRORS {
                                eprintln!(
                                    "Browser unstable - {} consecutive errors",
                                    consecutive_errors
                                );
                                break;
                            }
                        }
                    }
                }
            }
        });

        Ok(Self {
            browser,
            _handler: handle,
        })
    }

    pub async fn new_page(&self, url: &str) -> BrowserResult<Page> {
        let page = self
            .browser
            .new_page(url)
            .await
            .map_err(|e| BrowserError::Navigation(e.to_string()))?;

        tokio::time::timeout(Duration::from_secs(30), page.wait_for_navigation())
            .await
            .map_err(|_| BrowserError::Navigation("Navigation timeout".to_string()))?
            .map_err(|e| BrowserError::Navigation(e.to_string()))?;

        Ok(page)
    }

    pub async fn close(mut self) -> BrowserResult<()> {
        self.browser
            .close()
            .await
            .map_err(|e| BrowserError::Launch(e.to_string()))?;
        Ok(())
    }
}
