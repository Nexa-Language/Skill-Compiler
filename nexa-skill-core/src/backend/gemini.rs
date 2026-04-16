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

use serde_json::json;

use askama::Template;
use nexa_skill_templates::{ConstraintContext, GeminiContext, StepContext};

use crate::analyzer::ValidatedSkillIR;
use crate::error::EmitError;
use crate::ir::{ConstraintLevel, SecurityLevel, SkillIR};

use super::{Emitter, TargetPlatform};

/// Gemini emitter with YAML nested data optimization
///
/// Uses `requires_yaml_optimization` flag from SkillIR (computed by
/// NestedDataDetector in Analyzer phase) to decide format, instead of
/// local runtime computation.
pub struct GeminiEmitter;

impl GeminiEmitter {
    /// Create a new Gemini emitter
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl Emitter for GeminiEmitter {
    fn target(&self) -> TargetPlatform {
        TargetPlatform::Gemini
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

impl GeminiEmitter {
    /// Build GeminiContext from SkillIR
    fn build_context(&self, ir: &SkillIR) -> GeminiContext {
        // Description: limit to 1024 chars, remove XML tags for clean text
        let desc = if ir.description.len() > 1024 {
            &ir.description[..1024]
        } else {
            &ir.description
        };
        let clean_description = desc.replace('<', "").replace('>', "");

        let (security_level_display, security_instruction) = match ir.security_level {
            SecurityLevel::Critical => ("CRITICAL", "All operations require explicit user consent"),
            SecurityLevel::High => ("HIGH", "Sensitive operations require verification"),
            SecurityLevel::Medium => ("MEDIUM", "Follow normal safety protocols"),
            SecurityLevel::Low => ("LOW", "Minimal restrictions, proceed with caution"),
        };

        GeminiContext {
            name: ir.name.to_string(),
            version: ir.version.to_string(),
            clean_description,
            description: ir.description.clone(),
            hitl_required: ir.hitl_required,
            security_level: ir.security_level.to_string(),
            security_level_display: security_level_display.to_string(),
            security_instruction: security_instruction.to_string(),
            mcp_servers: ir.mcp_servers.iter().map(|s| s.to_string()).collect(),
            has_input_schema: ir.input_schema.is_some(),
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
                        ConstraintLevel::Block => " 🔴 [BLOCK]".to_string(),
                        ConstraintLevel::Error => " 🟠 [ERROR]".to_string(),
                        ConstraintLevel::Warning => " 🟡 [WARNING]".to_string(),
                    },
                })
                .collect(),
        }
    }

    /// Convert JSON to YAML for better Gemini parsing of nested data
    fn json_to_yaml(&self, json_value: &serde_json::Value) -> Result<String, EmitError> {
        serde_yaml::to_string(json_value)
            .map_err(|e| EmitError::SerializationError(format!("YAML conversion failed: {}", e)))
    }

    /// Generate YAML assets for nested data optimization
    fn generate_yaml_assets(&self, ir: &SkillIR) -> Vec<(String, String)> {
        let mut assets = Vec::new();

        // Convert input_schema to YAML if requires_yaml_optimization
        if let Some(input_schema) = &ir.input_schema {
            let content = if ir.requires_yaml_optimization {
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

        // Convert output_schema to YAML if requires_yaml_optimization
        if let Some(output_schema) = &ir.output_schema {
            let content = if ir.requires_yaml_optimization {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analyzer::ValidatedSkillIR;
    use crate::backend::Emitter;
    use crate::ir::{ProcedureStep, SecurityLevel, SkillIR};
    use std::sync::Arc;

    fn make_step(order: u32, instruction: &str) -> ProcedureStep {
        ProcedureStep {
            order,
            instruction: instruction.to_string(),
            is_critical: false,
            constraints: vec![],
            expected_output: None,
            on_error: None,
        }
    }

    fn make_test_ir() -> SkillIR {
        SkillIR {
            name: Arc::from("test-skill"),
            version: Arc::from("1.0.0"),
            description: "Test description".to_string(),
            security_level: SecurityLevel::Medium,
            procedures: vec![make_step(1, "Step 1")],
            ..Default::default()
        }
    }

    #[test]
    fn test_gemini_emitter_markdown_output() {
        let ir = make_test_ir();
        let validated = ValidatedSkillIR::new(ir);
        let emitter = GeminiEmitter::new();
        let result = emitter.emit(&validated).unwrap();
        assert!(result.contains("# test-skill"));
        assert!(result.contains("## Description"));
        assert!(result.contains("Test description"));
        assert!(result.contains("## Execution Steps"));
    }

    #[test]
    fn test_gemini_yaml_assets() {
        let ir = SkillIR {
            name: Arc::from("nested-skill"),
            version: Arc::from("1.0.0"),
            description: "Nested skill".to_string(),
            input_schema: Some(serde_json::json!({"type": "object", "properties": {}})),
            requires_yaml_optimization: true,
            procedures: vec![make_step(1, "Test")],
            ..Default::default()
        };
        let validated = ValidatedSkillIR::new(ir);
        let emitter = GeminiEmitter::new();
        let assets = emitter.generate_assets(&validated);
        assert!(!assets.is_empty());
        assert!(assets.iter().any(|(path, _)| path.contains("input.yaml")));
        assert!(assets.iter().any(|(path, _)| path.contains("config.yaml")));
    }

    #[test]
    fn test_gemini_target() {
        let emitter = GeminiEmitter::new();
        assert_eq!(emitter.target(), TargetPlatform::Gemini);
    }
}