//! Codex Emitter
//!
//! Emits OpenAI Codex-compatible JSON Schema format.

use async_trait::async_trait;
use serde_json::json;

use crate::analyzer::ValidatedSkillIR;
use crate::error::EmitError;

use super::{Emitter, TargetPlatform};

/// Codex JSON Schema emitter
pub struct CodexEmitter;

impl CodexEmitter {
    /// Create a new Codex emitter
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Emitter for CodexEmitter {
    fn target(&self) -> TargetPlatform {
        TargetPlatform::Codex
    }

    async fn emit(&self, ir: &ValidatedSkillIR) -> Result<String, EmitError> {
        let inner = ir.as_ref();

        // Build expanded description
        let mut description = inner.description.clone();
        if !inner.procedures.is_empty() {
            description.push_str("\n\nExecution Steps:\n");
            for step in &inner.procedures {
                description.push_str(&format!("{}. {}\n", step.order, step.instruction));
            }
        }

        let schema = json!({
            "name": inner.name,
            "description": description,
            "parameters": inner.input_schema.clone().unwrap_or_else(|| json!({
                "type": "object",
                "properties": {},
                "required": []
            })),
            "metadata": {
                "version": inner.version,
                "hitl_required": inner.hitl_required,
                "security_level": format!("{:?}", inner.security_level).to_lowercase(),
                "mcp_servers": &inner.mcp_servers,
            }
        });

        serde_json::to_string_pretty(&schema)
            .map_err(|e| EmitError::SerializationError(e.to_string()))
    }
}

impl Default for CodexEmitter {
    fn default() -> Self {
        Self::new()
    }
}
