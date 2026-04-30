//! IR Builder
//!
//! Transform RawAST into SkillIR.

use std::sync::Arc;

use chrono::Utc;
use crate::frontend::RawAST;
use crate::ir::{Approach, Constraint, Example, Permission, PermissionKind, ProcedureStep, SectionInfo, SecurityLevel, SkillIR, SkillMode};
use crate::security::SecurityBaseline;

/// Build SkillIR from RawAST
pub fn build_ir(raw: &RawAST) -> SkillIR {
    let fm = &raw.frontmatter;
    let body = &raw.body;

    // Build procedures from markdown body
    let procedures: Vec<ProcedureStep> = body.procedures
        .iter()
        .map(|p| {
            // Merge step title and body content into instruction.
            // Heading-based steps carry their body (code blocks, parameters, etc.)
            // in the `body` field; list-based steps have empty body.
            let instruction = if p.body.is_empty() {
                p.text.clone()
            } else {
                format!("{}\n\n{}", p.text, p.body)
            };
            ProcedureStep {
                order: p.order,
                instruction,
                is_critical: p.is_critical,
                constraints: vec![],
                expected_output: None,
                on_error: None,
            }
        })
        .collect();

    // Build examples from markdown body
    let few_shot_examples: Vec<Example> = body.examples
        .iter()
        .enumerate()
        .map(|(i, e)| Example {
            title: e.title.clone().or_else(|| Some(format!("Example {}", i + 1))),
            user_input: e.user_input.clone(),
            agent_response: e.agent_response.clone(),
            tags: vec![],
            difficulty: None,
        })
        .collect();

    // Build permissions from frontmatter
    let permissions: Vec<Permission> = fm.permissions
        .as_ref()
        .map(|perms| perms
            .iter()
            .filter_map(|p| {
                let kind = match p.kind.to_lowercase().as_str() {
                    "network" => PermissionKind::Network,
                    "filesystem" | "fs" => PermissionKind::FileSystem,
                    "database" | "db" => PermissionKind::Database,
                    "execute" | "exec" => PermissionKind::Execute,
                    "mcp" => PermissionKind::MCP,
                    "environment" | "env" => PermissionKind::Environment,
                    _ => return None,
                };
                Some(Permission {
                    kind,
                    scope: p.scope.clone(),
                    description: p.description.clone(),
                    read_only: SecurityBaseline::derive_read_only(kind, &p.scope),
                })
            })
            .collect())
        .unwrap_or_default();

    // Build constraints from sections (looking for "Constraints" section)
    let constraints: Vec<Constraint> = body.sections
        .iter()
        .filter(|s| s.title.to_lowercase().contains("constraint"))
        .flat_map(|s| {
            s.content.lines().filter_map(|line| {
                let line = line.trim();
                if line.is_empty() {
                    return None;
                }
                Some(Constraint {
                    source: Arc::from("user_defined"),
                    content: line.to_string(),
                    level: crate::ir::constraint::ConstraintLevel::Warning,
                    scope: crate::ir::constraint::ConstraintScope::Global,
                })
            })
        })
        .collect();

    // Build context gathering from sections
    let context_gathering: Vec<String> = body.sections
        .iter()
        .filter(|s| s.title.to_lowercase().contains("context"))
        .flat_map(|s| {
            s.content.lines().filter_map(|line| {
                let line = line.trim();
                if line.is_empty() { None } else { Some(line.to_string()) }
            })
        })
        .collect();

    // Build extra_sections: sections not already captured by specific fields.
    // Skip sections whose content is already represented in procedures, constraints,
    // context_gathering, etc. to avoid duplication in the compiled output.
    let consumed_section_patterns = [
        "procedure", "constraint", "context", "example", "示例",
    ];
    let is_heading_step = |title: &str| {
        title.trim().starts_with("Step ")
            || title.trim().starts_with("Phase ")
            || (title.trim().get(0..1).is_some_and(|c| c.chars().next().is_some_and(|d| d.is_ascii_digit()))
                && (title.trim().contains('.') || title.trim().starts_with(|c: char| c.is_ascii_digit())))
    };

    let extra_sections: Vec<SectionInfo> = body.sections
        .iter()
        .filter(|s| {
            let t = s.title.to_lowercase();
            // Skip if already consumed by a specific field
            !consumed_section_patterns.iter().any(|pat| t.contains(pat))
            // Skip heading-based procedure step sections (their content is in step bodies)
            && !is_heading_step(&s.title)
            // Skip empty sections
            && !s.content.trim().is_empty()
        })
        .map(|s| SectionInfo {
            level: s.level,
            title: s.title.clone(),
            content: s.content.clone(),
        })
        .collect();

    // Parse security level
    let security_level = fm.security_level
        .as_ref()
        .and_then(|s| match s.to_lowercase().as_str() {
            "low" => Some(SecurityLevel::Low),
            "medium" => Some(SecurityLevel::Medium),
            "high" => Some(SecurityLevel::High),
            "critical" => Some(SecurityLevel::Critical),
            _ => None,
        })
        .unwrap_or(SecurityLevel::Medium);

    // Infer SkillMode before moving procedures into SkillIR
    let mut mode = SkillMode::default();
    infer_skill_mode(&procedures, &body.approaches, &mut mode);

    SkillIR {
        name: Arc::from(fm.name.clone()),
        version: Arc::from(fm.version.clone().unwrap_or_else(|| "0.1.0".to_string())),
        description: fm.description.clone(),
        mcp_servers: fm.mcp_servers.clone().unwrap_or_default().into_iter().map(Arc::from).collect(),
        input_schema: fm.input_schema.clone(),
        output_schema: fm.output_schema.clone(),
        hitl_required: fm.hitl_required.unwrap_or(false),
        pre_conditions: fm.pre_conditions.clone().unwrap_or_default(),
        post_conditions: fm.post_conditions.clone().unwrap_or_default(),
        fallbacks: fm.fallbacks.clone().unwrap_or_default(),
        permissions,
        security_level,
        context_gathering,
        procedures,
        approaches: body.approaches.iter().map(|a| Approach {
            name: Arc::from(a.name.clone()),
            description: Arc::from(a.description.clone()),
            instructions: Arc::from(a.instructions.clone()),
        }).collect(),
        mode,
        few_shot_examples,
        anti_skill_constraints: constraints,
        extra_sections,
        requires_yaml_optimization: false,
        nested_data_depth: None,
        source_path: raw.source_path.clone(),
        source_hash: raw.source_hash.clone(),
        compiled_at: Some(Utc::now()),
    }
}

/// Infer the SkillMode from the extracted procedures and approaches.
///
/// Classification logic:
/// - Alternative: if approaches are present (mode-selector skills)
/// - Guideline: if the only procedure is the fallback "Follow the skill instructions"
/// - Toolkit: if procedures contain operation-like keywords
/// - Sequential: if procedures exist without the above patterns
fn infer_skill_mode(
    procedures: &[ProcedureStep],
    approaches: &[crate::frontend::RawApproach],
    mode: &mut SkillMode,
) {
    if !approaches.is_empty() {
        *mode = SkillMode::Alternative;
    } else if procedures.len() == 1
        && procedures[0].instruction.starts_with("Follow the skill instructions")
    {
        *mode = SkillMode::Guideline;
    } else if !procedures.is_empty()
        && procedures.iter().any(|p| {
            let t = p.instruction.to_lowercase();
            t.contains("operation")
                || t.contains("extract")
                || t.contains("merge")
                || t.contains("creating")
                || t.contains("editing")
                || t.contains("reading")
                || t.contains("analyzing")
                || t.contains("recalculat")
                || t.contains("library")
                || t.contains("common workflow")
        })
    {
        *mode = SkillMode::Toolkit;
    } else if !procedures.is_empty() {
        *mode = SkillMode::Sequential;
    } else {
        *mode = SkillMode::Guideline;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frontend::ASTBuilder;
    use crate::ir::{PermissionKind, SecurityLevel};

    #[test]
    fn test_build_ir_from_basic_skill() {
        let content = r#"---
name: test-skill
description: A test skill
version: "1.0.0"
---
# Test Skill
This is the body."#;
        let raw = ASTBuilder::build_from_content("test.md", content).unwrap();
        let ir = build_ir(&raw);
        assert_eq!(ir.name.as_ref(), "test-skill");
        assert_eq!(ir.description, "A test skill");
        assert_eq!(ir.version.as_ref(), "1.0.0");
    }

    #[test]
    fn test_build_ir_with_permissions() {
        let content = r#"---
name: db-skill
description: Database skill
permissions:
  - kind: database
    scope: "postgres:staging:SELECT"
---
# DB Skill"#;
        let raw = ASTBuilder::build_from_content("db.md", content).unwrap();
        let ir = build_ir(&raw);
        assert!(!ir.permissions.is_empty());
        assert_eq!(ir.permissions[0].kind, PermissionKind::Database);
        assert_eq!(ir.permissions[0].scope, "postgres:staging:SELECT");
        // SecurityBaseline::derive_read_only(Database, "postgres:staging:SELECT") → true (SELECT op)
        assert!(ir.permissions[0].read_only);
    }

    #[test]
    fn test_build_ir_default_values() {
        let content = r#"---
name: minimal-skill
description: Minimal skill
---
# Minimal"#;
        let raw = ASTBuilder::build_from_content("minimal.md", content).unwrap();
        let ir = build_ir(&raw);
        assert_eq!(ir.security_level, SecurityLevel::Medium);
        assert!(!ir.hitl_required);
        assert!(ir.mcp_servers.is_empty());
        assert!(!ir.requires_yaml_optimization);
        assert!(ir.nested_data_depth.is_none());
    }

    #[test]
    fn test_build_ir_security_level_mapping() {
        let content = r#"---
name: secure-skill
description: Secure skill
security_level: high
---
# Secure"#;
        let raw = ASTBuilder::build_from_content("secure.md", content).unwrap();
        let ir = build_ir(&raw);
        assert_eq!(ir.security_level, SecurityLevel::High);
    }
}