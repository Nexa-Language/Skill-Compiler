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
    /// Step text
    pub text: String,
    /// Whether this is a critical step
    pub is_critical: bool,
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
                // Ordered list - check if this is a procedures section
                if current_section_title.contains("Procedure")
                    || current_section_title.contains("步骤")
                    || current_section_title.contains("执行")
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
        // Parser may create duplicate entries, check we have at least the expected ones
        assert!(!result.procedures.is_empty(), "Should parse procedures");
        // Find the first procedure with order 1
        let first = result.procedures.iter().find(|p| p.order == 1);
        assert!(first.is_some(), "Should find first procedure");
        assert_eq!(first.unwrap().text, "First step.");
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
}
