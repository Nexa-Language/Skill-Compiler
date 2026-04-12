//! Gemini Emitter - YAML Nested Data Optimization
//!
//! Implements format-specific optimization based on empirical research:
//! - Main instruction body: Markdown (best for Gemini's "meta-communication" protocol)
//! - Nested data structures: YAML (51.9% accuracy vs JSON 43.1%, Markdown 48.2%)
//!
//! Key insight: When IR contains deeply nested data (config templates, API responses),
//! automatically convert to YAML format for optimal Gemini parsing accuracy.
//!
//! Reference: "高级提示词工程格式与智能体技能架构" research report
//! - YAML nested data accuracy: 51.9% [48.8%, 55.0%]
//! - Markdown accuracy: 48.2% [45.1%, 51.3%]
//! - JSON accuracy: 43.1% [40.1%, 46.2%]
//! - XML accuracy: 33.8% [30.9%, 36.8%]

use async_trait::async_trait;
use serde_json::json;
use serde_yaml;

use crate::analyzer::ValidatedSkillIR;
use crate::error::EmitError;
use crate::ir::SkillIR;

use super::{Emitter, TargetPlatform};

/// Gemini emitter with YAML nested data optimization
pub struct GeminiEmitter {
    /// Threshold for detecting nested data (depth level)
    nested_data_threshold: usize,
}

impl GeminiEmitter {
    /// Create a new Gemini emitter with default settings
    #[must_use]
    pub fn new() -> Self {
        Self {
            nested_data_threshold: 3, // Convert to YAML if nesting depth >= 3
        }
    }

    /// Create emitter with custom nested data threshold
    #[must_use]
    pub fn with_threshold(threshold: usize) -> Self {
        Self {
            nested_data_threshold: threshold,
        }
    }

    /// Check if JSON value has deep nesting (>= threshold levels)
    fn is_deeply_nested(&self, value: &serde_json::Value, current_depth: usize) -> bool {
        if current_depth >= self.nested_data_threshold {
            return true;
        }

        match value {
            serde_json::Value::Object(map) => {
                for v in map.values() {
                    if self.is_deeply_nested(v, current_depth + 1) {
                        return true;
                    }
                }
                false
            }
            serde_json::Value::Array(arr) => {
                for v in arr {
                    if self.is_deeply_nested(v, current_depth + 1) {
                        return true;
                    }
                }
                false
            }
            _ => false,
        }
    }

    /// Convert JSON to YAML for better Gemini parsing of nested data
    fn json_to_yaml(&self, json_value: &serde_json::Value) -> Result<String, EmitError> {
        serde_yaml::to_string(json_value)
            .map_err(|e| EmitError::SerializationError(format!("YAML conversion failed: {}", e)))
    }

    /// Generate the Markdown instruction body with YAML optimization
    fn generate_markdown_body(&self, ir: &SkillIR) -> String {
        let mut output = String::new();

        // YAML Frontmatter (Agent Skills standard - progressive disclosure)
        output.push_str("---\n");
        output.push_str(&format!("name: {}\n", ir.name));
        let desc = if ir.description.len() > 1024 {
            &ir.description[..1024]
        } else {
            &ir.description
        };
        output.push_str(&format!("description: {}\n", desc.replace('<', "").replace('>', "")));
        output.push_str(&format!("version: {}\n", ir.version));
        if ir.hitl_required {
            output.push_str("hitl_required: true\n");
        }
        output.push_str(&format!("security_level: {:?}\n", ir.security_level).to_lowercase());
        if !ir.mcp_servers.is_empty() {
            output.push_str("mcp_servers:\n");
            for server in &ir.mcp_servers {
                output.push_str(&format!("  - {}\n", server));
            }
        }
        output.push_str("---\n\n");

        // Markdown body - structured for Gemini's meta-communication protocol
        // Use H1/H2 headers at top for priority understanding
        output.push_str(&format!("# {}\n\n", ir.name));

        // Identity section (role definition at top - Gemini best practice)
        output.push_str("## Identity\n\n");
        output.push_str("You are an AI assistant executing a structured skill workflow.\n\n");

        // Constraints section (boundary definition - must be at top per Gemini docs)
        output.push_str("## Constraints\n\n");
        output.push_str("Follow these boundaries strictly:\n\n");
        if ir.hitl_required {
            output.push_str("- **HITL Required**: Request user approval before critical operations\n");
        }
        match ir.security_level {
            crate::ir::SecurityLevel::Critical => {
                output.push_str("- **Security Level: CRITICAL**: All operations require explicit user consent\n");
            }
            crate::ir::SecurityLevel::High => {
                output.push_str("- **Security Level: HIGH**: Sensitive operations require verification\n");
            }
            crate::ir::SecurityLevel::Medium => {
                output.push_str("- **Security Level: MEDIUM**: Follow normal safety protocols\n");
            }
            crate::ir::SecurityLevel::Low => {
                output.push_str("- **Security Level: LOW**: Minimal restrictions, proceed with caution\n");
            }
        }
        output.push_str("\n");

        // Description
        output.push_str("## Description\n\n");
        output.push_str(&ir.description);
        output.push_str("\n\n");

        // Context Gathering (Gemini-specific section)
        output.push_str("## Context Gathering\n\n");
        output.push_str("Before execution, gather the following information:\n\n");
        if ir.input_schema.is_some() {
            output.push_str("- Review input parameters from `assets/input.yaml`\n");
        }
        output.push_str("- Verify all required dependencies are available\n");
        output.push_str("- Check system state and prerequisites\n\n");

        // Execution Steps (ordered list - Gemini prefers explicit sequencing)
        if !ir.procedures.is_empty() {
            output.push_str("## Execution Steps\n\n");
            output.push_str("Execute in exact sequence:\n\n");
            for step in &ir.procedures {
                let critical_marker = if step.is_critical {
                    " ⚠️ **[CRITICAL]**"
                } else {
                    ""
                };
                output.push_str(&format!(
                    "{}. {}{}\n",
                    step.order, step.instruction, critical_marker
                ));
            }
            output.push_str("\n");
        }

        // Anti-Skill constraints
        if !ir.anti_skill_constraints.is_empty() {
            output.push_str("## Strict Constraints\n\n");
            output.push_str("NEVER perform these actions:\n\n");
            for constraint in &ir.anti_skill_constraints {
                let level_marker = match constraint.level {
                    crate::ir::ConstraintLevel::Block => " 🔴 [BLOCK]",
                    crate::ir::ConstraintLevel::Error => " 🟠 [ERROR]",
                    crate::ir::ConstraintLevel::Warning => " 🟡 [WARNING]",
                };
                output.push_str(&format!("- {}{}\n", constraint.content, level_marker));
            }
            output.push_str("\n");
        }

        // Output format (reference to schema, not inline)
        output.push_str("## Output Format\n\n");
        output.push_str("After completion, format output according to `assets/output.yaml`.\n");
        output.push_str("For CI/CD pipeline integration, use `-p` non-interactive mode.\n\n");

        output
    }

    /// Generate YAML assets for nested data optimization
    fn generate_yaml_assets(&self, ir: &SkillIR) -> Vec<(String, String)> {
        let mut assets = Vec::new();

        // Convert input_schema to YAML if deeply nested
        if let Some(input_schema) = &ir.input_schema {
            let content = if self.is_deeply_nested(input_schema, 0) {
                // Use YAML for better Gemini parsing accuracy (51.9% vs 43.1% JSON)
                self.json_to_yaml(input_schema).unwrap_or_else(|_| {
                    serde_json::to_string_pretty(input_schema).unwrap_or_default()
                })
            } else {
                // Keep as JSON for simple structures
                serde_json::to_string_pretty(input_schema).unwrap_or_default()
            };
            assets.push(("assets/input.yaml".to_string(), content));
        }

        // Convert output_schema to YAML if deeply nested
        if let Some(output_schema) = &ir.output_schema {
            let content = if self.is_deeply_nested(output_schema, 0) {
                self.json_to_yaml(output_schema).unwrap_or_else(|_| {
                    serde_json::to_string_pretty(output_schema).unwrap_or_default()
                })
            } else {
                serde_json::to_string_pretty(output_schema).unwrap_or_default()
            };
            assets.push(("assets/output.yaml".to_string(), content));
        }

        // Generate config.yaml for any additional nested configuration
        let config = json!({
            "name": ir.name,
            "version": ir.version,
            "security_level": format!("{:?}", ir.security_level).to_lowercase(),
            "hitl_required": ir.hitl_required,
            "mcp_servers": ir.mcp_servers,
        });
        let config_yaml = self.json_to_yaml(&config).unwrap_or_default();
        assets.push(("assets/config.yaml".to_string(), config_yaml));

        assets
    }
}

#[async_trait]
impl Emitter for GeminiEmitter {
    fn target(&self) -> TargetPlatform {
        TargetPlatform::Gemini
    }

    async fn emit(&self, ir: &ValidatedSkillIR) -> Result<String, EmitError> {
        let inner = ir.as_ref();
        Ok(self.generate_markdown_body(inner))
    }

    fn requires_manifest(&self) -> bool {
        true
    }

    /// Generate YAML-optimized assets for nested data
    fn generate_assets(&self, ir: &ValidatedSkillIR) -> Vec<(String, String)> {
        let inner = ir.as_ref();
        self.generate_yaml_assets(inner)
    }
}

impl Default for GeminiEmitter {
    fn default() -> Self {
        Self::new()
    }
}