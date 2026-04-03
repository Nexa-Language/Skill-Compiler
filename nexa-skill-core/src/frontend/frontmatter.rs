//! Frontmatter Parser
//!
//! Parse YAML frontmatter from SKILL.md files.

use crate::error::ParseError;
use serde::Deserialize;
use std::collections::HashMap;

/// Frontmatter metadata structure
///
/// This represents the YAML frontmatter section of a SKILL.md file.
#[derive(Debug, Clone, Deserialize)]
pub struct FrontmatterMeta {
    // === Required fields ===
    /// Skill name (kebab-case, 1-64 characters)
    pub name: String,

    /// Skill description with trigger conditions
    pub description: String,

    // === Optional fields ===
    /// Version number (semantic versioning)
    #[serde(default)]
    pub version: Option<String>,

    /// License identifier
    #[serde(default)]
    pub license: Option<String>,

    /// Environment compatibility notes
    #[serde(default)]
    pub compatibility: Option<String>,

    /// Extended metadata
    #[serde(default)]
    pub metadata: Option<HashMap<String, serde_json::Value>>,

    /// Pre-approved tools (experimental)
    #[serde(default)]
    pub allowed_tools: Option<String>,

    // === NSC extension fields ===
    /// MCP server dependencies
    #[serde(default)]
    pub mcp_servers: Option<Vec<String>>,

    /// Input parameter JSON Schema
    #[serde(default)]
    pub input_schema: Option<serde_json::Value>,

    /// Output parameter JSON Schema
    #[serde(default)]
    pub output_schema: Option<serde_json::Value>,

    /// Human-in-the-loop requirement flag
    #[serde(default)]
    pub hitl_required: Option<bool>,

    /// Pre-execution conditions
    #[serde(default)]
    pub pre_conditions: Option<Vec<String>>,

    /// Post-execution conditions
    #[serde(default)]
    pub post_conditions: Option<Vec<String>>,

    /// Error recovery strategies
    #[serde(default)]
    pub fallbacks: Option<Vec<String>>,

    /// Permission declarations
    #[serde(default)]
    pub permissions: Option<Vec<PermissionDecl>>,

    /// Security level
    #[serde(default)]
    pub security_level: Option<String>,
}

/// Permission declaration from frontmatter
#[derive(Debug, Clone, Deserialize)]
pub struct PermissionDecl {
    /// Permission type
    pub kind: String,
    /// Permission scope
    pub scope: String,
    /// Optional description
    #[serde(default)]
    pub description: Option<String>,
}

/// Extract frontmatter from SKILL.md content
///
/// # Errors
///
/// Returns an error if:
/// - No frontmatter is found
/// - The frontmatter is empty
/// - The YAML is malformed
pub fn extract_frontmatter(content: &str) -> Result<(FrontmatterMeta, &str), ParseError> {
    // Check for frontmatter boundaries
    if !content.starts_with("---") {
        return Err(ParseError::MissingFrontmatter);
    }

    // Find the closing ---
    let end_idx = content[3..]
        .find("\n---")
        .ok_or(ParseError::MissingFrontmatter)?;

    let yaml_content = &content[3..end_idx + 3];
    let body_start = end_idx + 7; // Skip "---\n---\n"

    // Parse YAML
    let meta: FrontmatterMeta =
        serde_yaml::from_str(yaml_content).map_err(|e| ParseError::YamlError(e.to_string()))?;

    let body = &content[body_start.min(content.len())..];

    Ok((meta, body))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_frontmatter_valid() {
        let content = r#"---
name: test-skill
description: A test skill
version: "1.0.0"
---

# Test Skill

This is the body.
"#;

        let result = extract_frontmatter(content);
        assert!(result.is_ok());

        let (meta, body) = result.unwrap();
        assert_eq!(meta.name, "test-skill");
        assert_eq!(meta.description, "A test skill");
        assert_eq!(meta.version, Some("1.0.0".to_string()));
        assert!(body.contains("# Test Skill"));
    }

    #[test]
    fn test_extract_frontmatter_missing() {
        let content = "# Test Skill\nNo frontmatter here.";
        let result = extract_frontmatter(content);
        assert!(result.is_err());
    }
}
