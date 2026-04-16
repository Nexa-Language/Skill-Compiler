//! Constraint Definition
//!
//! Represents anti-skill constraints injected by the analyzer.

use std::sync::Arc;

use serde::{Deserialize, Serialize};

/// Anti-skill constraint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Constraint {
    /// Constraint source (pattern ID) — shared identifier across Emitters
    pub source: Arc<str>,
    /// Constraint content
    pub content: String,
    /// Constraint level
    #[serde(default)]
    pub level: ConstraintLevel,
    /// Constraint scope
    #[serde(default)]
    pub scope: ConstraintScope,
}

/// Constraint level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ConstraintLevel {
    /// Warning level - display but don't block
    #[default]
    Warning,
    /// Error level - block execution
    Error,
    /// Block level - require human intervention
    Block,
}

/// Constraint scope
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ConstraintScope {
    /// Apply to all steps
    #[default]
    Global,
    /// Apply to specific steps
    SpecificSteps {
        /// Step IDs to apply to
        step_ids: Vec<u32>,
    },
    /// Apply to steps matching keywords
    KeywordMatch {
        /// Keywords to match
        keywords: Vec<String>,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constraint_level_serde() {
        // ConstraintLevel uses serde rename_all = "lowercase"
        assert_eq!(serde_json::to_string(&ConstraintLevel::Block).unwrap(), "\"block\"");
        assert_eq!(serde_json::to_string(&ConstraintLevel::Error).unwrap(), "\"error\"");
        assert_eq!(serde_json::to_string(&ConstraintLevel::Warning).unwrap(), "\"warning\"");
    }

    #[test]
    fn test_constraint_level_default() {
        assert_eq!(ConstraintLevel::default(), ConstraintLevel::Warning);
    }

    #[test]
    fn test_constraint_level_equality() {
        assert!(ConstraintLevel::Block == ConstraintLevel::Block);
        assert!(ConstraintLevel::Block != ConstraintLevel::Warning);
    }

    #[test]
    fn test_constraint_scope_default() {
        assert!(matches!(ConstraintScope::default(), ConstraintScope::Global));
    }

    #[test]
    fn test_constraint_scope_serde() {
        let global = serde_json::to_string(&ConstraintScope::Global).unwrap();
        assert!(global.contains("global"));
    }
}
