//! Claude Emitter
//!
//! Emits Claude-compatible XML format.

use async_trait::async_trait;

use crate::analyzer::ValidatedSkillIR;
use crate::error::EmitError;

use super::{Emitter, TargetPlatform};

/// Claude XML emitter
pub struct ClaudeEmitter;

impl ClaudeEmitter {
    /// Create a new Claude emitter
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Emitter for ClaudeEmitter {
    fn target(&self) -> TargetPlatform {
        TargetPlatform::Claude
    }

    async fn emit(&self, ir: &ValidatedSkillIR) -> Result<String, EmitError> {
        let inner = ir.as_ref();

        let mut output = String::new();
        output.push_str("<agent_skill>\n");
        output.push_str("  <metadata>\n");
        output.push_str(&format!("    <name>{}</name>\n", inner.name));
        output.push_str(&format!("    <version>{}</version>\n", inner.version));
        output.push_str("  </metadata>\n\n");
        output.push_str(&format!("  <intent>{}</intent>\n\n", inner.description));

        if inner.hitl_required {
            output.push_str("  <system_constraint>\n");
            output.push_str("    Wait for human explicit approval before execution.\n");
            output.push_str(
                "    This skill is marked as requiring Human-In-The-Loop (HITL) confirmation.\n",
            );
            output.push_str("  </system_constraint>\n\n");
        }

        if !inner.procedures.is_empty() {
            output.push_str("  <execution_steps>\n");
            for step in &inner.procedures {
                let critical_attr = if step.is_critical {
                    " critical=\"true\""
                } else {
                    ""
                };
                output.push_str(&format!(
                    "    <step order=\"{}\"{}>{}</step>\n",
                    step.order, critical_attr, step.instruction
                ));
            }
            output.push_str("  </execution_steps>\n\n");
        }

        if !inner.anti_skill_constraints.is_empty() {
            output.push_str("  <strict_constraints>\n");
            for constraint in &inner.anti_skill_constraints {
                output.push_str(&format!(
                    "    <anti_pattern source=\"{}\">{}</anti_pattern>\n",
                    constraint.source, constraint.content
                ));
            }
            output.push_str("  </strict_constraints>\n\n");
        }

        output.push_str("</agent_skill>\n");
        Ok(output)
    }
}

impl Default for ClaudeEmitter {
    fn default() -> Self {
        Self::new()
    }
}
