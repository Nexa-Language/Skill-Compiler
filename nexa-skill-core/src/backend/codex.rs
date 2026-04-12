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
//! This emitter generates ONLY Markdown output:
//! - Structured headers (H1/H2) aligned with GPT training corpus
//! - Ordered lists for step-by-step instructions
//! - Triple-quoted blocks for important content
//! - No JSON files - API layer handles structured output
//!
//! Reference: "高级提示词工程格式与智能体技能架构" research report

use async_trait::async_trait;

use crate::analyzer::ValidatedSkillIR;
use crate::error::EmitError;
use crate::ir::ConstraintLevel;
use crate::ir::SecurityLevel;
use crate::ir::SkillIR;

use super::{Emitter, TargetPlatform};

/// Codex emitter generating pure Markdown for GPT
pub struct CodexEmitter;

impl CodexEmitter {
    /// Create a new Codex emitter
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// Generate pure Markdown body
    /// Uses GPT-preferred format: headers, ordered lists, triple-quoted blocks
    fn generate_markdown_body(&self, ir: &SkillIR) -> String {
        let mut output = String::new();

        // === YAML Frontmatter (Agent Skills standard) ===
        output.push_str("---\n");
        output.push_str(&format!("name: {}\n", ir.name));
        
        // Description: limit to 1024 chars, remove XML tags for clean text
        let desc = if ir.description.len() > 1024 {
            &ir.description[..1024]
        } else {
            &ir.description
        };
        let clean_desc = desc.replace('<', "").replace('>', "");
        output.push_str(&format!("description: {}\n", clean_desc));
        output.push_str(&format!("version: {}\n", ir.version));
        
        if ir.hitl_required {
            output.push_str("hitl_required: true\n");
        }
        output.push_str(&format!("security_level: {}\n", ir.security_level.to_string().to_lowercase()));
        
        if !ir.mcp_servers.is_empty() {
            output.push_str("mcp_servers:\n");
            for server in &ir.mcp_servers {
                output.push_str(&format!("  - {}\n", server));
            }
        }
        output.push_str("---\n\n");

        // === Markdown Body (GPT-preferred format) ===
        // H1 title
        output.push_str(&format!("# {}\n\n", ir.name));

        // Identity (role definition at top - best practice)
        output.push_str("## Identity\n\n");
        output.push_str("You are an AI assistant executing a structured skill workflow.\n\n");

        // Outcome (clear result criteria)
        output.push_str("## Outcome\n\n");
        output.push_str("Define what constitutes successful completion of this skill.\n\n");

        // Constraints section
        output.push_str("## Constraints\n\n");
        output.push_str("Follow these boundaries strictly:\n\n");
        
        // Security level constraint
        let security_instruction = match ir.security_level {
            SecurityLevel::Critical => "MUST have human-in-the-loop approval. Auto-execution blocked.",
            SecurityLevel::High => "Requires human approval before execution.",
            SecurityLevel::Medium => "Follow normal safety protocols.",
            SecurityLevel::Low => "Standard execution allowed.",
        };
        output.push_str(&format!("- **Security Level: {}**: {}\n", 
            ir.security_level.to_string().to_uppercase(), security_instruction));
        
        // Anti-skill constraints
        for constraint in &ir.anti_skill_constraints {
            let level_marker = match constraint.level {
                ConstraintLevel::Block => " [BLOCK - Requires HITL]",
                ConstraintLevel::Error => " [ERROR - Must not proceed]",
                ConstraintLevel::Warning => " [WARNING]",
            };
            output.push_str(&format!("- {}{}\n", constraint.content, level_marker));
        }
        output.push_str("\n");

        // Description
        output.push_str("## Description\n\n");
        output.push_str(&ir.description);
        output.push_str("\n\n");

        // Context Gathering (pre-conditions)
        if !ir.context_gathering.is_empty() {
            output.push_str("## Context Gathering\n\n");
            output.push_str("Before execution, gather the following information:\n\n");
            for item in &ir.context_gathering {
                output.push_str(&format!("- {}\n", item));
            }
            output.push_str("\n");
        } else {
            output.push_str("## Context Gathering\n\n");
            output.push_str("Before execution, gather the following information:\n\n");
            output.push_str("- Verify all required dependencies are available\n");
            output.push_str("- Check system state and prerequisites\n\n");
        }

        // Execution Steps (ordered list - critical for GPT)
        if !ir.procedures.is_empty() {
            output.push_str("## Execution Steps\n\n");
            output.push_str("Execute in exact sequence:\n\n");
            for step in &ir.procedures {
                let critical_marker = if step.is_critical {
                    " **[CRITICAL - Requires HITL approval]**"
                } else {
                    ""
                };
                output.push_str(&format!("{}. {}{}\n", step.order, step.instruction, critical_marker));
            }
            output.push_str("\n");
        }

        // Few-shot Examples
        if !ir.few_shot_examples.is_empty() {
            output.push_str("## Examples\n\n");
            for example in &ir.few_shot_examples {
                if let Some(title) = &example.title {
                    output.push_str(&format!("### {}\n\n", title));
                }
                // Use triple-backtick for example content
                output.push_str("**User Input:**\n```\n");
                output.push_str(&example.user_input);
                output.push_str("\n```\n\n");
                output.push_str("**Agent Response:**\n```\n");
                output.push_str(&example.agent_response);
                output.push_str("\n```\n\n");
            }
        }

        // Edge Cases
        output.push_str("## Edge Cases\n\n");
        output.push_str("Handle these situations gracefully:\n\n");
        output.push_str("- If input is ambiguous: Ask user for clarification before proceeding\n");
        output.push_str("- If external API fails: Report error, do not fabricate data\n");
        output.push_str("- If required data missing: Halt and notify user\n\n");

        // Fallbacks
        if !ir.fallbacks.is_empty() {
            output.push_str("## Fallbacks\n\n");
            output.push_str("If primary approach fails:\n\n");
            for fallback in &ir.fallbacks {
                output.push_str(&format!("- {}\n", fallback));
            }
            output.push_str("\n");
        }

        // Output Format (natural language instruction, NOT JSON)
        output.push_str("## Output Format\n\n");
        output.push_str("After completion, format output as a clear, structured response.\n");
        output.push_str("Use appropriate formatting: headers, bullet points, or code blocks as needed.\n");
        output.push_str("For CI/CD pipeline integration, use `-p` non-interactive mode.\n\n");

        output
    }
}

#[async_trait]
impl Emitter for CodexEmitter {
    fn target(&self) -> TargetPlatform {
        TargetPlatform::Codex
    }

    async fn emit(&self, ir: &ValidatedSkillIR) -> Result<String, EmitError> {
        let inner = ir.as_ref();
        Ok(self.generate_markdown_body(inner))
    }

    fn requires_manifest(&self) -> bool {
        true
    }

    /// Codex does NOT generate JSON Schema assets
    /// JSON Schema is API layer's responsibility (Structured Outputs)
    fn generate_assets(&self, _ir: &ValidatedSkillIR) -> Vec<(String, String)> {
        // Return empty - no JSON files
        // API layer (OpenAI Structured Outputs) handles schema enforcement
        Vec::new()
    }
}

impl Default for CodexEmitter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::ProcedureStep;

    #[test]
    fn test_codex_generates_pure_markdown() {
        let ir = SkillIR {
            name: "test-skill".to_string(),
            description: "A test skill for unit testing".to_string(),
            version: "1.0.0".to_string(),
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

        let emitter = CodexEmitter::new();
        let output = emitter.generate_markdown_body(&ir);

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