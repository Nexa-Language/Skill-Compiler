//! Markdown Parser
//!
//! Parse Markdown body using pulldown-cmark event stream.

use pulldown_cmark::{Event, HeadingLevel, Parser, Tag, TagEnd};

/// Markdown body parsing result
#[derive(Debug, Clone, Default)]
pub struct MarkdownBody {
    /// Parsed sections
    pub sections: Vec<Section>,
    /// Parsed procedure steps
    pub procedures: Vec<RawProcedureStep>,
    /// Parsed examples
    pub examples: Vec<RawExample>,
    /// Code blocks found
    pub code_blocks: Vec<CodeBlock>,
}

/// A section in the Markdown body
#[derive(Debug, Clone)]
pub struct Section {
    /// Heading level (1-6)
    pub level: u8,
    /// Section title
    pub title: String,
    /// Section content
    pub content: String,
}

/// A raw procedure step from the Markdown
#[derive(Debug, Clone)]
pub struct RawProcedureStep {
    /// Step order (1-based)
    pub order: u32,
    /// Step text (title only, e.g. "Discover Available Tools")
    pub text: String,
    /// Whether this is a critical step
    pub is_critical: bool,
    /// Step body content — original Markdown text under the step heading,
    /// including code blocks, parameter lists, examples, etc.
    /// Empty for list-based procedure steps.
    pub body: String,
}

/// A raw example from the Markdown
#[derive(Debug, Clone)]
pub struct RawExample {
    /// Example title
    pub title: Option<String>,
    /// User input
    pub user_input: String,
    /// Agent response
    pub agent_response: String,
}

/// A code block from the Markdown
#[derive(Debug, Clone)]
pub struct CodeBlock {
    /// Language identifier
    pub language: Option<String>,
    /// Code content
    pub content: String,
}

/// Check if a section title indicates procedure-like content.
///
/// Extended matching covers real-world skill patterns:
/// - "Procedures" (standard)
/// - "步骤", "执行" (Chinese)
/// - "Core Workflow Pattern" (Slack/Discord/Salesforce automation skills)
/// - "Instructions" (file-organizer, invoice-organizer, etc.)
/// - "How to Use" / "How to" (changelog, canvas, etc.)
/// - "Common Workflow" (xlsx)
/// - "Skill Creation Process" (skill-creator)
/// - "Editing Workflow" (docx, pptx)
/// - "Creation Process" (skill-creator)
/// - Phases with steps (mcp-builder's "High-Level Workflow")
#[must_use]
fn is_procedure_section(title: &str) -> bool {
    let t = title.to_lowercase();
    t.contains("procedure")
        || t.contains("步骤")
        || t.contains("执行")
        || t.contains("workflow")
        || t.contains("instructions")
        || t.contains("how to use")
        || t.contains("how to")
        || t.contains("common workflow")
        || t.contains("core workflow")
        || t.contains("creation process")
        || t.contains("editing workflow")
        || t.contains("editing existing")
}

/// Extract procedure steps from section titles that match step/phase patterns.
///
/// Many real-world skills use sub-heading patterns like:
/// - "### Step 1: Discover Available Tools"
/// - "### 1. Gather Information"
/// - "#### 1.1 Understand Agent-Centric Design"
/// - "### Phase 1: Deep Research and Planning"
///
/// These appear as separate `Section` entries in the parsed result.
/// This function scans them and converts matching titles into procedure steps.
fn extract_heading_steps(sections: &[Section]) -> Vec<RawProcedureStep> {
    let mut steps = Vec::new();
    let mut order = 0u32;

    for section in sections {
        let title = section.title.trim();
        let body = section.content.trim().to_string();

        // Helper: create a step with title text and section body content
        fn make_step(order: &mut u32, text: String, body: String) -> Option<RawProcedureStep> {
            if text.is_empty() {
                return None;
            }
            *order += 1;
            let is_critical = text.contains("[CRITICAL]");
            Some(RawProcedureStep {
                order: *order,
                text,
                is_critical,
                body,
            })
        }

        // Pattern: "Step 1: Discover Available Tools"
        if let Some(rest) = title.strip_prefix("Step ") {
            let text = rest.split_once(':')
                .or_else(|| rest.split_once(' '))
                .map(|(_, t)| t.trim().to_string())
                .unwrap_or(rest.trim().to_string());
            if let Some(step) = make_step(&mut order, text, body) {
                steps.push(step);
            }
            continue;
        }

        // Pattern: "Phase 1: Deep Research and Planning"
        if let Some(rest) = title.strip_prefix("Phase ") {
            let text = rest.split_once(':')
                .or_else(|| rest.split_once(' '))
                .map(|(_, t)| t.trim().to_string())
                .unwrap_or(rest.trim().to_string());
            if let Some(step) = make_step(&mut order, text, body) {
                steps.push(step);
            }
            continue;
        }

        // Pattern: "1.1 Understand ..." (dotted sub-step like "1.1", "2.3")
        // Must be checked BEFORE the simple "N." pattern to avoid mis-matching
        if let Some(space_pos) = title.find(' ') {
            let prefix = &title[..space_pos];
            // Verify prefix matches "N.N" (single digit, dot, single/triple digit)
            if prefix.chars().next().is_some_and(|c| c.is_ascii_digit())
                && prefix.contains('.')
                && prefix.chars().filter(|c| *c == '.').count() == 1
                && prefix.ends_with(|c: char| c.is_ascii_digit())
            {
                let text = title[space_pos + 1..].trim().to_string();
                if let Some(step) = make_step(&mut order, text, body) {
                    steps.push(step);
                }
                continue;
            }
        }

        // Pattern: "1. Gather Information" (numbered sub-heading, N. ...)
        if let Some(dot_pos) = title.find('.') {
            if dot_pos > 0 && dot_pos <= 3 {
                let prefix = &title[..dot_pos];
                if prefix.chars().all(|c| c.is_ascii_digit()) {
                    let text = title[dot_pos + 1..].trim().to_string();
                    if let Some(step) = make_step(&mut order, text, body) {
                        steps.push(step);
                    }
                }
            }
        }
    }

    steps
}

/// Parse Markdown body content
///
/// # Arguments
///
/// * `body` - The Markdown body text (after frontmatter)
///
/// # Returns
///
/// A `MarkdownBody` containing parsed sections, procedures, examples, and code blocks.
#[must_use]
pub fn parse_markdown_body(body: &str) -> MarkdownBody {
    let parser = Parser::new(body);
    let mut result = MarkdownBody::default();

    // Parser state
    #[derive(Debug, Clone, PartialEq)]
    enum ParseState {
        Default,
        InHeading(u8),
        InProcedureList,
        InExampleBlockquote,
        InCodeBlock(Option<String>),
    }

    let mut state = ParseState::Default;
    let mut current_section_title = String::new();
    let mut current_content = String::new();
    let mut procedure_counter = 0u32;
    let mut current_code_block = String::new();
    let mut current_code_lang: Option<String> = None;

    for event in parser {
        match event {
            Event::Start(Tag::Heading { level, .. }) => {
                // Save previous section
                if !current_section_title.is_empty() {
                    let level = match state {
                        ParseState::InHeading(l) => l,
                        _ => 0,
                    };
                    result.sections.push(Section {
                        level,
                        title: current_section_title.clone(),
                        content: current_content.trim().to_string(),
                    });
                }
                current_section_title.clear();
                current_content.clear();

                state = ParseState::InHeading(match level {
                    HeadingLevel::H1 => 1,
                    HeadingLevel::H2 => 2,
                    HeadingLevel::H3 => 3,
                    HeadingLevel::H4 => 4,
                    HeadingLevel::H5 => 5,
                    HeadingLevel::H6 => 6,
                });
            }

            Event::End(TagEnd::Heading(_)) => {
                state = ParseState::Default;
            }

            Event::Text(text) => {
                match state {
                    ParseState::InHeading(_) => {
                        current_section_title.push_str(&text);
                    }
                    ParseState::InProcedureList => {
                        // Check for CRITICAL marker
                        let text_str = text.as_ref();
                        let is_critical = text_str.contains("[CRITICAL]");
                        let clean_text = text_str.replace("[CRITICAL]", "").trim().to_string();

                        if !clean_text.is_empty() {
                            procedure_counter += 1;
                            result.procedures.push(RawProcedureStep {
                                order: procedure_counter,
                                text: clean_text,
                                is_critical,
                                body: String::new(), // List-based steps have no separate body
                            });
                        }
                    }
                    ParseState::InCodeBlock(_) => {
                        current_code_block.push_str(&text);
                    }
                    _ => {
                        current_content.push_str(&text);
                    }
                }
            }

            Event::Start(Tag::List(Some(_))) => {
                // Ordered list - check if this is a procedure-like section.
                // Extended matching to cover real-world skill patterns:
                // "Procedures", "步骤", "执行", "Core Workflow Pattern",
                // "Instructions", "How to Use", "Common Workflow", etc.
                if is_procedure_section(&current_section_title)
                {
                    state = ParseState::InProcedureList;
                    procedure_counter = 0;
                }
            }

            Event::End(TagEnd::List(_)) => {
                if state == ParseState::InProcedureList {
                    state = ParseState::Default;
                }
            }

            Event::Start(Tag::CodeBlock(kind)) => {
                current_code_block.clear();
                current_code_lang = match kind {
                    pulldown_cmark::CodeBlockKind::Fenced(lang) => Some(lang.to_string()),
                    pulldown_cmark::CodeBlockKind::Indented => None,
                };
                state = ParseState::InCodeBlock(current_code_lang.clone());
            }

            Event::End(TagEnd::CodeBlock) => {
                result.code_blocks.push(CodeBlock {
                    language: current_code_lang.clone(),
                    content: current_code_block.trim().to_string(),
                });
                // Also append code block to current section content as Markdown-formatted text,
                // so that Section.content preserves the full body including code examples.
                // This is critical for heading-based procedure steps to carry their
                // code blocks, parameter descriptions, etc. into the compiled output.
                if !current_code_block.is_empty() {
                    if let Some(lang) = &current_code_lang {
                        current_content.push_str(&format!(
                            "\n```{}\n{}\n```",
                            lang,
                            current_code_block.trim()
                        ));
                    } else {
                        current_content.push_str(&format!(
                            "\n```\n{}\n```",
                            current_code_block.trim()
                        ));
                    }
                }
                state = ParseState::Default;
            }

            Event::Start(Tag::BlockQuote) => {
                if current_section_title.contains("Example")
                    || current_section_title.contains("示例")
                {
                    state = ParseState::InExampleBlockquote;
                }
            }

            Event::End(TagEnd::BlockQuote) => {
                if state == ParseState::InExampleBlockquote {
                    state = ParseState::Default;
                }
            }

            _ => {}
        }
    }

    // Save last section
    if !current_section_title.is_empty() {
        result.sections.push(Section {
            level: 0,
            title: current_section_title,
            content: current_content.trim().to_string(),
        });
    }

    // Post-processing: extract procedure steps from heading patterns.
    // Many real-world skills use "### Step 1:", "### 1. Gather Information",
    // "#### 1.1 Understand ..." sub-headings instead of ordered lists.
    // Only add heading-based steps if we haven't already extracted from lists.
    if result.procedures.is_empty() {
        let heading_steps = extract_heading_steps(&result.sections);
        result.procedures.extend(heading_steps);
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_procedures() {
        let body = r#"## Procedures

1. First step.
2. [CRITICAL] Second step is critical.
3. Third step.
"#;

        let result = parse_markdown_body(body);
        assert!(!result.procedures.is_empty(), "Should parse procedures");
        let first = result.procedures.iter().find(|p| p.order == 1);
        assert!(first.is_some(), "Should find first procedure");
        assert_eq!(first.unwrap().text, "First step.");
        // List-based steps have empty body
        assert_eq!(first.unwrap().body, "");
    }

    #[test]
    fn test_parse_sections() {
        let body = r#"# Main Title

## Description

This is a description.

## Procedures

1. Step one.
"#;

        let result = parse_markdown_body(body);
        assert!(!result.sections.is_empty());
    }

    #[test]
    fn test_heading_steps_with_body() {
        let body = r#"## Core Workflow Pattern

### Step 1: Discover Available Tools

```
RUBE_SEARCH_TOOLS
queries: [{use_case: "your task"}]
session: {generate_id: true}
```

This returns available tool slugs and input schemas.

### Step 2: Check Connection

```
RUBE_MANAGE_CONNECTIONS
toolkits: ["active_campaign"]
session_id: "your_session_id"
```

### Step 3: Execute Tools

Some final step content.
"#;

        let result = parse_markdown_body(body);
        assert_eq!(result.procedures.len(), 3, "Should extract 3 heading-based steps");

        // Step 1 should have body with code block
        let step1 = result.procedures.iter().find(|p| p.order == 1).unwrap();
        assert_eq!(step1.text, "Discover Available Tools");
        assert!(step1.body.contains("RUBE_SEARCH_TOOLS"), "Body should contain code block content");
        assert!(step1.body.contains("```"), "Body should preserve code block fences");

        // Step 2 should have body with code block
        let step2 = result.procedures.iter().find(|p| p.order == 2).unwrap();
        assert_eq!(step2.text, "Check Connection");
        assert!(step2.body.contains("RUBE_MANAGE_CONNECTIONS"));

        // Step 3 should have body with plain text
        let step3 = result.procedures.iter().find(|p| p.order == 3).unwrap();
        assert_eq!(step3.text, "Execute Tools");
        assert!(step3.body.contains("Some final step content"));
    }

    #[test]
    fn test_section_content_includes_code_blocks() {
        let body = r#"## Setup

1. Add the MCP server:
```
https://rube.app/mcp
```
2. Connect your account when prompted.
"#;

        let result = parse_markdown_body(body);
        let setup_section = result.sections.iter().find(|s| s.title == "Setup");
        assert!(setup_section.is_some(), "Should find Setup section");
        let content = &setup_section.unwrap().content;
        // Section content should now include the code block as Markdown
        assert!(content.contains("https://rube.app/mcp"), "Section content should include code block text");
        assert!(content.contains("```"), "Section content should preserve code fences");
    }

    #[test]
    fn test_numbered_heading_step_with_body() {
        let body = r#"## Procedures

### 1. Gather Information

First, collect all relevant data from the system.

```
SELECT * FROM users WHERE active = true;
```

### 2. Analyze Results

Process the gathered data.
"#;

        let result = parse_markdown_body(body);
        assert_eq!(result.procedures.len(), 2);

        let step1 = result.procedures.iter().find(|p| p.order == 1).unwrap();
        assert_eq!(step1.text, "Gather Information");
        assert!(step1.body.contains("SELECT * FROM users"));
        assert!(step1.body.contains("```"));

        let step2 = result.procedures.iter().find(|p| p.order == 2).unwrap();
        assert_eq!(step2.text, "Analyze Results");
        assert!(step2.body.contains("Process the gathered data"));
    }
}
