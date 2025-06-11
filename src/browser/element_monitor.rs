use crate::core::BrowserTrait;
use crate::errors::Result;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct ElementMonitor {
    is_monitoring: Arc<RwLock<bool>>,
    observer_active: Arc<RwLock<bool>>,
}

impl ElementMonitor {
    pub fn new() -> Self {
        Self {
            is_monitoring: Arc::new(RwLock::new(false)),
            observer_active: Arc::new(RwLock::new(false)),
        }
    }

    /// Start monitoring DOM changes with mutation observer
    pub async fn start_monitoring<B: BrowserTrait>(
        &self,
        browser: &B,
        tab: &B::TabHandle,
    ) -> Result<()> {
        let mut monitoring = self.is_monitoring.write().await;
        if *monitoring {
            return Ok(());
        }

        let observer_script = r#"
            (function() {
                // Remove existing observer if any
                if (window.browserAgentObserver) {
                    window.browserAgentObserver.disconnect();
                }

                // Track changes
                window.browserAgentChanges = {
                    hasChanges: false,
                    changeCount: 0,
                    lastChangeTime: Date.now(),
                    changeTypes: []
                };

                // Create mutation observer
                window.browserAgentObserver = new MutationObserver((mutations) => {
                    let significantChange = false;
                    let changeTypes = [];

                    mutations.forEach((mutation) => {
                        // Track different types of changes
                        if (mutation.type === 'childList') {
                            if (mutation.addedNodes.length > 0 || mutation.removedNodes.length > 0) {
                                // Check if added/removed nodes are interactive
                                const hasInteractiveNodes = Array.from(mutation.addedNodes).some(node => {
                                    if (node.nodeType !== 1) return false; // Element nodes only
                                    const tagName = node.tagName?.toLowerCase();
                                    return tagName && ['input', 'button', 'select', 'textarea', 'a', 'form'].includes(tagName);
                                }) || Array.from(mutation.removedNodes).some(node => {
                                    if (node.nodeType !== 1) return false;
                                    const tagName = node.tagName?.toLowerCase();
                                    return tagName && ['input', 'button', 'select', 'textarea', 'a', 'form'].includes(tagName);
                                });

                                if (hasInteractiveNodes) {
                                    significantChange = true;
                                    changeTypes.push('interactive_elements');
                                }

                                // Check for dropdown/suggestion elements
                                const hasDropdownElements = Array.from(mutation.addedNodes).some(node => {
                                    if (node.nodeType !== 1) return false;
                                    const className = node.className || '';
                                    const id = node.id || '';
                                    return className.toLowerCase().includes('dropdown') ||
                                           className.toLowerCase().includes('suggestion') ||
                                           className.toLowerCase().includes('autocomplete') ||
                                           className.toLowerCase().includes('menu') ||
                                           id.toLowerCase().includes('dropdown') ||
                                           id.toLowerCase().includes('suggestion');
                                });

                                if (hasDropdownElements) {
                                    significantChange = true;
                                    changeTypes.push('dropdown_suggestions');
                                }
                            }
                        } else if (mutation.type === 'attributes') {
                            // Track attribute changes that might affect interactivity
                            const attributeName = mutation.attributeName;
                            if (['class', 'style', 'disabled', 'hidden', 'aria-expanded', 'aria-hidden'].includes(attributeName)) {
                                significantChange = true;
                                changeTypes.push('visibility_changes');
                            }
                        }
                    });

                    if (significantChange) {
                        window.browserAgentChanges.hasChanges = true;
                        window.browserAgentChanges.changeCount++;
                        window.browserAgentChanges.lastChangeTime = Date.now();
                        window.browserAgentChanges.changeTypes = [...new Set([...window.browserAgentChanges.changeTypes, ...changeTypes])];

                        // Dispatch custom event
                        window.dispatchEvent(new CustomEvent('browserAgentDOMChange', {
                            detail: {
                                changeTypes: changeTypes,
                                timestamp: Date.now()
                            }
                        }));
                    }
                });

                // Start observing
                window.browserAgentObserver.observe(document.body, {
                    childList: true,
                    subtree: true,
                    attributes: true,
                    attributeFilter: ['class', 'style', 'disabled', 'hidden', 'aria-expanded', 'aria-hidden']
                });

                return { success: true, message: 'DOM monitoring started' };
            })()
        "#;

        browser.execute_script(tab, observer_script).await?;
        *monitoring = true;
        *self.observer_active.write().await = true;

        println!("✅ DOM monitoring started");
        Ok(())
    }

    /// Check if DOM has changed since last check
    pub async fn check_for_changes<B: BrowserTrait>(
        &self,
        browser: &B,
        tab: &B::TabHandle,
    ) -> Result<DOMChangeResult> {
        let check_script = r#"
            (function() {
                if (!window.browserAgentChanges) {
                    return { hasChanges: false, reason: 'monitor_not_active' };
                }

                const changes = window.browserAgentChanges;
                const result = {
                    hasChanges: changes.hasChanges,
                    changeCount: changes.changeCount,
                    lastChangeTime: changes.lastChangeTime,
                    changeTypes: changes.changeTypes,
                    timeSinceLastChange: Date.now() - changes.lastChangeTime
                };

                // Reset the flag
                window.browserAgentChanges.hasChanges = false;
                window.browserAgentChanges.changeTypes = [];

                return result;
            })()
        "#;

        let result = browser.execute_script(tab, check_script).await?;
        Ok(serde_json::from_value(result)?)
    }

    /// Wait for DOM changes with timeout
    pub async fn wait_for_changes<B: BrowserTrait>(
        &self,
        browser: &B,
        tab: &B::TabHandle,
        timeout_ms: u64,
    ) -> Result<DOMChangeResult> {
        let wait_script = format!(
            r#"
            (function() {{
                return new Promise((resolve) => {{
                    let resolved = false;

                    const resolveOnce = (result) => {{
                        if (!resolved) {{
                            resolved = true;
                            resolve(result);
                        }}
                    }};

                    // Check if changes already exist
                    if (window.browserAgentChanges && window.browserAgentChanges.hasChanges) {{
                        const changes = window.browserAgentChanges;
                        resolveOnce({{
                            hasChanges: true,
                            changeCount: changes.changeCount,
                            changeTypes: changes.changeTypes,
                            reason: 'immediate'
                        }});
                        return;
                    }}

                    // Listen for changes
                    const changeHandler = (event) => {{
                        resolveOnce({{
                            hasChanges: true,
                            changeTypes: event.detail.changeTypes,
                            timestamp: event.detail.timestamp,
                            reason: 'event_triggered'
                        }});
                    }};

                    window.addEventListener('browserAgentDOMChange', changeHandler);

                    // Timeout
                    setTimeout(() => {{
                        window.removeEventListener('browserAgentDOMChange', changeHandler);
                        resolveOnce({{
                            hasChanges: false,
                            reason: 'timeout'
                        }});
                    }}, {});
                }});
            }})()
        "#,
            timeout_ms
        );

        let result = browser.execute_script(tab, &wait_script).await?;
        Ok(serde_json::from_value(result)?)
    }

    pub async fn stop_monitoring<B: BrowserTrait>(
        &self,
        browser: &B,
        tab: &B::TabHandle,
    ) -> Result<()> {
        let stop_script = r#"
            (function() {
                if (window.browserAgentObserver) {
                    window.browserAgentObserver.disconnect();
                    delete window.browserAgentObserver;
                }
                if (window.browserAgentChanges) {
                    delete window.browserAgentChanges;
                }
                return { success: true };
            })()
        "#;

        browser.execute_script(tab, stop_script).await?;
        *self.is_monitoring.write().await = false;
        *self.observer_active.write().await = false;

        println!("✅ DOM monitoring stopped");
        Ok(())
    }
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct DOMChangeResult {
    pub has_changes: bool,
    pub change_count: Option<u32>,
    pub change_types: Option<Vec<String>>,
    pub last_change_time: Option<u64>,
    pub time_since_last_change: Option<u64>,
    pub reason: Option<String>,
}
