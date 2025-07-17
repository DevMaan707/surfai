use crate::{BrowserManager, ElementInteractor, HumanInteractor, PageScraper};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPError {
    pub code: i32,
    pub message: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPRequest {
    pub id: String,
    pub method: String,
    pub params: serde_json::Value,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPResponse {
    pub id: String,
    pub result: Option<serde_json::Value>,
    pub error: Option<MCPError>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPBrowserSession {
    pub id: String,
    pub headless: bool,
    pub stealth: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPTool {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
}

pub struct MCPBrowserServer {
    sessions: Arc<RwLock<HashMap<String, BrowserManager>>>,
    tools: Vec<MCPTool>,
}
impl MCPBrowserServer {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            tools: vec![
                MCPTool {
                    name: "create_browser_session".to_string(),
                    description: "Create a new browser session".to_string(),
                    input_schema: serde_json::json!({
                        "type": "object",
                        "properties": {
                            "headless": {"type": "boolean", "default": true},
                            "stealth": {"type": "boolean", "default": false}
                        }
                    }),
                },
                MCPTool {
                    name: "navigate_to_url".to_string(),
                    description: "Navigate to a specific URL".to_string(),
                    input_schema: serde_json::json!({
                        "type": "object",
                        "properties": {
                            "session_id": {"type": "string"},
                            "url": {"type": "string"}
                        },
                        "required": ["session_id", "url"]
                    }),
                },
                MCPTool {
                    name: "search_google".to_string(),
                    description: "Perform a Google search with human-like behavior".to_string(),
                    input_schema: serde_json::json!({
                        "type": "object",
                        "properties": {
                            "session_id": {"type": "string"},
                            "query": {"type": "string"}
                        },
                        "required": ["session_id", "query"]
                    }),
                },
                MCPTool {
                    name: "get_page_elements".to_string(),
                    description: "Get all interactable elements on the current page".to_string(),
                    input_schema: serde_json::json!({
                        "type": "object",
                        "properties": {
                            "session_id": {"type": "string"},
                            "element_types": {"type": "array", "items": {"type": "string"}}
                        },
                        "required": ["session_id"]
                    }),
                },
                MCPTool {
                    name: "click_element".to_string(),
                    description: "Click on an element with human-like behavior".to_string(),
                    input_schema: serde_json::json!({
                        "type": "object",
                        "properties": {
                            "session_id": {"type": "string"},
                            "selector": {"type": "string"}
                        },
                        "required": ["session_id", "selector"]
                    }),
                },
                MCPTool {
                    name: "type_text".to_string(),
                    description: "Type text into an element with human-like behavior".to_string(),
                    input_schema: serde_json::json!({
                        "type": "object",
                        "properties": {
                            "session_id": {"type": "string"},
                            "selector": {"type": "string"},
                            "text": {"type": "string"}
                        },
                        "required": ["session_id", "selector", "text"]
                    }),
                },
                MCPTool {
                    name: "scrape_page".to_string(),
                    description: "Scrape all data from the current page".to_string(),
                    input_schema: serde_json::json!({
                        "type": "object",
                        "properties": {
                            "session_id": {"type": "string"}
                        },
                        "required": ["session_id"]
                    }),
                },
                MCPTool {
                    name: "take_screenshot".to_string(),
                    description: "Take a screenshot of the current page".to_string(),
                    input_schema: serde_json::json!({
                        "type": "object",
                        "properties": {
                            "session_id": {"type": "string"}
                        },
                        "required": ["session_id"]
                    }),
                },
                MCPTool {
                    name: "close_session".to_string(),
                    description: "Close a browser session".to_string(),
                    input_schema: serde_json::json!({
                        "type": "object",
                        "properties": {
                            "session_id": {"type": "string"}
                        },
                        "required": ["session_id"]
                    }),
                },
            ],
        }
    }
    async fn handle_tools_list(&self, request: MCPRequest) -> MCPResponse {
        MCPResponse {
            id: request.id,
            result: Some(serde_json::json!({
                "tools": self.tools
            })),
            error: None,
        }
    }
    async fn handle_tools_call(&self, request: MCPRequest) -> MCPResponse {
        let tool_name = request.params.get("name").and_then(|v| v.as_str());
        let arguments = request
            .params
            .get("arguments")
            .cloned()
            .unwrap_or(serde_json::json!({}));

        match tool_name {
            Some("create_browser_session") => {
                self.create_browser_session(request.id, arguments).await
            }
            Some("navigate_to_url") => self.navigate_to_url(request.id, arguments).await,
            Some("search_google") => self.search_google(request.id, arguments).await,
            Some("get_page_elements") => self.get_page_elements(request.id, arguments).await,
            Some("click_element") => self.click_element(request.id, arguments).await,
            Some("type_text") => self.type_text(request.id, arguments).await,
            Some("scrape_page") => self.scrape_page(request.id, arguments).await,
            Some("take_screenshot") => self.take_screenshot(request.id, arguments).await,
            Some("close_session") => self.close_session(request.id, arguments).await,
            _ => MCPResponse {
                id: request.id,
                result: None,
                error: Some(MCPError {
                    code: -32602,
                    message: "Unknown tool".to_string(),
                }),
            },
        }
    }
    pub async fn handle_request(&self, request: MCPRequest) -> MCPResponse {
        match request.method.as_str() {
            "tools/list" => self.handle_tools_list(request).await,
            "tools/call" => self.handle_tools_call(request).await,
            _ => MCPResponse {
                id: request.id,
                result: None,
                error: Some(MCPError {
                    code: -32601,
                    message: format!("Method not found: {}", request.method),
                }),
            },
        }
    }
    async fn search_google(&self, id: String, args: serde_json::Value) -> MCPResponse {
        let session_id = match args.get("session_id").and_then(|v| v.as_str()) {
            Some(id) => id,
            None => {
                return MCPResponse {
                    id,
                    result: None,
                    error: Some(MCPError {
                        code: -32602,
                        message: "Missing session id".to_string(),
                    }),
                }
            }
        };
        let sessions = self.sessions.read().await;
        let query = match args.get("query").and_then(|v| v.as_str()) {
            Some(query) => query,
            None => {
                return MCPResponse {
                    id,
                    result: None,
                    error: Some(MCPError {
                        code: -32602,
                        message: "Missing session id".to_string(),
                    }),
                }
            }
        };
        let browser = match sessions.get(session_id) {
            Some(browser) => browser,
            None => {
                return MCPResponse {
                    id,
                    result: None,
                    error: Some(MCPError {
                        code: -32602,
                        message: "Missing session id".to_string(),
                    }),
                }
            }
        };
        match browser.new_page("https://www.google.com").await {
            Ok(page) => {
                let human_interactor = HumanInteractor::new(page.clone());
                //let element_interactor = ElementInteractor::new(page.clone());
                if let Err(e) = human_interactor.apply_stealth().await {
                    return MCPResponse {
                        id,
                        result: None,
                        error: Some(MCPError {
                            code: -32602,
                            message: format!("Stealth setup failed: {}", e),
                        }),
                    };
                }
                match human_interactor.search_with_realistic_pattern(query).await {
                    Ok(_) => match human_interactor.submit_search().await {
                        Ok(_) => {
                            if let Err(e) = human_interactor.wait_for_results(30).await {
                                return MCPResponse {
                                    id,
                                    result: None,
                                    error: Some(MCPError {
                                        code: -3030,
                                        message: format!("Search timeout: {}", e),
                                    }),
                                };
                            }

                            match human_interactor.extract_search_results().await {
                                Ok(results) => MCPResponse {
                                    id,
                                    result: Some(serde_json::json!({
                                        "success": true,
                                        "query": query,
                                        "results": results
                                    })),
                                    error: None,
                                },
                                Err(e) => {
                                    return MCPResponse {
                                        id,
                                        result: None,
                                        error: Some(MCPError {
                                            code: -3026,
                                            message: format!("Failed to extract results: {}", e),
                                        }),
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            return MCPResponse {
                                id,
                                result: None,
                                error: Some(MCPError {
                                    code: -3027,
                                    message: format!("Failed to search: {}", e),
                                }),
                            }
                        }
                    },
                    Err(e) => {
                        return MCPResponse {
                            id,
                            result: None,
                            error: Some(MCPError {
                                code: -3028,
                                message: format!("Failed to navigate to URL: {}", e),
                            }),
                        }
                    }
                }
            }
            Err(e) => {
                return MCPResponse {
                    id,
                    result: None,
                    error: Some(MCPError {
                        code: -3029,
                        message: format!("Failed to navigate to URL: {}", e),
                    }),
                }
            }
        }
    }
    async fn navigate_to_url(&self, id: String, args: serde_json::Value) -> MCPResponse {
        let session_id = match args.get("session_id").and_then(|v| v.as_str()) {
            Some(id) => id,
            None => {
                return MCPResponse {
                    id,
                    result: None,
                    error: Some(MCPError {
                        code: -32602,
                        message: "Missing session id".to_string(),
                    }),
                }
            }
        };
        let url = match args.get("url").and_then(|v| v.as_str()) {
            Some(url) => url,
            None => {
                return MCPResponse {
                    id,
                    result: None,
                    error: Some(MCPError {
                        code: -32602,
                        message: "Missing url".to_string(),
                    }),
                }
            }
        };

        let sessions = self.sessions.read().await;
        let browser = match sessions.get(session_id) {
            Some(browser) => browser,
            None => {
                return MCPResponse {
                    id,
                    result: None,
                    error: Some(MCPError {
                        code: -32602,
                        message: "Missing url".to_string(),
                    }),
                }
            }
        };
        match browser.new_page(url).await {
            Ok(page) => {
                let current_url = page.url().await.unwrap_or_else(|_| Some(url.to_string()));
                MCPResponse {
                    id,
                    result: Some(serde_json::json!({
                        "success": true,
                        "url": current_url
                    })),
                    error: None,
                }
            }
            Err(e) => MCPResponse {
                id,
                result: None,
                error: Some(MCPError {
                    code: -32603,
                    message: format!("Navigation failed: {}", e),
                }),
            },
        }
    }
    async fn create_browser_session(&self, id: String, args: serde_json::Value) -> MCPResponse {
        let headless: bool = args
            .clone()
            .get("headless")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);
        let stealth: bool = args
            .get("stealth")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);
        let session_id = Uuid::new_v4().to_string();
        let browser_result = if stealth {
            BrowserManager::new_stealth_browser(headless).await
        } else {
            BrowserManager::new(headless).await
        };

        match browser_result {
            Ok(browser) => {
                let mut sessions = self.sessions.write().await;
                sessions.insert(session_id.clone(), browser);

                MCPResponse {
                    id,
                    result: Some(serde_json::json!({
                        "session_id": session_id,
                        "headless": headless,
                        "stealth": stealth,
                        "created_at": chrono::Utc::now()
                    })),
                    error: None,
                }
            }
            Err(e) => MCPResponse {
                id,
                result: None,
                error: Some(MCPError {
                    code: -32603,
                    message: format!("Failed to create browser session: {}", e),
                }),
            },
        }
    }
    async fn get_page_elements(&self, id: String, args: serde_json::Value) -> MCPResponse {
        let session_id = match args.get("session_id").and_then(|v| v.as_str()) {
            Some(id) => id,
            None => return self.error_response(id, "Missing session_id"),
        };

        let element_types = args
            .get("element_types")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect()
            });

        let sessions = self.sessions.read().await;
        let browser = match sessions.get(session_id) {
            Some(browser) => browser,
            None => return self.error_response(id, "Session not found"),
        };

        // Get current page (assuming last created page)
        match browser.new_page("about:blank").await {
            Ok(page) => {
                let element_interactor = ElementInteractor::new(page);

                match element_interactor.label_elements(element_types).await {
                    Ok(elements) => MCPResponse {
                        id,
                        result: Some(serde_json::json!({
                            "success": true,
                            "elements": elements
                        })),
                        error: None,
                    },
                    Err(e) => self.error_response(id, &format!("Failed to get elements: {}", e)),
                }
            }
            Err(e) => self.error_response(id, &format!("Failed to get page: {}", e)),
        }
    }

    async fn click_element(&self, id: String, args: serde_json::Value) -> MCPResponse {
        let session_id = match args.get("session_id").and_then(|v| v.as_str()) {
            Some(id) => id,
            None => return self.error_response(id, "Missing session_id"),
        };

        let selector = match args.get("selector").and_then(|v| v.as_str()) {
            Some(selector) => selector,
            None => return self.error_response(id, "Missing selector"),
        };

        let sessions = self.sessions.read().await;
        let browser = match sessions.get(session_id) {
            Some(browser) => browser,
            None => return self.error_response(id, "Session not found"),
        };

        match browser.new_page("about:blank").await {
            Ok(page) => {
                let human_interactor = HumanInteractor::new(page);

                match human_interactor.click_human(selector).await {
                    Ok(_) => MCPResponse {
                        id,
                        result: Some(serde_json::json!({
                            "success": true,
                            "selector": selector
                        })),
                        error: None,
                    },
                    Err(e) => self.error_response(id, &format!("Click failed: {}", e)),
                }
            }
            Err(e) => self.error_response(id, &format!("Failed to get page: {}", e)),
        }
    }

    async fn type_text(&self, id: String, args: serde_json::Value) -> MCPResponse {
        let session_id = match args.get("session_id").and_then(|v| v.as_str()) {
            Some(id) => id,
            None => return self.error_response(id, "Missing session_id"),
        };

        let selector = match args.get("selector").and_then(|v| v.as_str()) {
            Some(selector) => selector,
            None => return self.error_response(id, "Missing selector"),
        };

        let text = match args.get("text").and_then(|v| v.as_str()) {
            Some(text) => text,
            None => return self.error_response(id, "Missing text"),
        };

        let sessions = self.sessions.read().await;
        let browser = match sessions.get(session_id) {
            Some(browser) => browser,
            None => return self.error_response(id, "Session not found"),
        };

        match browser.new_page("about:blank").await {
            Ok(page) => {
                let human_interactor = HumanInteractor::new(page);

                match human_interactor.type_human(selector, text).await {
                    Ok(_) => MCPResponse {
                        id,
                        result: Some(serde_json::json!({
                            "success": true,
                            "selector": selector,
                            "text": text
                        })),
                        error: None,
                    },
                    Err(e) => self.error_response(id, &format!("Type failed: {}", e)),
                }
            }
            Err(e) => self.error_response(id, &format!("Failed to get page: {}", e)),
        }
    }

    async fn scrape_page(&self, id: String, args: serde_json::Value) -> MCPResponse {
        let session_id = match args.get("session_id").and_then(|v| v.as_str()) {
            Some(id) => id,
            None => return self.error_response(id, "Missing session_id"),
        };

        let sessions = self.sessions.read().await;
        let browser = match sessions.get(session_id) {
            Some(browser) => browser,
            None => return self.error_response(id, "Session not found"),
        };

        match browser.new_page("about:blank").await {
            Ok(page) => {
                let scraper = PageScraper::new(page);

                match scraper.scrape_page().await {
                    Ok(page_data) => MCPResponse {
                        id,
                        result: Some(serde_json::to_value(page_data).unwrap()),
                        error: None,
                    },
                    Err(e) => self.error_response(id, &format!("Scraping failed: {}", e)),
                }
            }
            Err(e) => self.error_response(id, &format!("Failed to get page: {}", e)),
        }
    }

    async fn take_screenshot(&self, id: String, args: serde_json::Value) -> MCPResponse {
        let session_id = match args.get("session_id").and_then(|v| v.as_str()) {
            Some(id) => id,
            None => return self.error_response(id, "Missing session_id"),
        };

        let sessions = self.sessions.read().await;
        let browser = match sessions.get(session_id) {
            Some(browser) => browser,
            None => return self.error_response(id, "Session not found"),
        };

        match browser.new_page("about:blank").await {
            Ok(page) => {
                let scraper = PageScraper::new(page);

                match scraper.take_screenshot().await {
                    Ok(screenshot_data) => {
                        let base64_data = base64::encode(screenshot_data);
                        MCPResponse {
                            id,
                            result: Some(serde_json::json!({
                                "success": true,
                                "screenshot": base64_data,
                                "format": "png"
                            })),
                            error: None,
                        }
                    }
                    Err(e) => self.error_response(id, &format!("Screenshot failed: {}", e)),
                }
            }
            Err(e) => self.error_response(id, &format!("Failed to get page: {}", e)),
        }
    }

    async fn close_session(&self, id: String, args: serde_json::Value) -> MCPResponse {
        let session_id = match args.get("session_id").and_then(|v| v.as_str()) {
            Some(id) => id,
            None => return self.error_response(id, "Missing session_id"),
        };

        let mut sessions = self.sessions.write().await;
        match sessions.remove(session_id) {
            Some(browser) => match browser.close().await {
                Ok(_) => MCPResponse {
                    id,
                    result: Some(serde_json::json!({
                        "success": true,
                        "session_id": session_id
                    })),
                    error: None,
                },
                Err(e) => self.error_response(id, &format!("Failed to close browser: {}", e)),
            },
            None => self.error_response(id, "Session not found"),
        }
    }

    fn error_response(&self, id: String, message: &str) -> MCPResponse {
        MCPResponse {
            id,
            result: None,
            error: Some(MCPError {
                code: -32603,
                message: message.to_string(),
            }),
        }
    }
}
