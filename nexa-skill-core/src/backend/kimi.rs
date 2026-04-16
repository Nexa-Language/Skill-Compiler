//! Kimi Emitter - Full Markdown with Maximum Context Preservation
//!
//! Kimi (K2.5) excels at ultra-long context processing (128K+ tokens).
//! This emitter generates comprehensive Markdown that preserves ALL details,
//! relying on Kimi's strong reasoning capability to extract relevant information.
//!
//! Key difference from GeminiEmitter:
//! - No YAML optimization (Kimi handles JSON Schema inline well)
//! - No format simplification (preserve all sections)
//! - No emoji markers (plain text, blockquotes for constraints)
//! - No dual-payload assets (everything inline)
//!
//! Strategy: "Knowledge-intensive, weak constraints, strong reasoning"

use askama::Template;
use nexa_skill_templates::{
    ConstraintContext, ExampleContext, KimiContext, PermissionContext, StepContext,
};

use crate::analyzer::ValidatedSkillIR;
use crate::error::EmitError;
use crate::ir::SkillIR;

use super::{Emitter, TargetPlatform};

/// Kimi emitter — full Markdown with maximum context preservation
pub struct KimiEmitter;

impl KimiEmitter {
    /// Create a new Kimi emitter
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl Emitter for KimiEmitter {
    fn target(&self) -> TargetPlatform {
        TargetPlatform::Kimi
    }

    fn emit(&self, ir: &ValidatedSkillIR) -> Result<String, EmitError> {
        let inner = ir.as_ref();
        let context = self.build_context(inner);
        context
            .render()
            .map_err(|e| EmitError::TemplateError(format!("Template render failed: {}", e)))
    }

    fn file_extension(&self) -> &'static str {
        ".md"
    }

    fn requires_manifest(&self) -> bool {
        true
    }

    fn generate_assets(&self, _ir: &ValidatedSkillIR) -> Vec<(String, String)> {
        // Kimi does not generate separate asset files — all content is inline
        Vec::new()
    }
}

impl Default for KimiEmitter {
    fn default() -> Self {
        Self::new()
    }
}

impl KimiEmitter {
    /// Build KimiContext from SkillIR
    fn build_context(&self, ir: &SkillIR) -> KimiContext {
        KimiContext {
            name: ir.name.to_string(),
            version: ir.version.to_string(),
            security_level: ir.security_level.to_string(),
            description: ir.description.clone(),
            hitl_required: ir.hitl_required,
            mcp_servers: ir.mcp_servers.iter().map(|s| s.to_string()).collect(),
            permissions: ir
                .permissions
                .iter()
                .map(|p| PermissionContext {
                    kind_name: p.kind.display_name().to_string(),
                    scope: p.scope.clone(),
                    read_only: p.read_only,
                    description: p.description.clone().unwrap_or_default(),
                })
                .collect(),
            pre_conditions: ir.pre_conditions.clone(),
            context_gathering: ir.context_gathering.clone(),
            input_schema_json: ir
                .input_schema
                .as_ref()
                .and_then(|s| serde_json::to_string_pretty(s).ok())
                .unwrap_or_default(),
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
                    level_marker: String::new(), // Kimi uses blockquotes, no level markers
                })
                .collect(),
            fallbacks: ir.fallbacks.clone(),
            post_conditions: ir.post_conditions.clone(),
            output_schema_json: ir
                .output_schema
                .as_ref()
                .and_then(|s| serde_json::to_string_pretty(s).ok())
                .unwrap_or_default(),
            examples: ir
                .few_shot_examples
                .iter()
                .map(|e| ExampleContext {
                    title: e.title.clone().unwrap_or_default(),
                    user_input: e.user_input.clone(),
                    agent_response: e.agent_response.clone(),
                })
                .collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;
    use crate::analyzer::ValidatedSkillIR;
    use crate::ir::{
        Constraint, ConstraintLevel, ConstraintScope, Example, Permission, PermissionKind,
        ProcedureStep, SecurityLevel, SkillIR,
    };

    fn make_test_ir() -> SkillIR {
        SkillIR {
            name: Arc::from("database-migration"),
            version: Arc::from("2.1.0"),
            description: "执行 PostgreSQL 数据库表结构修改".to_string(),
            hitl_required: true,
            security_level: SecurityLevel::High,
            mcp_servers: vec![
                Arc::from("neon-postgres-admin"),
                Arc::from("github-pr-creator"),
            ],
            permissions: vec![Permission {
                kind: PermissionKind::Database,
                scope: "postgres:staging:ALTER".to_string(),
                description: Some("Allow ALTER on staging DB".to_string()),
                read_only: false,
            }],
            pre_conditions: vec!["确认当前数据库连接可用".to_string()],
            context_gathering: vec![
                "提取目标表的当前 Schema".to_string(),
                "检查是否有外键约束".to_string(),
            ],
            input_schema: Some(serde_json::json!({
                "type": "object",
                "properties": {
                    "target_table": { "type": "string" },
                    "migration_type": { "enum": ["add_column", "drop_column"] }
                },
                "required": ["target_table"]
            })),
            output_schema: Some(serde_json::json!({
                "type": "object",
                "properties": {
                    "success": { "type": "boolean" },
                    "migration_file": { "type": "string" }
                }
            })),
            procedures: vec![
                ProcedureStep {
                    order: 1,
                    instruction: "提取目标表的当前 Schema".to_string(),
                    is_critical: false,
                    constraints: Vec::new(),
                    expected_output: None,
                    on_error: None,
                },
                ProcedureStep {
                    order: 2,
                    instruction: "编写 SQL 迁移脚本".to_string(),
                    is_critical: true,
                    constraints: Vec::new(),
                    expected_output: None,
                    on_error: None,
                },
                ProcedureStep {
                    order: 3,
                    instruction: "执行迁移并验证结果".to_string(),
                    is_critical: false,
                    constraints: Vec::new(),
                    expected_output: None,
                    on_error: None,
                },
            ],
            anti_skill_constraints: vec![Constraint {
                source: Arc::from("db-cascade"),
                content: "Never use CASCADE without explicit user approval".to_string(),
                level: ConstraintLevel::Block,
                scope: ConstraintScope::Global,
            }],
            fallbacks: vec!["如果迁移失败，自动执行 DOWN 脚本回滚".to_string()],
            post_conditions: vec!["验证新表结构符合预期".to_string()],
            few_shot_examples: vec![Example {
                title: Some("添加列".to_string()),
                user_input: "添加 email 列到 users 表".to_string(),
                agent_response: "1. 检查表结构\n2. 编写 ALTER TABLE 语句\n3. 执行并验证"
                    .to_string(),
                tags: Vec::new(),
                difficulty: None,
            }],
            ..Default::default()
        }
    }

    #[test]
    fn test_kimi_generates_full_markdown() {
        let ir = make_test_ir();
        let validated = ValidatedSkillIR::new(ir);
        let emitter = KimiEmitter::new();
        let result = emitter.emit(&validated).unwrap();

        // Should contain title
        assert!(result.contains("# database-migration"));
        // Should contain version
        assert!(result.contains("**Version**: 2.1.0"));
        // Should contain security level
        assert!(result.contains("**Security Level**:"));
        // Should contain description
        assert!(result.contains("## Description"));
        // Should contain HITL notice
        assert!(result.contains("人工审批"));
        // Should contain MCP dependencies
        assert!(result.contains("## MCP Dependencies"));
        assert!(result.contains("neon-postgres-admin"));
        // Should contain permissions
        assert!(result.contains("## Permissions"));
        assert!(result.contains("Database: postgres:staging:ALTER"));
        // Should contain pre-conditions
        assert!(result.contains("## Pre-Conditions"));
        // Should contain context gathering
        assert!(result.contains("## Context Gathering"));
        // Should contain input schema inline
        assert!(result.contains("## Input Schema"));
        assert!(result.contains("```json"));
        // Should contain procedures
        assert!(result.contains("## Procedures"));
        assert!(result.contains("[CRITICAL]"));
        // Should contain safety constraints as blockquotes
        assert!(result.contains("## Safety Constraints"));
        assert!(result.contains("> Never use CASCADE"));
        // Should contain fallbacks
        assert!(result.contains("## Fallbacks"));
        // Should contain post-conditions
        assert!(result.contains("## Post-Conditions"));
        // Should contain output schema inline
        assert!(result.contains("## Output Schema"));
        // Should contain examples
        assert!(result.contains("## Examples"));
        assert!(result.contains("添加列"));
    }

    #[test]
    fn test_kimi_no_assets() {
        let ir = make_test_ir();
        let validated = ValidatedSkillIR::new(ir);
        let emitter = KimiEmitter::new();
        let assets = emitter.generate_assets(&validated);
        assert!(assets.is_empty());
    }

    #[test]
    fn test_kimi_target_platform() {
        let emitter = KimiEmitter::new();
        assert_eq!(emitter.target(), TargetPlatform::Kimi);
    }

    #[test]
    fn test_kimi_file_extension() {
        let emitter = KimiEmitter::new();
        assert_eq!(emitter.file_extension(), ".md");
    }

    #[test]
    fn test_kimi_minimal_ir() {
        // Test with minimal IR — only name, version, description
        let ir = SkillIR {
            name: Arc::from("simple-skill"),
            version: Arc::from("1.0.0"),
            description: "A simple skill".to_string(),
            procedures: vec![ProcedureStep {
                order: 1,
                instruction: "Do something".to_string(),
                is_critical: false,
                constraints: Vec::new(),
                expected_output: None,
                on_error: None,
            }],
            ..Default::default()
        };
        let validated = ValidatedSkillIR::new(ir);
        let emitter = KimiEmitter::new();
        let result = emitter.emit(&validated).unwrap();

        assert!(result.contains("# simple-skill"));
        assert!(result.contains("## Description"));
        assert!(result.contains("## Procedures"));
        // Sections for empty fields should NOT appear
        assert!(!result.contains("## MCP Dependencies"));
        assert!(!result.contains("## Permissions"));
        assert!(!result.contains("## Safety Constraints"));
        assert!(!result.contains("## Fallbacks"));
    }

    #[test]
    fn test_kimi_read_only_permission() {
        let ir = SkillIR {
            name: Arc::from("readonly-skill"),
            version: Arc::from("1.0.0"),
            description: "Read-only skill".to_string(),
            permissions: vec![Permission {
                kind: PermissionKind::FileSystem,
                scope: "/tmp/Read/*".to_string(),
                description: None,
                read_only: true,
            }],
            procedures: vec![ProcedureStep {
                order: 1,
                instruction: "Read file".to_string(),
                is_critical: false,
                constraints: Vec::new(),
                expected_output: None,
                on_error: None,
            }],
            ..Default::default()
        };
        let validated = ValidatedSkillIR::new(ir);
        let emitter = KimiEmitter::new();
        let result = emitter.emit(&validated).unwrap();

        assert!(result.contains("(read-only)"));
    }
}