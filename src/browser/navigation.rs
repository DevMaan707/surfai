use crate::core::BrowserTrait;
use crate::errors::Result;
use std::time::{Duration, Instant};

pub struct NavigationManager;

impl NavigationManager {
    pub async fn wait_for_navigation_complete<B: BrowserTrait>(
        browser: &B,
        tab: &B::TabHandle,
        timeout_ms: u64,
    ) -> Result<NavigationResult> {
        let start_time = Instant::now();
        let timeout = Duration::from_millis(timeout_ms);

        // Multi-layered navigation detection
        let navigation_script = r#"
            (function() {
                return new Promise((resolve) => {
                    let resolved = false;
                    let networkIdle = false;
                    let domReady = false;
                    let loadComplete = false;

                    const resolveOnce = (reason) => {
                        if (!resolved) {
                            resolved = true;
                            resolve({
                                success: true,
                                reason: reason,
                                readyState: document.readyState,
                                url: window.location.href,
                                timestamp: Date.now()
                            });
                        }
                    };

                    // Check current state
                    if (document.readyState === 'complete') {
                        loadComplete = true;
                    }

                    // DOM Content Loaded
                    if (document.readyState === 'interactive' || document.readyState === 'complete') {
                        domReady = true;
                    }

                    // Network activity monitoring
                    let requestCount = 0;
                    let responseCount = 0;

                    // Override fetch for monitoring
                    const originalFetch = window.fetch;
                    window.fetch = function(...args) {
                        requestCount++;
                        return originalFetch.apply(this, args).then(response => {
                            responseCount++;
                            return response;
                        }).catch(error => {
                            responseCount++;
                            throw error;
                        });
                    };

                    // Monitor XMLHttpRequest
                    const originalXHROpen = XMLHttpRequest.prototype.open;
                    XMLHttpRequest.prototype.open = function(...args) {
                        requestCount++;
                        this.addEventListener('loadend', () => {
                            responseCount++;
                        });
                        return originalXHROpen.apply(this, args);
                    };

                    // Check for network idle
                    const checkNetworkIdle = () => {
                        if (requestCount === responseCount) {
                            networkIdle = true;
                            if (domReady && networkIdle) {
                                resolveOnce('network_idle_and_dom_ready');
                            }
                        }
                    };

                    // Event listeners
                    document.addEventListener('DOMContentLoaded', () => {
                        domReady = true;
                        if (networkIdle) {
                            resolveOnce('dom_content_loaded');
                        }
                    });

                    window.addEventListener('load', () => {
                        loadComplete = true;
                        setTimeout(() => {
                            checkNetworkIdle();
                            if (networkIdle || requestCount === 0) {
                                resolveOnce('window_load_complete');
                            }
                        }, 100);
                    });

                    // Periodic network check
                    const networkCheck = setInterval(() => {
                        checkNetworkIdle();
                        if (networkIdle && domReady) {
                            clearInterval(networkCheck);
                            resolveOnce('periodic_network_check');
                        }
                    }, 100);

                    // Fallback timeout
                    setTimeout(() => {
                        clearInterval(networkCheck);
                        resolveOnce('timeout_fallback');
                    }, 10000);

                    // Immediate check for already loaded pages
                    if (loadComplete || document.readyState === 'complete') {
                        setTimeout(() => {
                            checkNetworkIdle();
                            if (networkIdle || requestCount === 0) {
                                resolveOnce('already_loaded');
                            }
                        }, 50);
                    }
                });
            })()
        "#;

        // Execute navigation detection
        while start_time.elapsed() < timeout {
            match browser.execute_script(tab, navigation_script).await {
                Ok(result) => {
                    if let Some(obj) = result.as_object() {
                        if obj
                            .get("success")
                            .and_then(|v| v.as_bool())
                            .unwrap_or(false)
                        {
                            return Ok(NavigationResult {
                                success: true,
                                reason: obj
                                    .get("reason")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("unknown")
                                    .to_string(),
                                url: obj
                                    .get("url")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("")
                                    .to_string(),
                                ready_state: obj
                                    .get("readyState")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("")
                                    .to_string(),
                                duration_ms: start_time.elapsed().as_millis() as u64,
                            });
                        }
                    }
                }
                Err(_) => {
                    // Continue trying
                }
            }

            tokio::time::sleep(Duration::from_millis(50)).await;
        }

        Err(crate::errors::BrowserAgentError::TimeoutError(
            "Navigation timeout".to_string(),
        ))
    }
}

#[derive(Debug, Clone)]
pub struct NavigationResult {
    pub success: bool,
    pub reason: String,
    pub url: String,
    pub ready_state: String,
    pub duration_ms: u64,
}
