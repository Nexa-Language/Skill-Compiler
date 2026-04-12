//! Routing Manifest Generator
//!
//! Implements "Progressive Routing Manifest" generation to solve Context Bloat.
//! Agent Skills standard relies on reading only YAML frontmatter for "progressive disclosure".
//!
//! When compiling a skill directory, generates a routing_manifest.yaml or skills_index.json
//! containing only name and description fields - the most efficient "memory routing table"
//! for underlying Agent systems.
//!
//! Reference: "高级提示词工程格式与智能体技能架构" research report
//! - Progressive disclosure: Agent only reads frontmatter initially
//! - Full skill loaded only when triggered (implicit or explicit invocation)
//! - This solves context bloat by separating "skill index" from "knowledge delivery"

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::ir::SkillIR;

/// Routing manifest entry - minimal metadata for progressive disclosure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingEntry {
    /// Skill name (must match directory name in kebab-case)
    pub name: String,
    /// Description (max 1024 chars, used for semantic routing)
    pub description: String,
    /// Optional: trigger keywords for implicit invocation
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub triggers: Vec<String>,
    /// Optional: file patterns this skill handles
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub file_patterns: Vec<String>,
    /// Optional: security level hint
    #[serde(skip_serializing_if = "Option::is_none")]
    pub security_level: Option<String>,
}

/// Routing manifest - the "memory routing table" for Agent systems
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingManifest {
    /// Manifest version
    pub version: String,
    /// Generated timestamp
    pub generated_at: String,
    /// Total skill count
    pub total_skills: usize,
    /// Routing entries indexed by skill name
    pub skills: HashMap<String, RoutingEntry>,
}

impl RoutingManifest {
    /// Create a new empty routing manifest
    pub fn new() -> Self {
        Self {
            version: "1.0".to_string(),
            generated_at: chrono::Utc::now().to_rfc3339(),
            total_skills: 0,
            skills: HashMap::new(),
        }
    }

    /// Add a skill to the routing manifest
    pub fn add_skill(&mut self, ir: &SkillIR) {
        let entry = RoutingEntry {
            name: ir.name.clone(),
            description: if ir.description.len() > 1024 {
                ir.description[..1024].to_string()
            } else {
                ir.description.clone()
            },
            triggers: extract_triggers(&ir.description),
            file_patterns: extract_file_patterns(&ir.description),
            security_level: Some(format!("{:?}", ir.security_level).to_lowercase()),
        };

        self.skills.insert(ir.name.clone(), entry);
        self.total_skills = self.skills.len();
    }

    /// Add multiple skills
    pub fn add_skills(&mut self, irs: &[SkillIR]) {
        for ir in irs {
            self.add_skill(ir);
        }
    }

    /// Generate YAML format (recommended for human readability)
    pub fn to_yaml(&self) -> Result<String, serde_yaml::Error> {
        serde_yaml::to_string(self)
    }

    /// Generate JSON format (alternative for programmatic access)
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Generate minimal skills_index.json (only name + description)
    pub fn to_minimal_json(&self) -> Result<String, serde_json::Error> {
        let minimal: Vec<MinimalEntry> = self.skills.values().map(|e| MinimalEntry {
            name: e.name.clone(),
            description: e.description.clone(),
        }).collect();

        serde_json::to_string_pretty(&minimal)
    }
}

impl Default for RoutingManifest {
    fn default() -> Self {
        Self::new()
    }
}

/// Minimal entry for skills_index.json (absolute minimal routing table)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinimalEntry {
    /// Skill name for routing lookup
    pub name: String,
    /// Brief description for trigger matching
    pub description: String,
}

/// Extract trigger keywords from description
/// Looks for patterns like "when X", "use this when", "triggered by"
fn extract_triggers(description: &str) -> Vec<String> {
    let mut triggers = Vec::new();
    
    // Common trigger patterns
    let trigger_patterns = [
        "when ",
        "use this when ",
        "triggered by ",
        "invoked when ",
        "activated when ",
    ];

    for pattern in trigger_patterns {
        if let Some(start) = description.find(pattern) {
            let rest = &description[start + pattern.len()..];
            // Extract up to next sentence or comma
            let trigger = rest.split(&['.', ',', '\n'][..])
                .next()
                .map(|s| s.trim().to_string())
                .unwrap_or_default();
            if !trigger.is_empty() && trigger.len() < 100 {
                triggers.push(trigger);
            }
        }
    }

    triggers
}

/// Extract file patterns from description
/// Looks for file extensions and path patterns mentioned
fn extract_file_patterns(description: &str) -> Vec<String> {
    let mut patterns = Vec::new();
    
    // Common file extension patterns
    let ext_pattern = regex::Regex::new(r"\b\.[a-z]{1,4}\b").unwrap();
    for cap in ext_pattern.captures_iter(description) {
        patterns.push(cap[0].to_string());
    }

    // Path patterns like "SKILL.md", "*.json"
    let path_pattern = regex::Regex::new(r"\b[A-Za-z_]+\.[a-z]+\b").unwrap();
    for cap in path_pattern.captures_iter(description) {
        let path = cap[0].to_string();
        if !patterns.contains(&path) {
            patterns.push(path);
        }
    }

    patterns
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::{SkillIR, SecurityLevel};

    #[test]
    fn test_routing_manifest_creation() {
        let mut manifest = RoutingManifest::new();
        
        let ir = SkillIR {
            name: "test-skill".to_string(),
            description: "A test skill for demonstration. Use this when testing.".to_string(),
            version: "1.0".to_string(),
            security_level: SecurityLevel::Medium,
            ..Default::default()
        };

        manifest.add_skill(&ir);
        
        assert_eq!(manifest.total_skills, 1);
        assert!(manifest.skills.contains_key("test-skill"));
    }

    #[test]
    fn test_yaml_output() {
        let mut manifest = RoutingManifest::new();
        
        let ir = SkillIR {
            name: "yaml-test".to_string(),
            description: "Test YAML generation".to_string(),
            version: "1.0".to_string(),
            security_level: SecurityLevel::Low,
            ..Default::default()
        };

        manifest.add_skill(&ir);
        
        let yaml = manifest.to_yaml().unwrap();
        assert!(yaml.contains("yaml-test"));
        assert!(yaml.contains("Test YAML generation"));
    }

    #[test]
    fn test_trigger_extraction() {
        let description = "Use this skill when reviewing SQL migrations or database changes.";
        let triggers = extract_triggers(description);
        
        assert!(!triggers.is_empty());
        assert!(triggers[0].contains("reviewing"));
    }
}