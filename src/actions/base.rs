use crate::errors::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Result of an action execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionResult {
    pub success: bool,
    pub message: String,
    pub data: Option<serde_json::Value>,
    pub execution_time_ms: u64,
}

impl ActionResult {
    pub fn success(message: String) -> Self {
        Self {
            success: true,
            message,
            data: None,
            execution_time_ms: 0,
        }
    }

    pub fn success_with_data(message: String, data: serde_json::Value) -> Self {
        Self {
            success: true,
            message,
            data: Some(data),
            execution_time_ms: 0,
        }
    }

    pub fn failure(message: String) -> Self {
        Self {
            success: false,
            message,
            data: None,
            execution_time_ms: 0,
        }
    }

    pub fn with_execution_time(mut self, time_ms: u64) -> Self {
        self.execution_time_ms = time_ms;
        self
    }
}

/// Error types for actions
#[derive(Debug, thiserror::Error)]
pub enum ActionError {
    #[error("Action not found: {0}")]
    ActionNotFound(String),

    #[error("Invalid parameters: {0}")]
    InvalidParameters(String),

    #[error("Execution failed: {0}")]
    ExecutionFailed(String),

    #[error("Timeout: {0}")]
    Timeout(String),
}

/// Base trait for all browser actions
#[async_trait]
pub trait Action: Send + Sync {
    /// Name of the action
    fn name(&self) -> &str;

    /// Description of what the action does
    fn description(&self) -> &str;

    /// Parameter schema for validation
    fn parameter_schema(&self) -> serde_json::Value;

    /// Execute the action
    async fn execute(
        &self,
        params: serde_json::Value,
        context: &ActionContext,
    ) -> Result<ActionResult>;

    /// Validate parameters before execution
    fn validate_params(&self, params: &serde_json::Value) -> Result<()> {
        // Default implementation - can be overridden
        Ok(())
    }
}

/// Context provided to actions during execution
#[derive(Debug)]
pub struct ActionContext {
    pub session_id: String,
    pub browser_state: Option<crate::dom::DomState>,
    pub variables: HashMap<String, serde_json::Value>,
    pub timeout_ms: u64,
}

impl ActionContext {
    pub fn new(session_id: String) -> Self {
        Self {
            session_id,
            browser_state: None,
            variables: HashMap::new(),
            timeout_ms: 30000,
        }
    }

    pub fn with_browser_state(mut self, state: crate::dom::DomState) -> Self {
        self.browser_state = Some(state);
        self
    }

    pub fn with_variable(mut self, key: String, value: serde_json::Value) -> Self {
        self.variables.insert(key, value);
        self
    }

    pub fn with_timeout(mut self, timeout_ms: u64) -> Self {
        self.timeout_ms = timeout_ms;
        self
    }
}
