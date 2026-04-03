//! Gemini Emitter
//!
//! Emits Gemini-compatible structured Markdown format.

use async_trait::async_trait;

use crate::analyzer::ValidatedSkillIR;
use crate::error::EmitError;

use super::{Emitter, TargetPlatform};

/// Gemini Markdown emitter
pub struct GeminiEmitter;

impl GeminiEmitter {
    /// Create a new Gemini emitter
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Emitter for GeminiEmitter {
    fn target(&self) -> TargetPlatform {
        TargetPlatform::Gemini
    }

    async fn emit(&self, ir: &ValidatedSkillIR) -> Result<String, EmitError> {
        let inner = ir.as_ref();

        let mut output = String::new();

        // Title
        output.push_str(&format!("# {}\n\n", inner.name));

        // Metadata
        output.push_str(&format!("**Version:** {}\n\n", inner.version));
        output.push_str(&format!(
            "**Security Level:** {:?}\n\n",
            inner.security_level
        ));

        if inner.hitl_required {
            output.push_str(
                "> ⚠️ **HITL Required**: This skill requires human approval before execution.\n\n",
            );
        }

        // Description
        output.push_str("## Description\n\n");
        output.push_str(&inner.description);
        output.push_str("\n\n");

        // Procedures
        if !inner.procedures.is_empty() {
            output.push_str("## Execution Steps\n\n");
            for step in &inner.procedures {
                let marker = if step.is_critical { " [CRITICAL]" } else { "" };
                output.push_str(&format!("{}. {}{}\n", step.order, step.instruction, marker));
            }
            output.push_str("\n");
        }

        // Constraints
        if !inner.anti_skill_constraints.is_empty() {
            output.push_str("## Strict Constraints\n\n");
            for constraint in &inner.anti_skill_constraints {
                output.push_str(&format!("- {}\n", constraint.content));
            }
            output.push_str("\n");
        }

        Ok(output)
    }
}

impl Default for GeminiEmitter {
    fn default() -> Self {
        Self::new()
    }
}
