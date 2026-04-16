//! Claude Emitter
//!
//! Emits Claude-compatible XML format using Askama templates.

use askama::Template;
use nexa_skill_templates::{ClaudeContext, ConstraintContext, ExampleContext, PermissionContext, SectionContext, StepContext};

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

impl Emitter for ClaudeEmitter {
    fn target(&self) -> TargetPlatform {
        TargetPlatform::Claude
    }

    fn emit(&self, ir: &ValidatedSkillIR) -> Result<String, EmitError> {
        let inner = ir.as_ref();
        let context = ClaudeContext {
            name: inner.name.to_string(),
            version: inner.version.to_string(),
            description: inner.description.clone(),
            hitl_required: inner.hitl_required,
            procedures: inner
                .procedures
                .iter()
                .map(|s| StepContext {
                    order: s.order,
                    instruction: s.instruction.clone(),
                    is_critical: s.is_critical,
                })
                .collect(),
            anti_skill_constraints: inner
                .anti_skill_constraints
                .iter()
                .map(|c| ConstraintContext {
                    source: c.source.to_string(),
                    content: c.content.clone(),
                    level_marker: String::new(), // Claude doesn't use level markers
                })
                .collect(),
            pre_conditions: inner.pre_conditions.clone(),
            post_conditions: inner.post_conditions.clone(),
            fallbacks: inner.fallbacks.clone(),
            context_gathering: inner.context_gathering.clone(),
            examples: inner
                .few_shot_examples
                .iter()
                .map(|e| ExampleContext {
                    title: e.title.clone().unwrap_or_default(),
                    user_input: e.user_input.clone(),
                    agent_response: e.agent_response.clone(),
                })
                .collect(),
            permissions: inner
                .permissions
                .iter()
                .map(|p| PermissionContext {
                    kind_name: p.kind.display_name().to_string(),
                    scope: p.scope.clone(),
                    read_only: p.read_only,
                    description: p.description.clone().unwrap_or_default(),
                })
                .collect(),
            mcp_servers: inner.mcp_servers.iter().map(|s| s.to_string()).collect(),
            security_level: inner.security_level.to_string().to_lowercase(),
            extra_sections: inner
                .extra_sections
                .iter()
                .map(|s| SectionContext {
                    level: s.level,
                    title: s.title.clone(),
                    content: s.content.clone(),
                })
                .collect(),
        };
        context
            .render()
            .map_err(|e| EmitError::TemplateError(format!("Template render failed: {}", e)))
    }
}

impl Default for ClaudeEmitter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analyzer::ValidatedSkillIR;
    use crate::backend::Emitter;
    use crate::ir::{ProcedureStep, SkillIR};
    use std::sync::Arc;

    fn make_step(order: u32, instruction: &str, is_critical: bool) -> ProcedureStep {
        ProcedureStep {
            order,
            instruction: instruction.to_string(),
            is_critical,
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
            hitl_required: true,
            procedures: vec![
                make_step(1, "Step 1", false),
                make_step(2, "Step 2", true),
            ],
            ..Default::default()
        }
    }

    #[test]
    fn test_claude_emitter_xml_output() {
        let ir = make_test_ir();
        let validated = ValidatedSkillIR::new(ir);
        let emitter = ClaudeEmitter::new();
        let result = emitter.emit(&validated).unwrap();
        assert!(result.contains("<agent_skill>"));
        assert!(result.contains("<name>test-skill</name>"));
        assert!(result.contains("<intent>Test description</intent>"));
        assert!(result.contains("<system_constraint>"));
        assert!(result.contains("<step order=\"1\">"));
        assert!(result.contains("critical=\"true\""));
        assert!(result.contains("</agent_skill>"));
    }

    #[test]
    fn test_claude_target() {
        let emitter = ClaudeEmitter::new();
        assert_eq!(emitter.target(), TargetPlatform::Claude);
    }
}