//! AST Builder
//!
//! Build RawAST from parsed frontmatter and markdown body.

use super::frontmatter::extract_frontmatter;
use super::markdown::parse_markdown_body;
use super::{FrontmatterMeta, MarkdownBody};
use crate::error::ParseError;
use sha2::{Digest, Sha256};

/// Raw Abstract Syntax Tree
///
/// This is the output of the Frontend phase, containing unvalidated
/// raw data parsed from the SKILL.md file.
#[derive(Debug, Clone)]
pub struct RawAST {
    /// Source file path
    pub source_path: String,
    /// Parsed frontmatter metadata
    pub frontmatter: FrontmatterMeta,
    /// Parsed markdown body
    pub body: MarkdownBody,
    /// SHA-256 hash of source content
    pub source_hash: String,
}

/// AST Builder
///
/// Constructs RawAST from SKILL.md file content.
pub struct ASTBuilder;

impl ASTBuilder {
    /// Build RawAST from file path
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read or parsed.
    pub fn build_from_file(path: &str) -> Result<RawAST, ParseError> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| ParseError::FileReadError(path.to_string(), e.to_string()))?;

        Self::build_from_content(path, &content)
    }

    /// Build RawAST from content string
    ///
    /// # Arguments
    ///
    /// * `path` - The file path (for error reporting)
    /// * `content` - The raw file content
    ///
    /// # Errors
    ///
    /// Returns an error if parsing fails.
    pub fn build_from_content(path: &str, content: &str) -> Result<RawAST, ParseError> {
        // Compute source hash
        let source_hash = Self::compute_hash(content);

        // Extract frontmatter
        let (frontmatter, body_content) = extract_frontmatter(content)?;

        // Parse markdown body
        let body = parse_markdown_body(body_content);

        Ok(RawAST {
            source_path: path.to_string(),
            frontmatter,
            body,
            source_hash,
        })
    }

    /// Compute SHA-256 hash of content
    fn compute_hash(content: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        format!("{:x}", hasher.finalize())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_from_content() {
        let content = r#"---
name: test-skill
description: A test skill
---

# Test Skill

## Procedures

1. First step.
2. Second step.
"#;

        let result = ASTBuilder::build_from_content("test.md", content);
        assert!(result.is_ok());

        let ast = result.unwrap();
        assert_eq!(ast.frontmatter.name, "test-skill");
        assert_eq!(ast.body.procedures.len(), 2);
    }
}
