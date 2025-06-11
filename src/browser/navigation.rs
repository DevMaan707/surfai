use crate::core::BrowserTrait;
use crate::errors::Result;
use std::time::Instant;

pub struct NavigationManager;

impl NavigationManager {
    pub async fn wait_for_navigation_complete<B: BrowserTrait>(
        browser: &B,
        tab: &B::TabHandle,
        timeout_ms: u64,
    ) -> Result<NavigationResult> {
        let start_time = Instant::now();

        // Dynamic, event-driven navigation detection
        let navigation_script = r#"
            (function() {
                return new Promise((resolve) => {
                    let resolved = false;
                    let startTime = Date.now();

                    const resolveOnce = (reason, additionalData = {}) => {
                        if (!resolved) {
                            resolved = true;
                            resolve({
                                success: true,
                                reason: reason,
                                readyState: document.readyState,
                                url: window.location.href,
                                timestamp: Date.now(),
                                loadTime: Date.now() - startTime,
                                ...additionalData
                            });
                        }
                    };

                    // Immediate check - if page is already complete
                    if (document.readyState === 'complete') {
                        // Double-check that resources are actually loaded
                        if (document.body && document.body.children.length > 0) {
                            resolveOnce('already_complete');
                            return;
                        }
                    }

                    // Track network activity
                    let pendingRequests = 0;
                    let networkQuiet = false;
                    let domReady = false;
                    let imagesLoaded = false;

                    // Monitor document ready state changes
                    const checkReadyState = () => {
                        if (document.readyState === 'interactive') {
                            domReady = true;
                            checkAllConditions('dom_interactive');
                        } else if (document.readyState === 'complete') {
                            domReady = true;
                            checkAllConditions('dom_complete');
                        }
                    };

                    // Monitor network requests
                    const checkNetworkQuiet = () => {
                        if (pendingRequests === 0) {
                            networkQuiet = true;
                            checkAllConditions('network_quiet');
                        }
                    };

                    // Monitor image loading
                    const checkImagesLoaded = () => {
                        const images = document.querySelectorAll('img');
                        let loadedImages = 0;

                        if (images.length === 0) {
                            imagesLoaded = true;
                            checkAllConditions('no_images');
                            return;
                        }

                        images.forEach(img => {
                            if (img.complete || img.naturalHeight > 0) {
                                loadedImages++;
                            }
                        });

                        if (loadedImages === images.length) {
                            imagesLoaded = true;
                            checkAllConditions('images_loaded');
                        }
                    };

                    // Check if all conditions are met
                    const checkAllConditions = (trigger) => {
                        // Primary condition: DOM is ready
                        if (domReady) {
                            // If we have basic content, we can proceed
                            if (document.body && document.body.children.length > 0) {
                                // For simple pages or when network is quiet, resolve immediately
                                if (networkQuiet || document.readyState === 'complete') {
                                    resolveOnce(`complete_${trigger}`, {
                                        trigger,
                                        hasContent: true,
                                        networkQuiet,
                                        imagesLoaded
                                    });
                                    return;
                                }

                                // For complex pages, wait a bit more but don't block indefinitely
                                setTimeout(() => {
                                    if (!resolved) {
                                        resolveOnce(`timeout_${trigger}`, {
                                            trigger,
                                            hasContent: true,
                                            networkQuiet,
                                            imagesLoaded
                                        });
                                    }
                                }, 1000);
                            }
                        }
                    };

                    // Set up event listeners
                    document.addEventListener('readystatechange', checkReadyState);
                    document.addEventListener('DOMContentLoaded', () => {
                        domReady = true;
                        checkAllConditions('dom_content_loaded');
                    });

                    window.addEventListener('load', () => {
                        domReady = true;
                        imagesLoaded = true;
                        networkQuiet = true;
                        resolveOnce('window_load', {
                            trigger: 'window_load',
                            hasContent: document.body && document.body.children.length > 0,
                            networkQuiet: true,
                            imagesLoaded: true
                        });
                    });

                    // Monitor fetch requests
                    const originalFetch = window.fetch;
                    window.fetch = function(...args) {
                        pendingRequests++;
                        return originalFetch.apply(this, args)
                            .finally(() => {
                                pendingRequests--;
                                setTimeout(checkNetworkQuiet, 100);
                            });
                    };

                    // Monitor XHR requests
                    const originalOpen = XMLHttpRequest.prototype.open;
                    XMLHttpRequest.prototype.open = function(...args) {
                        pendingRequests++;
                        this.addEventListener('loadend', () => {
                            pendingRequests--;
                            setTimeout(checkNetworkQuiet, 100);
                        });
                        return originalOpen.apply(this, args);
                    };

                    // Initial checks
                    checkReadyState();

                    // Check images after a short delay to let them start loading
                    setTimeout(checkImagesLoaded, 200);

                    // Absolute fallback - never wait more than reasonable time
                    setTimeout(() => {
                        if (!resolved) {
                            resolveOnce('absolute_fallback', {
                                trigger: 'timeout',
                                hasContent: document.body && document.body.children.length > 0,
                                networkQuiet,
                                imagesLoaded,
                                finalReadyState: document.readyState
                            });
                        }
                    }, 8000);
                });
            })()
        "#;

        // Execute the dynamic navigation detection
        let result = browser.execute_script(tab, navigation_script).await?;

        if let Some(obj) = result.as_object() {
            if obj
                .get("success")
                .and_then(|v| v.as_bool())
                .unwrap_or(false)
            {
                let reason = obj
                    .get("reason")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown")
                    .to_string();
                let url = obj
                    .get("url")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let ready_state = obj
                    .get("readyState")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let load_time = obj.get("loadTime").and_then(|v| v.as_u64()).unwrap_or(0);

                return Ok(NavigationResult {
                    success: true,
                    reason,
                    url,
                    ready_state,
                    duration_ms: start_time.elapsed().as_millis() as u64,
                    actual_load_time: load_time,
                    network_quiet: obj
                        .get("networkQuiet")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false),
                    has_content: obj
                        .get("hasContent")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false),
                });
            }
        }

        // If script execution failed, use minimal fallback
        Self::minimal_fallback(browser, tab).await
    }

    async fn minimal_fallback<B: BrowserTrait>(
        browser: &B,
        tab: &B::TabHandle,
    ) -> Result<NavigationResult> {
        let start_time = Instant::now();

        // Just check if we can get URL and basic page info
        let url = browser.get_url(tab).await.unwrap_or_default();

        if !url.is_empty() && !url.starts_with("about:") {
            Ok(NavigationResult {
                success: true,
                reason: "fallback_url_available".to_string(),
                url,
                ready_state: "unknown".to_string(),
                duration_ms: start_time.elapsed().as_millis() as u64,
                actual_load_time: 0,
                network_quiet: false,
                has_content: false,
            })
        } else {
            Err(crate::errors::BrowserAgentError::NavigationFailed(
                "Could not verify navigation success".to_string(),
            ))
        }
    }
}

#[derive(Debug, Clone)]
pub struct NavigationResult {
    pub success: bool,
    pub reason: String,
    pub url: String,
    pub ready_state: String,
    pub duration_ms: u64,
    pub actual_load_time: u64,
    pub network_quiet: bool,
    pub has_content: bool,
}

impl NavigationResult {
    pub fn is_fast_load(&self) -> bool {
        self.actual_load_time < 1000
    }

    pub fn is_complete_load(&self) -> bool {
        self.network_quiet && self.has_content && self.ready_state == "complete"
    }

    pub fn load_quality(&self) -> &str {
        match (
            self.has_content,
            self.network_quiet,
            self.ready_state.as_str(),
        ) {
            (true, true, "complete") => "excellent",
            (true, true, _) => "good",
            (true, false, _) => "partial",
            (false, _, _) => "minimal",
        }
    }
}
