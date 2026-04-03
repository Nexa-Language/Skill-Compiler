//! Constraint Definition
//!
//! Represents anti-skill constraints injected by the analyzer.

use serde::{Deserialize, Serialize};

/// Anti-skill constraint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Constraint {
    /// Constraint source (pattern ID)
    pub source: String,
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
