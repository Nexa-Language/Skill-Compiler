//! SkillIR - Core Intermediate Representation
//!
//! This is the central data structure that represents a skill
//! throughout the compilation pipeline.

use std::sync::Arc;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::{Constraint, Example, Permission, ProcedureStep, SectionInfo};

/// An alternative execution approach for mode-selector skills
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Approach {
    /// Approach name (e.g. "Basic Extraction", "Advanced Analysis")
    pub name: Arc<str>,
    /// One-line description of this approach
    pub description: Arc<str>,
    /// Full instructions for this approach
    pub instructions: Arc<str>,
}

/// How the skill should be executed
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SkillMode {
    /// Single sequential procedure (Explicit Steps / Workflow)
    Sequential,
    /// Multiple alternative approaches (Mode Selector)
    Alternative,
    /// Reference toolkit — pick relevant operations as needed
    Toolkit,
    /// Pure guidelines — no structured execution logic
    Guideline,
}

impl Default for SkillMode {
    fn default() -> Self {
        Self::Sequential
    }
}

impl std::fmt::Display for SkillMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SkillMode::Sequential => write!(f, "sequential"),
            SkillMode::Alternative => write!(f, "alternative"),
            SkillMode::Toolkit => write!(f, "toolkit"),
            SkillMode::Guideline => write!(f, "guideline"),
        }
    }
}

/// Nexa Skill Compiler Core Intermediate Representation
///
/// This is the central data structure that all compilation stages
/// operate on. It represents a fully parsed and validated skill.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillIR {
    // ===== Metadata & Routing =====
    /// Skill unique identifier (kebab-case, 1-64 characters)
    /// Shared across all Emitters — zero-copy via `Arc<str>`
    pub name: Arc<str>,

    /// Version number (semantic versioning)
    /// Shared across all Emitters — zero-copy via `Arc<str>`
    pub version: Arc<str>,

    /// Description with trigger conditions
    pub description: String,

    // ===== Interfaces & MCP =====
    /// MCP server dependencies
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub mcp_servers: Vec<Arc<str>>,

    /// Input parameter JSON Schema
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_schema: Option<serde_json::Value>,

    /// Output parameter JSON Schema
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_schema: Option<serde_json::Value>,

    // ===== Security & Control =====
    /// Human-in-the-loop requirement flag
    #[serde(default)]
    pub hitl_required: bool,

    /// Pre-execution conditions
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub pre_conditions: Vec<String>,

    /// Post-execution conditions
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub post_conditions: Vec<String>,

    /// Error recovery strategies
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub fallbacks: Vec<String>,

    /// Permission declarations
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub permissions: Vec<Permission>,

    /// Security level
    #[serde(default = "default_security_level")]
    pub security_level: SecurityLevel,

    // ===== Execution Logic =====
    /// Context gathering steps
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub context_gathering: Vec<String>,

    /// Standard operating procedure steps
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub procedures: Vec<ProcedureStep>,

    /// Alternative execution approaches (for mode-selector skills)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub approaches: Vec<Approach>,

    /// Skill execution mode (Sequential, Alternative, Toolkit, Guideline)
    #[serde(default)]
    pub mode: SkillMode,

    /// Few-shot examples
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub few_shot_examples: Vec<Example>,

    // ===== Compile-time Injection =====
    /// Anti-skill constraints (injected by Analyzer)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub anti_skill_constraints: Vec<Constraint>,

    // ===== Additional Sections =====
    /// Sections from the Markdown body not captured by specific fields
    /// (e.g. "Setup", "Prerequisites", "Known Pitfalls", "Quick Reference").
    /// These carry context essential for LLM instruction adherence.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub extra_sections: Vec<SectionInfo>,

    // ===== AST Optimization Flags =====
    /// Whether YAML optimization is required for nested data
    ///
    /// When nested_data_depth >= 3, Gemini Emitter should use YAML format.
    /// Academic basis: YAML nested data accuracy 51.9% > Markdown 48.2% > JSON 43.1%
    #[serde(default, skip_serializing_if = "is_false")]
    pub requires_yaml_optimization: bool,

    /// Nested data depth
    ///
    /// Computed by NestedDataDetector during the Analyzer phase.
    /// Used for backend format selection decisions.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nested_data_depth: Option<usize>,

    // ===== Meta Information =====
    /// Source file path
    #[serde(skip_serializing)]
    pub source_path: String,

    /// Source file hash (SHA-256)
    #[serde(skip_serializing)]
    pub source_hash: String,

    /// Compilation timestamp
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compiled_at: Option<DateTime<Utc>>,
}

/// Security level for skills
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum SecurityLevel {
    /// Low security - basic format validation only
    Low,

    /// Medium security - permission declaration check (default)
    #[default]
    Medium,

    /// High security - mandatory HITL, dangerous keyword scan
    High,

    /// Critical security - no auto-execution, requires human approval
    Critical,
}

impl std::fmt::Display for SecurityLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Low => write!(f, "low"),
            Self::Medium => write!(f, "medium"),
            Self::High => write!(f, "high"),
            Self::Critical => write!(f, "critical"),
        }
    }
}

impl SecurityLevel {
    /// Get audit intensity description
    #[must_use]
    pub const fn audit_intensity(&self) -> &'static str {
        match self {
            Self::Low => "Basic format validation",
            Self::Medium => "Permission declaration check",
            Self::High => "Mandatory HITL, dangerous keyword scan",
            Self::Critical => "No auto-execution, requires human approval",
        }
    }

    /// Check if HITL is required
    #[must_use]
    pub const fn requires_hitl(&self) -> bool {
        matches!(self, Self::High | Self::Critical)
    }

    /// Check if auto-execution is blocked
    #[must_use]
    pub const fn blocks_auto_execution(&self) -> bool {
        matches!(self, Self::Critical)
    }
}

fn default_security_level() -> SecurityLevel {
    SecurityLevel::default()
}

/// Helper for serde skip_serializing_if on bool fields
fn is_false(v: &bool) -> bool {
    !v
}

impl SkillIR {
    /// Validate the IR
    ///
    /// # Errors
    ///
    /// Returns an error if validation fails.
    pub fn validate(&self) -> Result<(), IRError> {
        // Validate name
        if self.name.is_empty() {
            return Err(IRError::MissingRequiredField("name"));
        }
        if !Self::is_valid_name(&self.name) {
            return Err(IRError::InvalidNameFormat(self.name.to_string()));
        }

        // Validate description
        if self.description.is_empty() {
            return Err(IRError::MissingRequiredField("description"));
        }
        if self.description.len() > 1024 {
            return Err(IRError::DescriptionTooLong(self.description.len()));
        }

        // Validate HITL requirement consistency
        if self.hitl_required && self.security_level == SecurityLevel::Low {
            return Err(IRError::InconsistentSecurityLevel);
        }

        Ok(())
    }

    /// Check if name is valid kebab-case
    pub fn is_valid_name(name: &str) -> bool {
        !name.is_empty()
            && name.len() <= 64
            && name
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
            && !name.starts_with('-')
            && !name.ends_with('-')
            && !name.contains("--")
    }
}

/// IR construction error
#[derive(Debug, Clone, thiserror::Error, miette::Diagnostic)]
pub enum IRError {
    #[error("Missing required field: {0}")]
    #[diagnostic(
        code(nsc::ir::missing_field),
        help("Add the '{0}' field to your frontmatter")
    )]
    MissingRequiredField(&'static str),

    #[error("Invalid name format: '{0}'")]
    #[diagnostic(
        code(nsc::ir::invalid_name),
        help("Name must be kebab-case: lowercase letters, numbers, hyphens. 1-64 chars.")
    )]
    InvalidNameFormat(String),

    #[error("Description too long: {0} characters (max 1024)")]
    #[diagnostic(code(nsc::ir::description_too_long))]
    DescriptionTooLong(usize),

    #[error("Inconsistent security level: HITL required but security level is Low")]
    #[diagnostic(code(nsc::ir::inconsistent_security))]
    InconsistentSecurityLevel,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_security_level_requires_hitl() {
        assert!(!SecurityLevel::Low.requires_hitl());
        assert!(!SecurityLevel::Medium.requires_hitl());
        assert!(SecurityLevel::High.requires_hitl());
        assert!(SecurityLevel::Critical.requires_hitl());
    }

    #[test]
    fn test_security_level_blocks_auto_execution() {
        assert!(!SecurityLevel::Low.blocks_auto_execution());
        assert!(!SecurityLevel::Medium.blocks_auto_execution());
        assert!(!SecurityLevel::High.blocks_auto_execution());
        assert!(SecurityLevel::Critical.blocks_auto_execution());
    }

    #[test]
    fn test_validate_name() {
        let valid_names = vec!["test-skill", "database-migration", "skill123", "a"];
        for name in valid_names {
            let ir = SkillIR {
                name: Arc::from(name),
                version: Arc::from("1.0.0"),
                description: "Test".to_string(),
                ..Default::default()
            };
            assert!(ir.validate().is_ok(), "Name '{}' should be valid", name);
        }
    }
}

impl Default for SkillIR {
    fn default() -> Self {
        Self {
            name: Arc::from(""),
            version: Arc::from("1.0.0"),
            description: String::new(),
            mcp_servers: Vec::new(),
            input_schema: None,
            output_schema: None,
            hitl_required: false,
            pre_conditions: Vec::new(),
            post_conditions: Vec::new(),
            fallbacks: Vec::new(),
            permissions: Vec::new(),
            security_level: SecurityLevel::default(),
            context_gathering: Vec::new(),
            procedures: Vec::new(),
            approaches: Vec::new(),
            mode: SkillMode::default(),
            few_shot_examples: Vec::new(),
            anti_skill_constraints: Vec::new(),
            extra_sections: Vec::new(),
            requires_yaml_optimization: false,
            nested_data_depth: None,
            source_path: String::new(),
            source_hash: String::new(),
            compiled_at: None,
        }
    }
}
