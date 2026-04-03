//! Anti-Skill Injector
//!
//! Injects safety constraints based on detected patterns.

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
                    source: "anti-skill-injector".to_string(),
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
