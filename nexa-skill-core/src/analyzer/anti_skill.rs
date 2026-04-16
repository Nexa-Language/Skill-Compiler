//! Anti-Skill Injector
//!
//! Injects safety constraints based on detected patterns.

use std::sync::Arc;

use crate::ir::{Constraint, ConstraintLevel, ConstraintScope, SkillIR};

/// Anti-skill injector
pub struct AntiSkillInjector {
    /// Anti-pattern rules
    patterns: Vec<AntiPattern>,
}

/// Anti-pattern rule
struct AntiPattern {
    trigger_keywords: Vec<&'static str>,
    constraint_content: &'static str,
}

impl AntiSkillInjector {
    /// Create a new anti-skill injector
    #[must_use]
    pub fn new() -> Self {
        Self {
            patterns: Self::load_default_patterns(),
        }
    }

    /// Load default anti-patterns
    fn load_default_patterns() -> Vec<AntiPattern> {
        vec![
            AntiPattern {
                trigger_keywords: vec!["HTTP", "GET", "POST", "fetch", "request"],
                constraint_content: "Never execute an HTTP request without a timeout parameter (default 10s). Do not retry more than 3 times on 403 Forbidden errors.",
            },
            AntiPattern {
                trigger_keywords: vec!["BeautifulSoup", "HTML parse", "scrape"],
                constraint_content: "Do not attempt to parse raw JavaScript variables using HTML parsers. Fallback to Regex if <script> tags are encountered.",
            },
            AntiPattern {
                trigger_keywords: vec!["DROP", "DELETE", "TRUNCATE"],
                constraint_content: "Never execute destructive database operations without explicit user confirmation. Always show affected rows before execution.",
            },
            AntiPattern {
                trigger_keywords: vec!["while", "loop", "repeat"],
                constraint_content: "All loops must have a maximum iteration limit (default 1000). Implement a counter and break condition to prevent infinite loops.",
            },
        ]
    }

    /// Inject anti-skill constraints into the IR
    #[must_use]
    pub fn inject(&self, mut ir: SkillIR) -> SkillIR {
        // Collect all procedure text
        let all_text: String = ir
            .procedures
            .iter()
            .map(|p| p.instruction.as_str())
            .collect::<Vec<_>>()
            .join(" ");

        // Check each pattern
        for pattern in &self.patterns {
            let matches = pattern
                .trigger_keywords
                .iter()
                .any(|keyword| all_text.contains(keyword));

            if matches {
                ir.anti_skill_constraints.push(Constraint {
                    source: Arc::from("anti-skill-injector"),
                    content: pattern.constraint_content.to_string(),
                    level: ConstraintLevel::Warning,
                    scope: ConstraintScope::Global,
                });
            }
        }

        ir
    }
}

impl Default for AntiSkillInjector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::{ConstraintLevel, ProcedureStep, SkillIR};
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

    fn make_test_ir(name: &str, procedures: Vec<ProcedureStep>) -> SkillIR {
        SkillIR {
            name: Arc::from(name),
            version: Arc::from("1.0.0"),
            description: "A test skill".to_string(),
            procedures,
            ..Default::default()
        }
    }

    #[test]
    fn test_inject_drop_constraint() {
        // Procedure containing "DROP" triggers db-destructive pattern
        let ir = make_test_ir("db-skill", vec![make_step(1, "Execute database changes with DROP")]);
        let injector = AntiSkillInjector::new();
        let result = injector.inject(ir);
        assert!(!result.anti_skill_constraints.is_empty());
        let db_constraint = result.anti_skill_constraints.iter().find(|c| {
            c.source == Arc::from("anti-skill-injector")
                && c.content.contains("destructive database")
        });
        assert!(db_constraint.is_some());
        assert_eq!(db_constraint.unwrap().level, ConstraintLevel::Warning);
    }

    #[test]
    fn test_inject_http_constraint() {
        let ir = make_test_ir("http-skill", vec![make_step(1, "Make HTTP request to API")]);
        let injector = AntiSkillInjector::new();
        let result = injector.inject(ir);
        let http_constraint = result.anti_skill_constraints.iter().find(|c| c.content.contains("timeout"));
        assert!(http_constraint.is_some());
    }

    #[test]
    fn test_no_injection_for_safe_skill() {
        let ir = make_test_ir("safe-skill", vec![make_step(1, "Read and display data safely")]);
        let injector = AntiSkillInjector::new();
        let result = injector.inject(ir);
        assert!(result.anti_skill_constraints.is_empty());
    }

    #[test]
    fn test_preserves_existing_fields() {
        let ir = SkillIR {
            name: Arc::from("test"),
            version: Arc::from("1.0.0"),
            description: "Test skill".to_string(),
            mcp_servers: vec![Arc::from("postgres-server")],
            hitl_required: true,
            procedures: vec![make_step(1, "DROP table")],
            ..Default::default()
        };
        let injector = AntiSkillInjector::new();
        let result = injector.inject(ir);
        assert_eq!(result.name, Arc::from("test"));
        assert_eq!(result.mcp_servers, vec![Arc::from("postgres-server")]);
        assert!(result.hitl_required);
        assert!(!result.anti_skill_constraints.is_empty());
    }

    #[test]
    fn test_multiple_dangerous_keywords() {
        // "loop" matches loop pattern, "DROP"/"TRUNCATE" match the same db-destructive pattern
        let ir = make_test_ir(
            "dangerous-skill",
            vec![make_step(1, "Use loop and DROP and TRUNCATE")],
        );
        let injector = AntiSkillInjector::new();
        let result = injector.inject(ir);
        // At least 2 constraints: loop + db-destructive
        assert!(result.anti_skill_constraints.len() >= 2);
    }

    #[test]
    fn test_default_patterns_loaded() {
        let injector = AntiSkillInjector::new();
        // Verify by injecting an IR that triggers all 4 default patterns
        let ir = make_test_ir(
            "all-patterns",
            vec![make_step(1, "HTTP BeautifulSoup DROP while")],
        );
        let result = injector.inject(ir);
        assert_eq!(result.anti_skill_constraints.len(), 4);
    }
}
