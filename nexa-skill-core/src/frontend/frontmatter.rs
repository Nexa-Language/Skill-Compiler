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
/// Tolerant parsing: allows leading whitespace/newline before `---`.
/// Validates name format (kebab-case) and checks description for XML tags.
///
/// # Errors
///
/// Returns an error if:
/// - No frontmatter is found (or non-whitespace precedes `---`)
/// - The frontmatter is empty
/// - The YAML is malformed
/// - Name is not kebab-case format
/// - Description contains non-safe XML tags
pub fn extract_frontmatter(content: &str) -> Result<(FrontmatterMeta, &str), ParseError> {
    // Find the start of frontmatter (tolerant: allow leading whitespace)
    let start = content
        .find("---")
        .ok_or(ParseError::MissingFrontmatter)?;

    // Ensure --- is at the beginning (after optional whitespace only)
    let prefix = &content[..start];
    if !prefix.trim().is_empty() {
        // Non-whitespace before --- means it's not frontmatter
        return Err(ParseError::MissingFrontmatter);
    }

    // Find the closing ---
    let after_open = &content[start + 3..];
    let end_idx = after_open
        .find("\n---")
        .ok_or(ParseError::MissingFrontmatter)?;

    let yaml_content = &after_open[..end_idx];

    // Calculate body start position in original content
    let body_start = start + 3 + end_idx + 4; // --- + yaml + \n---
    let body = content[body_start.min(content.len())..].trim_start_matches('\n');

    // Check for empty frontmatter
    if yaml_content.trim().is_empty() {
        return Err(ParseError::EmptyFrontmatter);
    }

    // Parse YAML
    let mut meta: FrontmatterMeta =
        serde_yaml::from_str(yaml_content).map_err(|e| ParseError::YamlError(e.to_string()))?;

    // Normalize name to kebab-case if it doesn't already conform.
    // Real-world skill files may use "Ahrefs Automation" or "Canvas Design"
    // which are human-readable but not valid kebab-case identifiers.
    // Automatic normalization preserves intent while satisfying the format constraint.
    if !is_valid_kebab_case(&meta.name) {
        let normalized = normalize_to_kebab_case(&meta.name);
        if is_valid_kebab_case(&normalized) {
            meta.name = normalized;
        } else {
            return Err(ParseError::InvalidNameFormat(meta.name));
        }
    }

    // Validate name format (kebab-case, early detection) — now guaranteed

    // Check for XML tags in description (compile-time error)
    if contains_xml_tags(&meta.description) {
        return Err(ParseError::XmlTagsInDescription(meta.description));
    }

    Ok((meta, body))
}

/// Normalize a name string to kebab-case format.
///
/// Converts: uppercase → lowercase, spaces/underscores → hyphens,
/// removes consecutive hyphens, strips leading/trailing hyphens.
/// Truncates to 64 characters if needed.
///
/// Examples:
/// - "Ahrefs Automation" → "ahrefs-automation"
/// - "Canvas Design" → "canvas-design"
/// - "My_Cool_Skill" → "my-cool-skill"
fn normalize_to_kebab_case(name: &str) -> String {
    let mut result = String::new();
    let mut prev_was_hyphen = false;

    for ch in name.chars() {
        if ch.is_ascii_uppercase() {
            result.push(ch.to_ascii_lowercase());
            prev_was_hyphen = false;
        } else if ch == ' ' || ch == '_' {
            if !prev_was_hyphen && !result.is_empty() {
                result.push('-');
                prev_was_hyphen = true;
            }
        } else if ch == '-' {
            if !prev_was_hyphen && !result.is_empty() {
                result.push('-');
                prev_was_hyphen = true;
            }
        } else if ch.is_ascii_lowercase() || ch.is_ascii_digit() {
            result.push(ch);
            prev_was_hyphen = false;
        }
        // Skip other characters (punctuation, symbols, etc.)
    }

    // Remove trailing hyphen if present
    if result.ends_with('-') {
        result.pop();
    }

    // Truncate to 64 characters
    if result.len() > 64 {
        result = result[..64].to_string();
        // Remove trailing hyphen after truncation
        if result.ends_with('-') {
            result.pop();
        }
    }

    result
}

/// Check if name is valid kebab-case format
///
/// Rules: 1-64 chars, lowercase letters/digits/hyphens only,
/// no leading/trailing hyphens, no double hyphens.
fn is_valid_kebab_case(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 64
        && name.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
        && !name.starts_with('-')
        && !name.ends_with('-')
        && !name.contains("--")
}

/// Check if text contains XML/HTML tags that would interfere with LLM parsing
///
/// Safe Markdown HTML tags (`<em>`, `<strong>`, `<code>`, `<pre>`, `<p>`,
/// `<br>`, `<hr>`, `<a>`, `<span>`, `<div>`) are allowed.
/// Custom/semantic tags like `<warning>`, `<constraint>`, `<danger>` trigger an error.
fn contains_xml_tags(text: &str) -> bool {
    let safe_tags: [&str; 10] = ["br", "hr", "em", "strong", "code", "pre", "p", "span", "div", "a"];

    let chars: Vec<char> = text.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '<' && i + 1 < chars.len() {
            let next = chars[i + 1];
            // Skip closing tags </...> and comments <!--
            if next == '/' || next == '!' {
                i += 1;
                continue;
            }
            // Check if it starts with a letter (XML tag)
            if next.is_ascii_alphabetic() {
                // Find the closing >
                let mut j = i + 1;
                while j < chars.len() && chars[j] != '>' {
                    j += 1;
                }
                if j < chars.len() {
                    // Found a complete XML tag — extract tag name
                    let tag_name_start = i + 1;
                    let tag_name_end = chars[tag_name_start..j]
                        .iter()
                        .position(|c| !c.is_ascii_alphabetic() && *c != '-' && *c != '_')
                        .map(|p| tag_name_start + p)
                        .unwrap_or(j);
                    let tag_name: String = chars[tag_name_start..tag_name_end].iter().collect();
                    // Only flag non-safe tags
                    if !safe_tags.contains(&tag_name.as_str()) {
                        return true;
                    }
                }
            }
        }
        i += 1;
    }
    false
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

    #[test]
    fn test_valid_frontmatter_with_whitespace() {
        // Leading whitespace before --- should be tolerated
        let content = "   \n---\nname: test-skill\ndescription: A test skill\n---\n\n# Body\n";
        let result = extract_frontmatter(content);
        assert!(result.is_ok());
        let (meta, body) = result.unwrap();
        assert_eq!(meta.name, "test-skill");
        assert!(body.contains("# Body"));
    }

    #[test]
    fn test_xml_tags_in_description() {
        let content = "---\nname: test-skill\ndescription: <danger>Do not do this</danger>\n---\n\n# Body\n";
        let result = extract_frontmatter(content);
        assert!(result.is_err());
        match result.unwrap_err() {
            ParseError::XmlTagsInDescription(desc) => {
                assert!(desc.contains("<danger>"));
            }
            other => panic!("Expected XmlTagsInDescription, got: {other}"),
        }
    }

    #[test]
    fn test_safe_html_tags_allowed() {
        // Safe Markdown HTML tags should not trigger error
        let content = "---\nname: test-skill\ndescription: Use <em>emphasis</em> and <strong>bold</strong> text\n---\n\n# Body\n";
        let result = extract_frontmatter(content);
        assert!(result.is_ok());
        let (meta, _) = result.unwrap();
        assert_eq!(meta.name, "test-skill");
    }

    #[test]
    fn test_invalid_name_format_auto_normalized() {
        // Name with uppercase is now auto-normalized to kebab-case
        // "TestSkill" → "testskill" (no spaces, so no hyphens inserted)
        let content = "---\nname: TestSkill\ndescription: A test skill\n---\n\n# Body\n";
        let result = extract_frontmatter(content);
        assert!(result.is_ok());
        let (meta, _) = result.unwrap();
        assert_eq!(meta.name, "testskill");
    }

    #[test]
    fn test_name_with_spaces_auto_normalized() {
        // "Ahrefs Automation" → "ahrefs-automation"
        let content = "---\nname: Ahrefs Automation\ndescription: SEO research\n---\n\n# Body\n";
        let result = extract_frontmatter(content);
        assert!(result.is_ok());
        let (meta, _) = result.unwrap();
        assert_eq!(meta.name, "ahrefs-automation");
    }

    #[test]
    fn test_valid_kebab_case_name() {
        let content = "---\nname: my-cool-skill-v2\ndescription: A test skill\n---\n\n# Body\n";
        let result = extract_frontmatter(content);
        assert!(result.is_ok());
        let (meta, _) = result.unwrap();
        assert_eq!(meta.name, "my-cool-skill-v2");
    }

    #[test]
    fn test_empty_frontmatter_tolerant() {
        let content = "---\n\n---\n\n# Body\n";
        let result = extract_frontmatter(content);
        assert!(result.is_err());
        match result.unwrap_err() {
            ParseError::EmptyFrontmatter => {}
            other => panic!("Expected EmptyFrontmatter, got: {other}"),
        }
    }

    #[test]
    fn test_leading_newline_tolerant() {
        let content = "\n---\nname: test-skill\ndescription: A test skill\n---\n\n# Body\n";
        let result = extract_frontmatter(content);
        assert!(result.is_ok());
        let (meta, body) = result.unwrap();
        assert_eq!(meta.name, "test-skill");
        assert!(body.contains("# Body"));
    }

    #[test]
    fn test_name_starts_with_hyphen_auto_normalized() {
        // "-skill" → "skill" (leading hyphen stripped by normalize)
        let content = "---\nname: -skill\ndescription: A test skill\n---\n\n# Body\n";
        let result = extract_frontmatter(content);
        assert!(result.is_ok());
        let (meta, _) = result.unwrap();
        assert_eq!(meta.name, "skill");
    }

    #[test]
    fn test_name_double_hyphen_auto_normalized() {
        // "test--skill" → "test-skill" (consecutive hyphens collapsed)
        let content = "---\nname: test--skill\ndescription: A test skill\n---\n\n# Body\n";
        let result = extract_frontmatter(content);
        assert!(result.is_ok());
        let (meta, _) = result.unwrap();
        assert_eq!(meta.name, "test-skill");
    }

    #[test]
    fn test_truly_invalid_name_still_rejected() {
        // A name that cannot be normalized (e.g. all special chars) should still be rejected
        let content = "---\nname: !!!\ndescription: A test skill\n---\n\n# Body\n";
        let result = extract_frontmatter(content);
        assert!(result.is_err());
    }

    // --- Helper function unit tests ---

    #[test]
    fn test_is_valid_kebab_case() {
        assert!(is_valid_kebab_case("test-skill"));
        assert!(is_valid_kebab_case("my-cool-skill-v2"));
        assert!(is_valid_kebab_case("a"));           // single char
        assert!(is_valid_kebab_case("skill123"));    // digits allowed

        assert!(!is_valid_kebab_case(""));            // empty
        assert!(!is_valid_kebab_case("-skill"));      // leading hyphen
        assert!(!is_valid_kebab_case("skill-"));      // trailing hyphen
        assert!(!is_valid_kebab_case("test--skill")); // double hyphen
        assert!(!is_valid_kebab_case("TestSkill"));   // uppercase
        assert!(!is_valid_kebab_case("test skill"));  // space
        assert!(!is_valid_kebab_case("a-very-very-very-very-very-very-very-very-long-name-exceeds-limits")); // 70 chars >64
    }

    #[test]
    fn test_contains_xml_tags() {
        // Unsafe tags → true
        assert!(contains_xml_tags("<warning>be careful</warning>"));
        assert!(contains_xml_tags("<constraint>do this</constraint>"));
        assert!(contains_xml_tags("<danger>avoid</danger>"));
        assert!(contains_xml_tags("<custom>text</custom>"));

        // Safe tags → false
        assert!(!contains_xml_tags("<em>emphasis</em>"));
        assert!(!contains_xml_tags("<strong>bold</strong>"));
        assert!(!contains_xml_tags("<code>inline code</code>"));
        assert!(!contains_xml_tags("<pre>block</pre>"));
        assert!(!contains_xml_tags("<p>paragraph</p>"));
        assert!(!contains_xml_tags("<br>"));
        assert!(!contains_xml_tags("<hr>"));
        assert!(!contains_xml_tags("<a href=\"url\">link</a>"));
        assert!(!contains_xml_tags("<span>text</span>"));
        assert!(!contains_xml_tags("<div>block</div>"));

        // No tags → false
        assert!(!contains_xml_tags("plain text only"));
        assert!(!contains_xml_tags("text with <no angle brackets")); // incomplete tag, no closing >

        // Closing tags and comments → false
        assert!(!contains_xml_tags("</closing>"));
        assert!(!contains_xml_tags("<!-- comment -->"));
    }
}
