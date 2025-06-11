use crate::actions::base::ActionContext;
use crate::actions::{Action, ActionError, ActionResult};
use crate::errors::Result;
use std::collections::HashMap;
use std::sync::Arc;

/// Registry for browser actions
pub struct ActionRegistry {
    actions: HashMap<String, Arc<dyn Action>>,
}

impl ActionRegistry {
    pub fn new() -> Self {
        Self {
            actions: HashMap::new(),
        }
    }

    /// Register a new action
    pub fn register<A: Action + 'static>(&mut self, action: A) {
        let name = action.name().to_string();
        self.actions.insert(name, Arc::new(action));
    }

    /// Get an action by name
    pub fn get_action(&self, name: &str) -> Option<Arc<dyn Action>> {
        self.actions.get(name).cloned()
    }

    /// List all registered actions
    pub fn list_actions(&self) -> Vec<String> {
        self.actions.keys().cloned().collect()
    }

    /// Execute an action by name
    pub async fn execute_action(
        &self,
        name: &str,
        params: serde_json::Value,
        context: &ActionContext,
    ) -> Result<ActionResult> {
        let action = self.get_action(name).ok_or_else(|| {
            crate::errors::BrowserAgentError::ActionError(ActionError::ActionNotFound(
                name.to_string(),
            ))
        })?;

        // Validate parameters
        action.validate_params(&params).map_err(|e| {
            crate::errors::BrowserAgentError::ActionError(ActionError::InvalidParameters(
                e.to_string(),
            ))
        })?;

        // Execute action with timing
        let start_time = std::time::Instant::now();
        let result = action.execute(params, context).await?;
        let execution_time = start_time.elapsed().as_millis() as u64;

        Ok(result.with_execution_time(execution_time))
    }

    /// Get action metadata
    pub fn get_action_metadata(&self, name: &str) -> Option<ActionMetadata> {
        self.get_action(name).map(|action| ActionMetadata {
            name: action.name().to_string(),
            description: action.description().to_string(),
            parameter_schema: action.parameter_schema(),
        })
    }

    /// Get metadata for all actions
    pub fn get_all_metadata(&self) -> Vec<ActionMetadata> {
        self.actions
            .values()
            .map(|action| ActionMetadata {
                name: action.name().to_string(),
                description: action.description().to_string(),
                parameter_schema: action.parameter_schema(),
            })
            .collect()
    }
}

impl Default for ActionRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Metadata about an action
#[derive(Debug, Clone)]
pub struct ActionMetadata {
    pub name: String,
    pub description: String,
    pub parameter_schema: serde_json::Value,
}
