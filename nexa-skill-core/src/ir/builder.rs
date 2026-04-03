//! IR Builder
//!
//! Transform RawAST into SkillIR.

use chrono::Utc;
use crate::frontend::RawAST;
use crate::ir::{Constraint, Example, Permission, PermissionKind, ProcedureStep, SecurityLevel, SkillIR};

/// Build SkillIR from RawAST
pub fn build_ir(raw: &RawAST) -> SkillIR {
    let fm = &raw.frontmatter;
    let body = &raw.body;

    // Build procedures from markdown body
    let procedures: Vec<ProcedureStep> = body.procedures
        .iter()
        .map(|p| ProcedureStep {
            order: p.order,
            instruction: p.text.clone(),
            is_critical: p.is_critical,
            constraints: vec![],
            expected_output: None,
            on_error: None,
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
                    read_only: false,
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
                    source: "user_defined".to_string(),
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

    SkillIR {
        name: fm.name.clone(),
        version: fm.version.clone().unwrap_or_else(|| "0.1.0".to_string()),
        description: fm.description.clone(),
        mcp_servers: fm.mcp_servers.clone().unwrap_or_default(),
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
        few_shot_examples,
        anti_skill_constraints: constraints,
        source_path: raw.source_path.clone(),
        source_hash: raw.source_hash.clone(),
        compiled_at: Some(Utc::now()),
    }
}