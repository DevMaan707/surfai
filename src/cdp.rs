use crate::error::{BrowserError, BrowserResult};
use chromiumoxide::Page;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CDPScript {
    pub name: String,
    pub script: String,
    pub params: Option<Value>,
}

pub struct CDPExecutor {
    page: Page,
}

impl CDPExecutor {
    pub fn new(page: Page) -> Self {
        Self { page }
    }

    pub async fn execute_script(&self, script: &str) -> BrowserResult<Value> {
        let result = self
            .page
            .evaluate(script)
            .await
            .map_err(|e| BrowserError::Script(e.to_string()))?;

        let value = result
            .into_value()
            .map_err(|e| BrowserError::Script(e.to_string()))?;

        Ok(value)
    }

    pub async fn execute_cdp_script(&self, cdp_script: CDPScript) -> BrowserResult<Value> {
        let script = if let Some(params) = cdp_script.params {
            format!("(function(params) {{ {} }})({})", cdp_script.script, params)
        } else {
            format!("(function() {{ {} }})()", cdp_script.script)
        };

        self.execute_script(&script).await
    }

    pub async fn get_performance_metrics(&self) -> BrowserResult<Value> {
        let script = r#"
            (() => {
                return {
                    timing: performance.timing,
                    navigation: performance.navigation,
                    memory: performance.memory || {},
                    resources: performance.getEntriesByType('resource').slice(0, 10)
                };
            })();
        "#;

        self.execute_script(script).await
    }

    pub async fn execute_dom_script(
        &self,
        selector: &str,
        action: &str,
        value: Option<&str>,
    ) -> BrowserResult<Value> {
        let script = match action {
            "click" => format!("(() => {{ const el = document.querySelector('{}'); if (el) {{ el.click(); return true; }} return false; }})()", selector),
            "getValue" => format!("(() => {{ const el = document.querySelector('{}'); return el ? el.value : null; }})()", selector),
            "setText" => format!("(() => {{ const el = document.querySelector('{}'); if (el) {{ el.value = '{}'; return true; }} return false; }})()", selector, value.unwrap_or("")),
            "getAttribute" => format!("(() => {{ const el = document.querySelector('{}'); return el ? el.getAttribute('{}') : null; }})()", selector, value.unwrap_or("")),
            "innerHTML" => format!("(() => {{ const el = document.querySelector('{}'); return el ? el.innerHTML : null; }})()", selector),
            "setInnerHTML" => format!("(() => {{ const el = document.querySelector('{}'); if (el) {{ el.innerHTML = '{}'; return true; }} return false; }})()", selector, value.unwrap_or("")),
            _ => return Err(BrowserError::Script(format!("Unknown DOM action: {}", action))),
        };

        self.execute_script(&script).await
    }

    pub async fn setup_event_listeners(&self, events: Vec<&str>) -> BrowserResult<Value> {
        let events_json = serde_json::to_string(&events)?;
        let script = format!(
            r#"
            (() => {{
                const events = {};
                const eventLog = [];

                events.forEach(eventType => {{
                    document.addEventListener(eventType, function(e) {{
                        eventLog.push({{
                            type: eventType,
                            target: e.target.tagName,
                            timestamp: Date.now()
                        }});
                    }});
                }});

                window.getEventLog = function() {{
                    return eventLog;
                }};

                return {{ message: 'Event listeners setup', events: events }};
            }})();
        "#,
            events_json
        );

        self.execute_script(&script).await
    }
}
