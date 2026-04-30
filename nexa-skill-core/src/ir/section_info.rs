//! Section Info Definition
//!
//! Represents an arbitrary Markdown section that wasn't captured
//! by specific SkillIR fields (procedures, constraints, etc.)
//! but still carries meaningful content for the compiled output.

use serde::{Deserialize, Serialize};

/// A section from the Markdown body that carries additional context
/// not captured by structured SkillIR fields.
///
/// Examples: "Setup", "Prerequisites", "Known Pitfalls",
/// "Quick Reference", "Core Workflows", "Tool Discovery", etc.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SectionInfo {
    /// Section heading level (1-6)
    pub level: u8,
    /// Section title (e.g. "Setup", "Known Pitfalls")
    pub title: String,
    /// Section content in original Markdown format,
    /// including code blocks, lists, tables, etc.
    pub content: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_section_info_construction() {
        let section = SectionInfo {
            level: 2,
            title: "Setup".to_string(),
            content: "1. Add MCP server\n2. Connect account".to_string(),
        };
        assert_eq!(section.level, 2);
        assert_eq!(section.title, "Setup");
        assert!(section.content.contains("MCP server"));
    }
}