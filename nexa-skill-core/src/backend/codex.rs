//! Codex Emitter - Pure Markdown Output for GPT
//!
//! Implements the "Decoupled Reasoning and Formatting" architecture based on
//! empirical research showing JSON format tax causes up to 40% performance degradation.
//!
//! Key insight from research (2025):
//! - GPT models should NOT parse JSON input prompts
//! - Markdown input saves 34-38% tokens compared to JSON
//! - JSON Schema enforcement is API layer's responsibility, NOT compiler's
//!
//! This emitter generates ONLY Markdown output using Askama templates:
//! - Structured headers (H1/H2) aligned with GPT training corpus
//! - Ordered lists for step-by-step instructions
//! - Triple-quoted blocks for important content
//! - No JSON files - API layer handles structured output
//!
//! Reference: "高级提示词工程格式与智能体技能架构" research report

use askama::Template;
use nexa_skill_templates::{CodexContext, ConstraintContext, ExampleContext, StepContext};

use crate::analyzer::ValidatedSkillIR;
use crate::error::EmitError;
use crate::ir::{ConstraintLevel, SecurityLevel, SkillIR};

use super::{Emitter, TargetPlatform};

/// Codex emitter generating pure Markdown for GPT
pub struct CodexEmitter;

impl CodexEmitter {
    /// Create a new Codex emitter
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl Emitter for CodexEmitter {
    fn target(&self) -> TargetPlatform {
        TargetPlatform::Codex
    }

    fn emit(&self, ir: &ValidatedSkillIR) -> Result<String, EmitError> {
        let inner = ir.as_ref();
        let context = self.build_context(inner);
        context
            .render()
            .map_err(|e| EmitError::TemplateError(format!("Template render failed: {}", e)))
    }

    fn requires_manifest(&self) -> bool {
        true
    }

    /// Codex does NOT generate JSON Schema assets
    /// JSON Schema is API layer's responsibility (Structured Outputs)
    fn generate_assets(&self, _ir: &ValidatedSkillIR) -> Vec<(String, String)> {
        Vec::new()
    }
}

impl Default for CodexEmitter {
    fn default() -> Self {
        Self::new()
    }
}

impl CodexEmitter {
    /// Build CodexContext from SkillIR
    fn build_context(&self, ir: &SkillIR) -> CodexContext {
        // Description: limit to 1024 chars, remove XML tags for clean text
        let desc = if ir.description.len() > 1024 {
            &ir.description[..1024]
        } else {
            &ir.description
        };
        let clean_description = desc.replace('<', "").replace('>', "");

        let security_instruction = match ir.security_level {
            SecurityLevel::Critical => {
                "MUST have human-in-the-loop approval. Auto-execution blocked."
            }
            SecurityLevel::High => "Requires human approval before execution.",
            SecurityLevel::Medium => "Follow normal safety protocols.",
            SecurityLevel::Low => "Standard execution allowed.",
        };

        // Context gathering: use defaults when IR has no custom items
        let context_gathering = if ir.context_gathering.is_empty() {
            vec![
                "Verify all required dependencies are available".to_string(),
                "Check system state and prerequisites".to_string(),
            ]
        } else {
            ir.context_gathering.clone()
        };

        CodexContext {
            name: ir.name.to_string(),
            version: ir.version.to_string(),
            clean_description,
            description: ir.description.clone(),
            hitl_required: ir.hitl_required,
            security_level: ir.security_level.to_string(),
            security_level_upper: ir.security_level.to_string().to_uppercase(),
            security_instruction: security_instruction.to_string(),
            mcp_servers: ir.mcp_servers.iter().map(|s| s.to_string()).collect(),
            has_input_schema: ir.input_schema.is_some(),
            has_output_schema: ir.output_schema.is_some(),
            procedures: ir
                .procedures
                .iter()
                .map(|s| StepContext {
                    order: s.order,
                    instruction: s.instruction.clone(),
                    is_critical: s.is_critical,
                })
                .collect(),
            anti_skill_constraints: ir
                .anti_skill_constraints
                .iter()
                .map(|c| ConstraintContext {
                    source: c.source.to_string(),
                    content: c.content.clone(),
                    level_marker: match c.level {
                        ConstraintLevel::Block => " [BLOCK - Requires HITL]".to_string(),
                        ConstraintLevel::Error => " [ERROR - Must not proceed]".to_string(),
                        ConstraintLevel::Warning => " [WARNING]".to_string(),
                    },
                })
                .collect(),
            context_gathering,
            examples: ir
                .few_shot_examples
                .iter()
                .map(|e| ExampleContext {
                    title: e.title.clone().unwrap_or_default(),
                    user_input: e.user_input.clone(),
                    agent_response: e.agent_response.clone(),
                })
                .collect(),
            fallbacks: ir.fallbacks.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;
    use crate::analyzer::ValidatedSkillIR;
    use crate::ir::{ProcedureStep, SkillIR};

    #[test]
    fn test_codex_generates_pure_markdown() {
        let ir = SkillIR {
            name: Arc::from("test-skill"),
            description: "A test skill for unit testing".to_string(),
            version: Arc::from("1.0.0"),
            security_level: SecurityLevel::Medium,
            procedures: vec![ProcedureStep {
                order: 1,
                instruction: "First step".to_string(),
                is_critical: false,
                on_error: None,
                constraints: vec![],
                expected_output: None,
            }],
            ..Default::default()
        };

        let validated = ValidatedSkillIR::new(ir);
        let emitter = CodexEmitter::new();
        let output = emitter.emit(&validated).unwrap();

        // Verify Markdown structure
        assert!(output.contains("# test-skill")); // H1 title
        assert!(output.contains("## Identity")); // Role definition
        assert!(output.contains("## Outcome")); // Result criteria
        assert!(output.contains("---")); // YAML frontmatter
        assert!(output.contains("1. First step")); // Ordered list
        assert!(!output.contains("assets/")); // No asset references
        assert!(!output.contains(".json")); // No JSON references
    }

    #[test]
    fn test_codex_no_json_assets() {
        let ir = SkillIR::default();
        let validated = ValidatedSkillIR::new(ir);
        let emitter = CodexEmitter::new();
        let assets = emitter.generate_assets(&validated);

        // Codex should return empty assets list
        assert!(assets.is_empty());
    }
}