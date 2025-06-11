use crate::core::{BrowserTrait, Config, DomProcessorTrait, SessionTrait};
use crate::dom::{DomProcessor, DomState};
use crate::errors::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

use super::element_monitor::ElementMonitor;
use super::navigation::{NavigationManager, NavigationResult};

pub struct BrowserSession<B: BrowserTrait> {
    browser: Arc<B>,
    tab: Option<B::TabHandle>,
    dom_processor: DomProcessor,
    config: Config,
    element_highlights: Vec<ElementHighlight>,
    element_monitor: ElementMonitor,
    auto_refresh_enabled: bool,
    session_id: String,
    current_session_data: Option<SessionData>,
}

#[derive(Debug, Clone)]
pub struct ElementHighlight {
    pub element_id: String,
    pub element_number: usize,
    pub color: String,
    pub element_type: String,
    pub css_selector: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionData {
    pub session_id: String,
    pub domain: String,
    pub url: String,
    pub cookies: Vec<CookieData>,
    pub local_storage: HashMap<String, String>,
    pub session_storage: HashMap<String, String>,
    pub user_agent: Option<String>,
    pub viewport: Option<ViewportData>,
    pub custom_headers: HashMap<String, String>,
    pub auth_tokens: HashMap<String, String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub metadata: SessionMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CookieData {
    pub name: String,
    pub value: String,
    pub domain: String,
    pub path: String,
    pub expires: Option<i64>,
    pub http_only: bool,
    pub secure: bool,
    pub same_site: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewportData {
    pub width: u32,
    pub height: u32,
    pub device_scale_factor: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMetadata {
    pub login_selectors: Vec<String>,
    pub success_indicators: Vec<String>,
    pub failure_indicators: Vec<String>,
    pub csrf_tokens: HashMap<String, String>,
    pub form_data: HashMap<String, String>,
}

impl<B: BrowserTrait> BrowserSession<B> {
    pub async fn new(mut browser: B, config: Config) -> Result<Self> {
        browser.launch(&config).await?;
        let tab = browser.new_tab().await?;
        let browser = Arc::new(browser);
        let dom_processor = DomProcessor::new(config.dom.clone());
        let element_monitor = ElementMonitor::new();
        let session_id = uuid::Uuid::new_v4().to_string();

        Ok(Self {
            browser,
            tab: Some(tab),
            dom_processor,
            config,
            element_highlights: Vec::new(),
            element_monitor,
            auto_refresh_enabled: true,
            session_id,
            current_session_data: None,
        })
    }

    pub async fn new_with_session(
        mut browser: B,
        config: Config,
        session_data: SessionData,
    ) -> Result<Self> {
        let mut session = Self::new(browser, config).await?;
        session.inject_session(session_data).await?;
        Ok(session)
    }

    pub async fn navigate_and_wait_reactive(&mut self, url: &str) -> Result<NavigationResult> {
        self.navigate_smart(url).await
    }
    pub async fn extract_session(&mut self, domain: &str) -> Result<SessionData> {
        let tab = self
            .tab
            .as_ref()
            .ok_or_else(|| crate::errors::BrowserAgentError::NoActiveTab)?;

        let current_url = self.browser.get_url(tab).await?;

        println!("🔍 Extracting session data for domain: {}", domain);

        let cookies = self.extract_cookies(domain).await?;
        println!("   Extracted {} cookies", cookies.len());

        let local_storage = self.extract_local_storage().await?;
        println!("   Extracted {} localStorage items", local_storage.len());

        let session_storage = self.extract_session_storage().await?;
        println!(
            "   Extracted {} sessionStorage items",
            session_storage.len()
        );

        let auth_tokens = self.extract_auth_tokens().await?;
        println!("   Extracted {} auth tokens", auth_tokens.len());

        let csrf_tokens = self.extract_csrf_tokens().await?;
        println!("   Extracted {} CSRF tokens", csrf_tokens.len());

        let viewport = self.get_viewport_info().await?;

        let user_agent = self.get_user_agent().await?;

        let session_data = SessionData {
            session_id: self.session_id.clone(),
            domain: domain.to_string(),
            url: current_url,
            cookies,
            local_storage,
            session_storage,
            user_agent: Some(user_agent),
            viewport: Some(viewport),
            custom_headers: HashMap::new(),
            auth_tokens,
            timestamp: chrono::Utc::now(),
            metadata: SessionMetadata {
                login_selectors: vec![],
                success_indicators: vec![],
                failure_indicators: vec![],
                csrf_tokens,
                form_data: HashMap::new(),
            },
        };

        self.current_session_data = Some(session_data.clone());
        println!("✅ Session extraction completed");

        Ok(session_data)
    }

    pub async fn inject_session(&mut self, session_data: SessionData) -> Result<()> {
        println!(
            "💉 Injecting session data for domain: {}",
            session_data.domain
        );

        let current_url = {
            let tab = self
                .tab
                .as_ref()
                .ok_or_else(|| crate::errors::BrowserAgentError::NoActiveTab)?;
            self.browser.get_url(tab).await?
        };

        if !current_url.contains(&session_data.domain) {
            let domain_url = if session_data.domain.starts_with("http") {
                session_data.domain.clone()
            } else {
                format!("https://{}", session_data.domain)
            };
            self.navigate_and_wait_reactive(&domain_url).await?;
        }

        let tab = self
            .tab
            .as_ref()
            .ok_or_else(|| crate::errors::BrowserAgentError::NoActiveTab)?;

        self.inject_cookies(&session_data.cookies).await?;
        println!("   Injected {} cookies", session_data.cookies.len());

        self.inject_local_storage(&session_data.local_storage)
            .await?;
        println!(
            "   Injected {} localStorage items",
            session_data.local_storage.len()
        );

        self.inject_session_storage(&session_data.session_storage)
            .await?;
        println!(
            "   Injected {} sessionStorage items",
            session_data.session_storage.len()
        );

        if !session_data.custom_headers.is_empty() {
            self.set_custom_headers(&session_data.custom_headers)
                .await?;
            println!(
                "   Set {} custom headers",
                session_data.custom_headers.len()
            );
        }

        self.inject_auth_tokens(&session_data.auth_tokens).await?;
        println!("   Injected {} auth tokens", session_data.auth_tokens.len());

        if let Some(viewport) = &session_data.viewport {
            self.set_viewport(viewport).await?;
        }

        self.browser
            .execute_script(tab, "window.location.reload()")
            .await?;

        tokio::time::sleep(tokio::time::Duration::from_millis(2000)).await;

        self.current_session_data = Some(session_data);
        println!("✅ Session injection completed");

        Ok(())
    }

    pub async fn delete_session(&mut self) -> Result<()> {
        let tab = self
            .tab
            .as_ref()
            .ok_or_else(|| crate::errors::BrowserAgentError::NoActiveTab)?;

        println!("🗑️ Deleting session data");

        self.clear_all_cookies().await?;

        let clear_storage_script = r#"
            (function() {
                try {
                    localStorage.clear();

                    sessionStorage.clear();

                    if (window.indexedDB) {
                        indexedDB.databases().then(databases => {
                            databases.forEach(db => {
                                indexedDB.deleteDatabase(db.name);
                            });
                        }).catch(e => console.log('IndexedDB clear failed:', e));
                    }

                    if ('caches' in window) {
                        caches.keys().then(names => {
                            names.forEach(name => {
                                caches.delete(name);
                            });
                        }).catch(e => console.log('Cache clear failed:', e));
                    }

                    return { success: true, message: 'All storage cleared' };
                } catch (error) {
                    return { success: false, error: error.message };
                }
            })()
        "#;

        let result = self
            .browser
            .execute_script(tab, clear_storage_script)
            .await?;
        println!("   Storage clear result: {:?}", result);

        self.current_session_data = None;
        println!("✅ Session deletion completed");

        Ok(())
    }

    pub async fn validate_session(&self, success_indicators: &[String]) -> Result<bool> {
        let tab = self
            .tab
            .as_ref()
            .ok_or_else(|| crate::errors::BrowserAgentError::NoActiveTab)?;

        if success_indicators.is_empty() {
            return Ok(true);
        }

        let validation_script = format!(
            r#"
            (function() {{
                const indicators = {};
                let validCount = 0;

                for (const indicator of indicators) {{
                    if (document.querySelector(indicator)) {{
                        validCount++;
                        continue;
                    }};

                    if (document.body.textContent.includes(indicator)) {{
                        validCount++;
                        continue;
                    }}

                    if (localStorage.getItem(indicator)) {{
                        validCount++;
                        continue;
                    }}

                    if (document.cookie.includes(indicator)) {{
                        validCount++;
                        continue;
                    }}
                }}

                return {{
                    valid: validCount > 0,
                    validCount: validCount,
                    totalIndicators: indicators.length
                }};
            }})()
        "#,
            serde_json::to_string(success_indicators)?
        );

        let result = self.browser.execute_script(tab, &validation_script).await?;
        let is_valid = result
            .get("valid")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        Ok(is_valid)
    }

    async fn extract_cookies(&self, domain: &str) -> Result<Vec<CookieData>> {
        let tab = self
            .tab
            .as_ref()
            .ok_or_else(|| crate::errors::BrowserAgentError::NoActiveTab)?;

        let cookie_script = r#"
            (function() {
                const cookies = [];
                document.cookie.split(';').forEach(cookie => {
                    const [name, value] = cookie.trim().split('=');
                    if (name && value) {
                        cookies.push({
                            name: name.trim(),
                            value: value.trim(),
                            domain: window.location.hostname,
                            path: '/',
                            httpOnly: false,
                            secure: window.location.protocol === 'https:',
                            sameSite: null
                        });
                    }
                });
                return cookies;
            })()
        "#;

        let result = self.browser.execute_script(tab, cookie_script).await?;
        let cookies: Vec<CookieData> = serde_json::from_value(result)?;
        Ok(cookies)
    }

    async fn extract_local_storage(&self) -> Result<HashMap<String, String>> {
        let tab = self
            .tab
            .as_ref()
            .ok_or_else(|| crate::errors::BrowserAgentError::NoActiveTab)?;

        let script = r#"
            (function() {
                const storage = {};
                for (let i = 0; i < localStorage.length; i++) {
                    const key = localStorage.key(i);
                    if (key) {
                        storage[key] = localStorage.getItem(key);
                    }
                }
                return storage;
            })()
        "#;

        let result = self.browser.execute_script(tab, script).await?;
        let storage: HashMap<String, String> = serde_json::from_value(result)?;
        Ok(storage)
    }

    async fn extract_session_storage(&self) -> Result<HashMap<String, String>> {
        let tab = self
            .tab
            .as_ref()
            .ok_or_else(|| crate::errors::BrowserAgentError::NoActiveTab)?;

        let script = r#"
            (function() {
                const storage = {};
                for (let i = 0; i < sessionStorage.length; i++) {
                    const key = sessionStorage.key(i);
                    if (key) {
                        storage[key] = sessionStorage.getItem(key);
                    }
                }
                return storage;
            })()
        "#;

        let result = self.browser.execute_script(tab, script).await?;
        let storage: HashMap<String, String> = serde_json::from_value(result)?;
        Ok(storage)
    }

    async fn extract_auth_tokens(&self) -> Result<HashMap<String, String>> {
        let tab = self
            .tab
            .as_ref()
            .ok_or_else(|| crate::errors::BrowserAgentError::NoActiveTab)?;

        let script = r#"
            (function() {
                const tokens = {};

                const tokenKeys = [
                    'access_token', 'accessToken', 'authToken', 'auth_token',
                    'bearer_token', 'bearerToken', 'jwt', 'JWT', 'token',
                    'refresh_token', 'refreshToken', 'id_token', 'idToken',
                    'session_token', 'sessionToken', 'api_key', 'apiKey',
                    'authorization', 'Authorization', 'x-auth-token'
                ];

                tokenKeys.forEach(key => {
                    const value = localStorage.getItem(key);
                    if (value) tokens[key] = value;
                });

                tokenKeys.forEach(key => {
                    const value = sessionStorage.getItem(key);
                    if (value) tokens[`session_${key}`] = value;
                });

                document.cookie.split(';').forEach(cookie => {
                    const [name, value] = cookie.trim().split('=');
                    if (name && value && tokenKeys.some(key => name.toLowerCase().includes(key.toLowerCase()))) {
                        tokens[`cookie_${name.trim()}`] = value.trim();
                    }
                });

                const metaTags = document.querySelectorAll('meta[name*="token"], meta[name*="csrf"]');
                metaTags.forEach(meta => {
                    const name = meta.getAttribute('name');
                    const content = meta.getAttribute('content');
                    if (name && content) {
                        tokens[`meta_${name}`] = content;
                    }
                });

                return tokens;
            })()
        "#;

        let result = self.browser.execute_script(tab, script).await?;
        let tokens: HashMap<String, String> = serde_json::from_value(result)?;
        Ok(tokens)
    }

    async fn extract_csrf_tokens(&self) -> Result<HashMap<String, String>> {
        let tab = self
            .tab
            .as_ref()
            .ok_or_else(|| crate::errors::BrowserAgentError::NoActiveTab)?;

        let script = r#"
            (function() {
                const tokens = {};

                const metaTags = document.querySelectorAll('meta[name*="csrf"], meta[name*="token"]');
                metaTags.forEach(meta => {
                    const name = meta.getAttribute('name');
                    const content = meta.getAttribute('content');
                    if (name && content) {
                        tokens[name] = content;
                    }
                });

                const hiddenInputs = document.querySelectorAll('input[type="hidden"][name*="csrf"], input[type="hidden"][name*="token"]');
                hiddenInputs.forEach(input => {
                    const name = input.getAttribute('name');
                    const value = input.getAttribute('value');
                    if (name && value) {
                        tokens[name] = value;
                    }
                });

                const csrfKeys = ['csrf_token', 'csrfToken', '_token', 'authenticity_token'];
                csrfKeys.forEach(key => {
                    const value = localStorage.getItem(key);
                    if (value) tokens[key] = value;
                });

                return tokens;
            })()
        "#;

        let result = self.browser.execute_script(tab, script).await?;
        let tokens: HashMap<String, String> = serde_json::from_value(result)?;
        Ok(tokens)
    }
    pub async fn navigate_smart(&mut self, url: &str) -> Result<NavigationResult> {
        let tab = self
            .tab
            .as_ref()
            .ok_or_else(|| crate::errors::BrowserAgentError::NoActiveTab)?;

        println!("🚀 Smart navigating to: {}", url);

        // Start navigation
        self.browser.navigate(tab, url).await?;

        // Use dynamic navigation detection
        let nav_result = NavigationManager::wait_for_navigation_complete(
            self.browser.as_ref(),
            tab,
            self.config.session.navigation_timeout_ms,
        )
        .await?;

        println!(
            "✅ Navigation completed: {} | Quality: {} | Load time: {}ms | Reason: {}",
            nav_result.url,
            nav_result.load_quality(),
            nav_result.actual_load_time,
            nav_result.reason
        );

        // Only start monitoring if navigation was successful
        if nav_result.has_content {
            self.element_monitor
                .start_monitoring(self.browser.as_ref(), tab)
                .await?;

            if self.auto_refresh_enabled {
                let _ = self.refresh_elements_after_change().await;
            }
        }

        Ok(nav_result)
    }
    async fn get_viewport_info(&self) -> Result<ViewportData> {
        let tab = self
            .tab
            .as_ref()
            .ok_or_else(|| crate::errors::BrowserAgentError::NoActiveTab)?;

        let script = r#"
            (function() {
                return {
                    width: window.innerWidth,
                    height: window.innerHeight,
                    deviceScaleFactor: window.devicePixelRatio || 1
                };
            })()
        "#;

        let result = self.browser.execute_script(tab, script).await?;
        let viewport: ViewportData = serde_json::from_value(result)?;
        Ok(viewport)
    }

    async fn get_user_agent(&self) -> Result<String> {
        let tab = self
            .tab
            .as_ref()
            .ok_or_else(|| crate::errors::BrowserAgentError::NoActiveTab)?;

        let result = self
            .browser
            .execute_script(tab, "navigator.userAgent")
            .await?;
        Ok(result.as_str().unwrap_or("").to_string())
    }

    async fn inject_cookies(&self, cookies: &[CookieData]) -> Result<()> {
        let tab = self
            .tab
            .as_ref()
            .ok_or_else(|| crate::errors::BrowserAgentError::NoActiveTab)?;

        for cookie in cookies {
            let cookie_script = format!(
                r#"
                (function() {{
                    let cookieString = '{}={}; path={}';

                    if ('{}' !== 'null') {{
                        const expires = new Date({} * 1000);
                        cookieString += '; expires=' + expires.toUTCString();
                    }}

                    if ({}) {{
                        cookieString += '; secure';
                    }}

                    if ('{}' !== 'null') {{
                        cookieString += '; samesite={}';
                    }}

                    document.cookie = cookieString;
                    return {{ success: true, cookie: cookieString }};
                }})()
            "#,
                cookie.name,
                cookie.value,
                cookie.path,
                cookie
                    .expires
                    .map(|e| e.to_string())
                    .unwrap_or_else(|| "null".to_string()),
                cookie.expires.unwrap_or(0),
                cookie.secure,
                cookie.same_site.as_ref().unwrap_or(&"null".to_string()),
                cookie.same_site.as_ref().unwrap_or(&"".to_string())
            );

            self.browser.execute_script(tab, &cookie_script).await?;
        }

        Ok(())
    }

    async fn inject_local_storage(&self, storage: &HashMap<String, String>) -> Result<()> {
        let tab = self
            .tab
            .as_ref()
            .ok_or_else(|| crate::errors::BrowserAgentError::NoActiveTab)?;

        let script = format!(
            r#"
            (function() {{
                const storage = {};
                let count = 0;
                try {{
                    for (const [key, value] of Object.entries(storage)) {{
                        localStorage.setItem(key, value);
                        count++;
                    }}
                    return {{ success: true, count: count }};
                }} catch (error) {{
                    return {{ success: false, error: error.message, count: count }};
                }}
            }})()
        "#,
            serde_json::to_string(storage)?
        );

        self.browser.execute_script(tab, &script).await?;
        Ok(())
    }

    async fn inject_session_storage(&self, storage: &HashMap<String, String>) -> Result<()> {
        let tab = self
            .tab
            .as_ref()
            .ok_or_else(|| crate::errors::BrowserAgentError::NoActiveTab)?;

        let script = format!(
            r#"
            (function() {{
                const storage = {};
                let count = 0;
                try {{
                    for (const [key, value] of Object.entries(storage)) {{
                        sessionStorage.setItem(key, value);
                        count++;
                    }}
                    return {{ success: true, count: count }};
                }} catch (error) {{
                    return {{ success: false, error: error.message, count: count }};
                }}
            }})()
        "#,
            serde_json::to_string(storage)?
        );

        self.browser.execute_script(tab, &script).await?;
        Ok(())
    }

    async fn inject_auth_tokens(&self, tokens: &HashMap<String, String>) -> Result<()> {
        let tab = self
            .tab
            .as_ref()
            .ok_or_else(|| crate::errors::BrowserAgentError::NoActiveTab)?;

        let script = format!(
            r#"
            (function() {{
                const tokens = {};
                let count = 0;

                try {{
                    for (const [key, value] of Object.entries(tokens)) {{
                        if (key.startsWith('cookie_')) {{
                            continue;
                        }} else if (key.startsWith('session_')) {{
                            const realKey = key.replace('session_', '');
                            sessionStorage.setItem(realKey, value);
                            count++;
                        }} else if (key.startsWith('meta_')) {{
                            continue;
                        }} else {{
                            localStorage.setItem(key, value);
                            count++;
                        }}
                    }}

                    return {{ success: true, count: count }};
                }} catch (error) {{
                    return {{ success: false, error: error.message, count: count }};
                }}
            }})()
        "#,
            serde_json::to_string(tokens)?
        );

        self.browser.execute_script(tab, &script).await?;
        Ok(())
    }

    async fn set_custom_headers(&self, _headers: &HashMap<String, String>) -> Result<()> {
        println!("⚠️ Custom headers setting not implemented (requires CDP)");
        Ok(())
    }

    async fn set_viewport(&self, viewport: &ViewportData) -> Result<()> {
        let tab = self
            .tab
            .as_ref()
            .ok_or_else(|| crate::errors::BrowserAgentError::NoActiveTab)?;

        let script = format!(
            r#"
            (function() {{
                window.resizeTo({}, {});
                return {{
                    success: true,
                    width: window.innerWidth,
                    height: window.innerHeight
                }};
            }})()
        "#,
            viewport.width, viewport.height
        );

        self.browser.execute_script(tab, &script).await?;
        Ok(())
    }

    async fn clear_all_cookies(&self) -> Result<()> {
        let tab = self
            .tab
            .as_ref()
            .ok_or_else(|| crate::errors::BrowserAgentError::NoActiveTab)?;

        let script = r#"
            (function() {
                const cookies = document.cookie.split(';');
                let clearedCount = 0;

                cookies.forEach(cookie => {
                    const name = cookie.split('=')[0].trim();
                    if (name) {
                        document.cookie = name + '=; expires=Thu, 01 Jan 1970 00:00:00 UTC; path=/;';
                        document.cookie = name + '=; expires=Thu, 01 Jan 1970 00:00:00 UTC; path=/; domain=' + window.location.hostname + ';';
                        document.cookie = name + '=; expires=Thu, 01 Jan 1970 00:00:00 UTC; path=/; domain=.' + window.location.hostname + ';';
                        clearedCount++;
                    }
                });

                return { success: true, clearedCount: clearedCount };
            })()
        "#;

        self.browser.execute_script(tab, script).await?;
        Ok(())
    }

    async fn check_and_refresh_if_needed(&mut self) -> Result<()> {
        if !self.auto_refresh_enabled {
            return Ok(());
        }

        let tab = self
            .tab
            .as_ref()
            .ok_or_else(|| crate::errors::BrowserAgentError::NoActiveTab)?;

        let change_result = self
            .element_monitor
            .wait_for_changes(self.browser.as_ref(), tab, 1000)
            .await?;

        if change_result.has_changes {
            println!("🔄 DOM changes detected: {:?}", change_result.change_types);
            self.refresh_elements_after_change().await?;
        } else {
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            let quick_check = self
                .element_monitor
                .check_for_changes(self.browser.as_ref(), tab)
                .await?;

            if quick_check.has_changes {
                self.refresh_elements_after_change().await?;
            }
        }

        Ok(())
    }

    async fn refresh_elements_after_change(&mut self) -> Result<()> {
        println!("🔄 Refreshing elements due to DOM changes...");

        self.clear_element_highlights().await?;

        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

        let highlights = self.highlight_interactive_elements().await?;

        println!("✅ Refreshed {} interactive elements", highlights.len());
        Ok(())
    }

    pub async fn type_text_enhanced(&self, selector: &str, text: &str) -> Result<()> {
        let tab = self
            .tab
            .as_ref()
            .ok_or_else(|| crate::errors::BrowserAgentError::NoActiveTab)?;

        let typing_script = format!(
            r#"
                (function() {{
                    const element = document.querySelector('{}');
                    if (!element) return {{ success: false, error: 'Element not found' }};

                    try {{
                        element.focus();
                        element.click();

                        element.value = '';
                        if (element.textContent !== undefined) {{
                            element.textContent = '';
                        }}
                        if (element.innerHTML !== undefined && element.contentEditable === 'true') {{
                            element.innerHTML = '';
                        }}
                        if (element.tagName.toLowerCase() === 'input' || element.tagName.toLowerCase() === 'textarea') {{
                                                   element.value = '{}';
                                                   element.dispatchEvent(new Event('input', {{ bubbles: true, cancelable: true }}));
                                                                           }} else if (element.contentEditable === 'true') {{
                                                                               element.textContent = '{}';
                                                                               element.innerHTML = '{}';
                                                                           }}

                                                                           const events = ['focus', 'input', 'change', 'keydown', 'keyup', 'blur'];
                                                                           events.forEach(eventType => {{
                                                                               const event = new Event(eventType, {{ bubbles: true, cancelable: true }});
                                                                               element.dispatchEvent(event);
                                                                           }});

                                                                           if (element.name === 'q' || element.getAttribute('role') === 'searchbox') {{
                                                                               element.dispatchEvent(new InputEvent('input', {{
                                                                                   bubbles: true,
                                                                                   cancelable: true,
                                                                                   inputType: 'insertText',
                                                                                   data: '{}'
                                                                               }}));
                                                                           }}

                                                                           const finalValue = element.value || element.textContent || element.innerHTML || '';

                                                                           return {{
                                                                               success: true,
                                                                               finalValue: finalValue,
                                                                               elementType: element.tagName.toLowerCase(),
                                                                               elementName: element.name || 'unnamed'
                                                                           }};

                                                                       }} catch (error) {{
                                                                           return {{ success: false, error: error.message }};
                                                                       }}
                                                                   }})()
                                                                   "#,
            selector.replace("'", "\\'"),
            text.replace("'", "\\'")
                .replace("\"", "\\\"")
                .replace("\\", "\\\\"),
            text.replace("'", "\\'")
                .replace("\"", "\\\"")
                .replace("\\", "\\\\"),
            text.replace("'", "\\'")
                .replace("\"", "\\\"")
                .replace("\\", "\\\\"),
            text.replace("'", "\\'")
                .replace("\"", "\\\"")
                .replace("\\", "\\\\")
        );

        let result = self.browser.execute_script(tab, &typing_script).await?;

        if let Some(result_obj) = result.as_object() {
            if result_obj
                .get("success")
                .and_then(|v| v.as_bool())
                .unwrap_or(false)
            {
                println!("✅ Successfully typed in element: {}", selector);
                if let Some(final_value) = result_obj.get("finalValue") {
                    println!("   Final value: {}", final_value);
                }
                return Ok(());
            } else if let Some(error) = result_obj.get("error") {
                println!("❌ Typing failed: {}", error);
            }
        }

        Err(crate::errors::BrowserAgentError::ElementNotFound(format!(
            "Failed to type in element: {}",
            selector
        )))
    }

    pub async fn get_ai_elements(&self) -> Result<Vec<AIElement>> {
        let dom_state = self.get_page_state(false).await?;
        let mut ai_elements = Vec::new();

        for element in &dom_state.elements {
            if !element.is_clickable && !element.is_interactable && element.text_content.is_none() {
                continue;
            }

            let ai_element = AIElement {
                id: element.id.clone(),
                element_number: ai_elements.len() + 1,
                tag_name: element.tag_name.clone(),
                element_type: self.classify_element_type(element),
                selector: element.css_selector.clone(),
                xpath: element.xpath.clone(),
                text_content: element.text_content.clone(),
                placeholder: element.attributes.get("placeholder").cloned(),
                label: self.extract_element_label(element),
                description: self.generate_element_description(element),
                capabilities: self.get_element_capabilities(element),
                attributes: element.attributes.clone(),
                is_visible: element.is_visible,
                ai_instructions: self.generate_ai_instructions(element),
            };

            ai_elements.push(ai_element);
        }

        Ok(ai_elements)
    }

    pub async fn highlight_interactive_elements(&mut self) -> Result<Vec<ElementHighlight>> {
        let tab = self
            .tab
            .as_ref()
            .ok_or_else(|| crate::errors::BrowserAgentError::NoActiveTab)?;

        self.clear_element_highlights().await?;

        let dom_state = self.get_page_state(false).await?;

        let mut highlights = Vec::new();
        let mut element_counter = 1;

        let mut batch_script = String::from(
            r#"
                                                                   (function() {
                                                                       const results = [];
                                                                       const style = document.createElement('style');
                                                                       style.textContent = `
                                                                           .browser-automation-highlight {
                                                                               position: fixed !important;
                                                                               pointer-events: none !important;
                                                                               z-index: 999999 !important;
                                                                               box-sizing: border-box !important;
                                                                               font-family: Arial, sans-serif !important;
                                                                           }
                                                                           .browser-automation-highlight-label {
                                                                               position: absolute !important;
                                                                               top: -25px !important;
                                                                               left: -3px !important;
                                                                               color: white !important;
                                                                               padding: 2px 6px !important;
                                                                               font-size: 12px !important;
                                                                               font-weight: bold !important;
                                                                               border-radius: 3px !important;
                                                                               white-space: nowrap !important;
                                                                           }
                                                                       `;
                                                                       document.head.appendChild(style);
                                                                   "#,
        );

        for element in &dom_state.clickable_elements {
            let color = match element.tag_name.as_str() {
                "button" => "#0000FF",
                "input" => "#00FF00",
                "select" => "#FF6600",
                "textarea" => "#9900FF",
                "a" => "#00FFFF",
                _ => "#FF0000",
            };

            batch_script.push_str(&format!(
                                                                       r#"
                                                                       try {{
                                                                           const element = document.querySelector('{}');
                                                                           if (element) {{
                                                                               const rect = element.getBoundingClientRect();
                                                                               if (rect.width > 0 && rect.height > 0) {{
                                                                                   const overlay = document.createElement('div');
                                                                                   overlay.className = 'browser-automation-highlight browser-automation-highlight-{}';
                                                                                   overlay.style.left = rect.left + 'px';
                                                                                   overlay.style.top = rect.top + 'px';
                                                                                   overlay.style.width = rect.width + 'px';
                                                                                   overlay.style.height = rect.height + 'px';
                                                                                   overlay.style.border = '3px solid {}';
                                                                                   overlay.style.backgroundColor = 'rgba(255,255,255,0.1)';

                                                                                   const label = document.createElement('div');
                                                                                   label.className = 'browser-automation-highlight-label';
                                                                                   label.style.backgroundColor = '{}';
                                                                                   label.textContent = '{}';

                                                                                   overlay.appendChild(label);
                                                                                   document.body.appendChild(overlay);
                                                                                   results.push({});
                                                                               }}
                                                                           }}
                                                                       }} catch(e) {{
                                                                           console.error('Highlight error for element {}:', e);
                                                                       }}
                                                                       "#,
                                                                       element.css_selector.replace("'", "\\'"),
                                                                       element_counter,
                                                                       color,
                                                                       color,
                                                                       element_counter,
                                                                       element_counter,
                                                                       element_counter
                                                                   ));

            highlights.push(ElementHighlight {
                element_id: element.id.clone(),
                element_number: element_counter,
                color: color.to_string(),
                element_type: element.tag_name.clone(),
                css_selector: element.css_selector.clone(),
            });
            element_counter += 1;
        }

        batch_script.push_str(" return results.length; })()");

        let result = self.browser.execute_script(tab, &batch_script).await?;
        println!("✅ Highlighted {} elements", result.as_u64().unwrap_or(0));

        self.element_highlights = highlights.clone();
        Ok(highlights)
    }

    pub async fn clear_element_highlights(&self) -> Result<()> {
        let tab = self
            .tab
            .as_ref()
            .ok_or_else(|| crate::errors::BrowserAgentError::NoActiveTab)?;

        let clear_script = r#"
                                                                   (function() {
                                                                       const highlights = document.querySelectorAll('.browser-automation-highlight');
                                                                       highlights.forEach(highlight => highlight.remove());
                                                                       const styles = document.querySelectorAll('style');
                                                                       styles.forEach(style => {
                                                                           if (style.textContent && style.textContent.includes('browser-automation-highlight')) {
                                                                               style.remove();
                                                                           }
                                                                       });
                                                                       return highlights.length;
                                                                   })()
                                                               "#;

        self.browser.execute_script(tab, clear_script).await?;
        Ok(())
    }

    pub async fn click_element_by_number(&self, element_number: usize) -> Result<()> {
        if let Some(highlight) = self
            .element_highlights
            .iter()
            .find(|h| h.element_number == element_number)
        {
            self.click(&highlight.css_selector).await
        } else {
            Err(crate::errors::BrowserAgentError::ElementNotFound(format!(
                "Element number {} not found",
                element_number
            )))
        }
    }

    pub async fn type_in_element_by_number(&self, element_number: usize, text: &str) -> Result<()> {
        if let Some(highlight) = self
            .element_highlights
            .iter()
            .find(|h| h.element_number == element_number)
        {
            self.type_text_enhanced(&highlight.css_selector, text).await
        } else {
            Err(crate::errors::BrowserAgentError::ElementNotFound(format!(
                "Element number {} not found",
                element_number
            )))
        }
    }

    pub fn get_highlighted_elements(&self) -> &[ElementHighlight] {
        &self.element_highlights
    }

    pub async fn click_with_refresh(&mut self, selector: &str) -> Result<()> {
        self.click(selector).await?;
        self.check_and_refresh_if_needed().await?;
        Ok(())
    }

    pub async fn type_with_refresh(&mut self, selector: &str, text: &str) -> Result<()> {
        self.type_text_enhanced(selector, text).await?;
        self.check_and_refresh_if_needed().await?;
        Ok(())
    }

    pub async fn click_element_by_number_with_refresh(
        &mut self,
        element_number: usize,
    ) -> Result<()> {
        self.click_element_by_number(element_number).await?;
        self.check_and_refresh_if_needed().await?;
        Ok(())
    }

    pub async fn wait_for_elements(&mut self, selector: &str, timeout_ms: u64) -> Result<bool> {
        let tab = self
            .tab
            .as_ref()
            .ok_or_else(|| crate::errors::BrowserAgentError::NoActiveTab)?;

        let wait_script = format!(
            r#"
                                                                   (function() {{
                                                                       return new Promise((resolve) => {{
                                                                           const checkElement = () => {{
                                                                               const elements = document.querySelectorAll('{}');
                                                                               if (elements.length > 0) {{
                                                                                   resolve({{ found: true, count: elements.length }});
                                                                                   return true;
                                                                               }}
                                                                               return false;
                                                                           }};

                                                                           if (checkElement()) return;

                                                                           const observer = new MutationObserver(() => {{
                                                                               if (checkElement()) {{
                                                                                   observer.disconnect();
                                                                               }}
                                                                           }});

                                                                           observer.observe(document.body, {{
                                                                               childList: true,
                                                                               subtree: true
                                                                           }});

                                                                           setTimeout(() => {{
                                                                               observer.disconnect();
                                                                               resolve({{ found: false, timeout: true }});
                                                                           }}, {});
                                                                       }});
                                                                   }})()
                                                               "#,
            selector.replace("'", "\\'"),
            timeout_ms
        );

        let result = self.browser.execute_script(tab, &wait_script).await?;
        let found = result
            .get("found")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        if found {
            self.refresh_elements_after_change().await?;
        }

        Ok(found)
    }

    pub async fn get_current_interactive_elements(&self) -> Result<Vec<AIElement>> {
        self.get_ai_elements().await
    }

    pub async fn auto_login_and_extract_session(
        &mut self,
        login_url: &str,
        username: &str,
        password: &str,
        login_config: LoginConfig,
    ) -> Result<SessionData> {
        println!("🔐 Starting auto-login process for: {}", login_url);

        self.navigate_and_wait_reactive(login_url).await?;

        tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;

        self.highlight_interactive_elements().await?;

        let username_filled = self
            .try_fill_field(&login_config.username_selectors, username)
            .await?;
        if !username_filled {
            return Err(crate::errors::BrowserAgentError::ElementNotFound(
                "Username field not found".to_string(),
            ));
        }

        let password_filled = self
            .try_fill_field(&login_config.password_selectors, password)
            .await?;
        if !password_filled {
            return Err(crate::errors::BrowserAgentError::ElementNotFound(
                "Password field not found".to_string(),
            ));
        }

        let submit_clicked = self
            .try_click_element(&login_config.submit_selectors)
            .await?;
        if !submit_clicked {
            return Err(crate::errors::BrowserAgentError::ElementNotFound(
                "Submit button not found".to_string(),
            ));
        }

        tokio::time::sleep(tokio::time::Duration::from_millis(3000)).await;

        let login_successful = self
            .validate_session(&login_config.success_indicators)
            .await?;
        if !login_successful {
            return Err(crate::errors::BrowserAgentError::ConfigurationError(
                "Login appears to have failed".to_string(),
            ));
        }

        println!("✅ Login successful! Extracting session...");

        let domain = url::Url::parse(login_url)
            .map_err(|e| crate::errors::BrowserAgentError::ConfigurationError(e.to_string()))?
            .host_str()
            .unwrap_or("unknown")
            .to_string();

        let mut session_data = self.extract_session(&domain).await?;

        session_data.metadata.login_selectors = login_config.username_selectors.clone();
        session_data.metadata.success_indicators = login_config.success_indicators.clone();
        session_data.metadata.failure_indicators = login_config.failure_indicators.clone();

        Ok(session_data)
    }

    async fn try_fill_field(&mut self, selectors: &[String], value: &str) -> Result<bool> {
        for selector in selectors {
            if let Ok(_) = self.type_with_refresh(selector, value).await {
                println!("✅ Filled field with selector: {}", selector);
                return Ok(true);
            }
        }
        Ok(false)
    }

    async fn try_click_element(&mut self, selectors: &[String]) -> Result<bool> {
        for selector in selectors {
            if let Ok(_) = self.click_with_refresh(selector).await {
                println!("✅ Clicked element with selector: {}", selector);
                return Ok(true);
            }
        }
        Ok(false)
    }

    fn classify_element_type(&self, element: &crate::dom::DomElement) -> String {
        match element.tag_name.as_str() {
            "input" => {
                let input_type = element
                    .attributes
                    .get("type")
                    .map(|s| s.as_str())
                    .unwrap_or("text");
                match input_type {
                    "text" | "email" | "password" | "search" | "url" | "tel" => {
                        "text_input".to_string()
                    }
                    "checkbox" => "checkbox".to_string(),
                    "radio" => "radio_button".to_string(),
                    "submit" | "button" => "button".to_string(),
                    "file" => "file_upload".to_string(),
                    _ => format!("input_{}", input_type),
                }
            }
            "textarea" => "text_area".to_string(),
            "select" => "dropdown".to_string(),
            "button" => "button".to_string(),
            "a" => "link".to_string(),
            _ => {
                if element.is_clickable {
                    "clickable_element".to_string()
                } else {
                    "text_element".to_string()
                }
            }
        }
    }

    fn extract_element_label(&self, element: &crate::dom::DomElement) -> Option<String> {
        if let Some(aria_label) = element.attributes.get("aria-label") {
            return Some(aria_label.clone());
        }

        if let Some(title) = element.attributes.get("title") {
            return Some(title.clone());
        }

        if let Some(placeholder) = element.attributes.get("placeholder") {
            return Some(placeholder.clone());
        }

        if let Some(name) = element.attributes.get("name") {
            return Some(name.clone());
        }

        if let Some(text) = &element.text_content {
            if !text.trim().is_empty() && text.len() < 100 {
                return Some(text.clone());
            }
        }

        None
    }

    fn generate_element_description(&self, element: &crate::dom::DomElement) -> String {
        let mut description_parts = Vec::new();

        let element_type = self.classify_element_type(element);
        description_parts.push(format!("A {} element", element_type.replace("_", " ")));

        if let Some(label) = self.extract_element_label(element) {
            description_parts.push(format!("labeled '{}'", label));
        }

        if let Some(id) = &element.element_id {
            description_parts.push(format!("with ID '{}'", id));
        }

        match element.tag_name.as_str() {
            "input" => {
                let input_type = element
                    .attributes
                    .get("type")
                    .map(|s| s.as_str())
                    .unwrap_or("text");
                match input_type {
                    "search" => description_parts.push("for entering search queries".to_string()),
                    "email" => description_parts.push("for entering email addresses".to_string()),
                    "password" => description_parts.push("for entering passwords".to_string()),
                    "submit" => description_parts.push("for submitting forms".to_string()),
                    _ => description_parts.push("for text input".to_string()),
                }
            }
            "textarea" => description_parts.push("for multi-line text input".to_string()),
            "select" => description_parts.push("for selecting from options".to_string()),
            "button" => description_parts.push("that can be clicked".to_string()),
            "a" => {
                if let Some(href) = element.attributes.get("href") {
                    description_parts.push(format!("linking to '{}'", href));
                } else {
                    description_parts.push("that can be clicked".to_string());
                }
            }
            _ => {}
        }

        description_parts.join(" ")
    }

    fn get_element_capabilities(&self, element: &crate::dom::DomElement) -> Vec<String> {
        let mut capabilities = Vec::new();

        if element.is_clickable {
            capabilities.push("clickable".to_string());
        }

        if element.is_interactable {
            capabilities.push("can_receive_text_input".to_string());
        }

        if matches!(element.tag_name.as_str(), "select") {
            capabilities.push("can_select_options".to_string());
        }

        if matches!(element.tag_name.as_str(), "input") {
            if let Some(input_type) = element.attributes.get("type") {
                match input_type.as_str() {
                    "checkbox" => capabilities.push("can_check_uncheck".to_string()),
                    "radio" => capabilities.push("can_select".to_string()),
                    "file" => capabilities.push("can_upload_files".to_string()),
                    _ => {}
                }
            }
        }

        capabilities
    }

    fn generate_ai_instructions(&self, element: &crate::dom::DomElement) -> String {
        match element.tag_name.as_str() {
            "input" => {
                let input_type = element
                    .attributes
                    .get("type")
                    .map(|s| s.as_str())
                    .unwrap_or("text");
                match input_type {
                                                                           "search" => "Use type_in_element_by_number() to enter search terms, then look for a search button to click or press Enter".to_string(),
                                                                           "text" | "email" | "password" | "url" | "tel" => "Use type_in_element_by_number() to enter text".to_string(),
                                                                           "checkbox" => "Use click_element_by_number() to check/uncheck".to_string(),
                                                                           "radio" => "Use click_element_by_number() to select this option".to_string(),
                                                                           "submit" | "button" => "Use click_element_by_number() to submit the form".to_string(),
                                                                           _ => "Use click_element_by_number() to interact".to_string(),
                                                                       }
            }
            "textarea" => "Use type_in_element_by_number() to enter multi-line text".to_string(),
            "select" => {
                "Use click_element_by_number() to open dropdown, then select an option".to_string()
            }
            "button" => "Use click_element_by_number() to activate this button".to_string(),
            "a" => "Use click_element_by_number() to follow this link".to_string(),
            _ => {
                if element.is_clickable {
                    "Use click_element_by_number() to interact with this element".to_string()
                } else {
                    "This element contains text content for reference".to_string()
                }
            }
        }
    }

    pub fn get_session_data(&self) -> Option<&SessionData> {
        self.current_session_data.as_ref()
    }

    pub fn set_auto_refresh(&mut self, enabled: bool) {
        self.auto_refresh_enabled = enabled;
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AIElement {
    pub id: String,
    pub element_number: usize,
    pub tag_name: String,
    pub element_type: String,
    pub selector: String,
    pub xpath: String,
    pub text_content: Option<String>,
    pub placeholder: Option<String>,
    pub label: Option<String>,
    pub description: String,
    pub capabilities: Vec<String>,
    pub attributes: std::collections::HashMap<String, String>,
    pub is_visible: bool,
    pub ai_instructions: String,
}

#[derive(Debug, Clone)]
pub struct LoginConfig {
    pub username_selectors: Vec<String>,
    pub password_selectors: Vec<String>,
    pub submit_selectors: Vec<String>,
    pub success_indicators: Vec<String>,
    pub failure_indicators: Vec<String>,
}

impl Default for LoginConfig {
    fn default() -> Self {
        Self {
            username_selectors: vec![
                "input[name='username']".to_string(),
                "input[name='email']".to_string(),
                "input[type='email']".to_string(),
                "input[id*='username']".to_string(),
                "input[id*='email']".to_string(),
                "input[placeholder*='username']".to_string(),
                "input[placeholder*='email']".to_string(),
            ],
            password_selectors: vec![
                "input[type='password']".to_string(),
                "input[name='password']".to_string(),
                "input[id*='password']".to_string(),
            ],
            submit_selectors: vec![
                "button[type='submit']".to_string(),
                "input[type='submit']".to_string(),
                "button:contains('Login')".to_string(),
                "button:contains('Sign in')".to_string(),
                "button:contains('Log in')".to_string(),
                ".login-button".to_string(),
                ".signin-button".to_string(),
            ],
            success_indicators: vec![
                "dashboard".to_string(),
                "profile".to_string(),
                "logout".to_string(),
                "welcome".to_string(),
            ],
            failure_indicators: vec![
                "error".to_string(),
                "invalid".to_string(),
                "incorrect".to_string(),
                "failed".to_string(),
            ],
        }
    }
}

#[async_trait]
impl<B: BrowserTrait> SessionTrait<B> for BrowserSession<B> {
    async fn new(browser: B, config: Config) -> Result<Self> {
        Self::new(browser, config).await
    }

    async fn navigate_and_wait(&mut self, url: &str) -> Result<()> {
        self.navigate_and_wait_reactive(url).await?;
        Ok(())
    }

    async fn get_page_state(&self, include_screenshot: bool) -> Result<DomState> {
        let tab = self
            .tab
            .as_ref()
            .ok_or_else(|| crate::errors::BrowserAgentError::NoActiveTab)?;
        self.dom_processor
            .extract_dom_state(self.browser.as_ref(), tab, include_screenshot)
            .await
    }

    async fn click(&self, selector: &str) -> Result<()> {
        let tab = self
            .tab
            .as_ref()
            .ok_or_else(|| crate::errors::BrowserAgentError::NoActiveTab)?;

        let click_script = format!(
            r#"
                                                                   (function() {{
                                                                       const element = document.querySelector('{}');
                                                                       if (!element) return {{ success: false, error: 'Element not found' }};

                                                                       try {{
                                                                           element.scrollIntoView({{ behavior: 'smooth', block: 'center' }});

                                                                           setTimeout(() => {{
                                                                               element.focus();
                                                                               element.click();

                                                                               const rect = element.getBoundingClientRect();
                                                                               const centerX = rect.left + rect.width / 2;
                                                                               const centerY = rect.top + rect.height / 2;

                                                                               ['mousedown', 'mouseup', 'click'].forEach(eventType => {{
                                                                                   const event = new MouseEvent(eventType, {{
                                                                                       bubbles: true,
                                                                                       cancelable: true,
                                                                                       clientX: centerX,
                                                                                       clientY: centerY
                                                                                   }});
                                                                                   element.dispatchEvent(event);
                                                                               }});
                                                                           }}, 100);

                                                                           return {{ success: true, elementType: element.tagName.toLowerCase() }};
                                                                       }} catch (e) {{
                                                                           return {{ success: false, error: e.message }};
                                                                       }}
                                                                   }})()
                                                                   "#,
            selector.replace("'", "\\'")
        );

        let result = self.browser.execute_script(tab, &click_script).await?;

        if result
            .get("success")
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
        {
            println!("✅ Successfully clicked element: {}", selector);
            Ok(())
        } else {
            let error_msg = result
                .get("error")
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown error");
            println!("❌ Click failed: {}", error_msg);
            Err(crate::errors::BrowserAgentError::ElementNotFound(format!(
                "Failed to click element {}: {}",
                selector, error_msg
            )))
        }
    }

    async fn type_text(&self, selector: &str, text: &str) -> Result<()> {
        self.type_text_enhanced(selector, text).await
    }

    async fn execute_script(&self, script: &str) -> Result<serde_json::Value> {
        let tab = self
            .tab
            .as_ref()
            .ok_or_else(|| crate::errors::BrowserAgentError::NoActiveTab)?;
        self.browser.execute_script(tab, script).await
    }

    async fn screenshot(&self) -> Result<Vec<u8>> {
        let tab = self
            .tab
            .as_ref()
            .ok_or_else(|| crate::errors::BrowserAgentError::NoActiveTab)?;
        self.browser.take_screenshot(tab).await
    }

    async fn current_url(&self) -> Result<String> {
        let tab = self
            .tab
            .as_ref()
            .ok_or_else(|| crate::errors::BrowserAgentError::NoActiveTab)?;
        self.browser.get_url(tab).await
    }

    async fn close(&self) -> Result<()> {
        self.clear_element_highlights().await?;
        self.element_monitor
            .stop_monitoring(
                self.browser.as_ref(),
                self.tab
                    .as_ref()
                    .ok_or_else(|| crate::errors::BrowserAgentError::NoActiveTab)?,
            )
            .await?;
        Ok(())
    }
}
impl BrowserSession<crate::browser::ChromeBrowser> {
    /// Quick builder for common use cases
    pub async fn quick_start() -> Result<Self> {
        let config = Config::default();
        let browser = crate::browser::ChromeBrowser::new();
        Self::new(browser, config).await
    }

    /// Quick builder for demos with visible browser
    pub async fn demo_mode() -> Result<Self> {
        let mut config = Config::default();
        config.browser.headless = false;
        config.browser.viewport.width = 1920;
        config.browser.viewport.height = 1080;
        config.dom.enable_ai_labels = true;
        config.dom.extract_all_elements = true;
        config.features.enable_highlighting = true;
        config.features.enable_state_tracking = true;

        let browser = crate::browser::ChromeBrowser::new();
        let mut session = Self::new(browser, config).await?;
        session.set_auto_refresh(true);
        Ok(session)
    }

    /// Quick builder with custom config
    pub async fn with_config(config: Config) -> Result<Self> {
        let browser = crate::browser::ChromeBrowser::new();
        Self::new(browser, config).await
    }
}
