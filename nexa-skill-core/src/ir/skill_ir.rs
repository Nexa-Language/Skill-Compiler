//! SkillIR - Core Intermediate Representation
//!
//! This is the central data structure that represents a skill
//! throughout the compilation pipeline.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::{Constraint, Example, Permission, ProcedureStep};

/// Nexa Skill Compiler Core Intermediate Representation
///
/// This is the central data structure that all compilation stages
/// operate on. It represents a fully parsed and validated skill.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillIR {
    // ===== Metadata & Routing =====
    /// Skill unique identifier (kebab-case, 1-64 characters)
    pub name: String,

    /// Version number (semantic versioning)
    pub version: String,

    /// Description with trigger conditions
    pub description: String,

    // ===== Interfaces & MCP =====
    /// MCP server dependencies
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub mcp_servers: Vec<String>,

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

    /// Few-shot examples
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub few_shot_examples: Vec<Example>,

    // ===== Compile-time Injection =====
    /// Anti-skill constraints (injected by Analyzer)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub anti_skill_constraints: Vec<Constraint>,

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
            return Err(IRError::InvalidNameFormat(self.name.clone()));
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
    fn is_valid_name(name: &str) -> bool {
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
                name: name.to_string(),
                version: "1.0.0".to_string(),
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
            name: String::new(),
            version: "1.0.0".to_string(),
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
            few_shot_examples: Vec::new(),
            anti_skill_constraints: Vec::new(),
            source_path: String::new(),
            source_hash: String::new(),
            compiled_at: None,
        }
    }
}
