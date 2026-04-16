//! Markdown Parser
//!
//! Parse Markdown body using pulldown-cmark event stream.
//!
//! Supports 6 skill structure patterns via multi-strategy extraction:
//! 1. Explicit steps (ordered list + heading patterns)
//! 2. Workflow extraction (Workflow N: Title sub-headings)
//! 3. Section-as-phase (sequential phase sections)
//! 4. Mode-as-approach (alternative approaches under mode-selector)
//! 5. Reference-as-operation (operation sub-headings under reference)
//! 6. Full-body fallback (entire content as single step)

use pulldown_cmark::{Event, HeadingLevel, Parser, Tag, TagEnd};

/// Section kind classification for multi-strategy extraction
#[derive(Debug, Clone, PartialEq)]
pub enum SectionKind {
    /// 步骤型: procedure, instructions, workflow, steps, 步骤, 执行
    Procedure,
    /// 模式选择型: how to use, usage, ways to use, using
    ModeSelector,
    /// 上下文型: when to use, prerequisites, setup, overview
    Context,
    /// 参考型: common tasks, quick start, quick reference, libraries
    Reference,
    /// 准则型: tips, best practices, pitfalls, guidelines
    Guideline,
    /// 示例型: example, examples
    Example,
    /// 元数据型: dependencies, resources, related, see also
    Metadata,
    /// 未分类
    Unknown,
}

/// Markdown body parsing result
#[derive(Debug, Clone, Default)]
pub struct MarkdownBody {
    /// Parsed sections
    pub sections: Vec<Section>,
    /// Parsed procedure steps
    pub procedures: Vec<RawProcedureStep>,
    /// Parsed alternative approaches (for mode-selector skills)
    pub approaches: Vec<RawApproach>,
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
    /// Classified section kind
    pub kind: SectionKind,
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

/// A raw alternative approach from mode-selector sections
#[derive(Debug, Clone)]
pub struct RawApproach {
    /// Approach name (from sub-heading title)
    pub name: String,
    /// One-line description of this approach
    pub description: String,
    /// Full instructions for this approach
    pub instructions: String,
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

/// Classify a section title into a SectionKind.
///
/// Uses conservative matching: only classifies when clear keywords are present.
/// Priority: ModeSelector/Example checked before Procedure to handle overlaps
/// like "example workflow" and "how to use".
#[must_use]
pub fn classify_section(title: &str) -> SectionKind {
    let t = title.to_lowercase();

    // ModeSelector: must be checked before Procedure/Example
    if t.contains("how to use")
        || t.contains("usage")
        || t.contains("ways to use")
        || t.contains("using")
    {
        return SectionKind::ModeSelector;
    }

    // "example workflow" → ModeSelector, not Example or Procedure
    if t.contains("example workflow") {
        return SectionKind::ModeSelector;
    }

    // Example: contains "example" (but not "example workflow")
    if t.contains("example") {
        return SectionKind::Example;
    }

    // Procedure: explicit step/workflow keywords
    if t.contains("procedure")
        || t.contains("instructions")
        || t.contains("workflow")
        || t.contains("steps")
        || t.contains("步骤")
        || t.contains("执行")
        || t.contains("core workflow pattern")
        || t.contains("editing workflow")
        || t.contains("creation process")
        || t.contains("common workflow")
    {
        return SectionKind::Procedure;
    }

    // Context
    if t.contains("when to use")
        || t.contains("prerequisites")
        || t.contains("setup")
        || t.contains("overview")
        || t.contains("context")
        || t.contains("purpose")
    {
        return SectionKind::Context;
    }

    // Reference
    if t.contains("common tasks")
        || t.contains("quick start")
        || t.contains("quick reference")
        || t.contains("libraries")
        || t.contains("command-line")
        || t.contains("tools")
    {
        return SectionKind::Reference;
    }

    // Guideline
    if t.contains("tips")
        || t.contains("best practices")
        || t.contains("pitfalls")
        || t.contains("guidelines")
        || t.contains("common mistake")
        || t.contains("pro tips")
    {
        return SectionKind::Guideline;
    }

    // Metadata
    if t.contains("dependencies")
        || t.contains("resources")
        || t.contains("related")
        || t.contains("see also")
        || t.contains("reference files")
    {
        return SectionKind::Metadata;
    }

    SectionKind::Unknown
}

/// Check if a section title indicates procedure-like content.
///
/// Extended matching covers real-world skill patterns.
/// Now delegates to [`classify_section()`] for consistency.
#[deprecated(note = "Use classify_section() instead. This function is kept for backward compatibility.")]
#[allow(dead_code)]
#[must_use]
fn is_procedure_section(title: &str) -> bool {
    classify_section(title) == SectionKind::Procedure
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

/// Strategy 2: Extract workflow steps from Procedure sections.
///
/// Finds Procedure-kind sections and treats their sub-headings containing
/// "Workflow" as individual procedure steps. Each "Workflow N: Title"
/// sub-heading becomes a step with its full content as body.
fn extract_workflow_steps(sections: &[Section]) -> Vec<RawProcedureStep> {
    let mut steps = Vec::new();
    let mut order = 0u32;
    let mut in_procedure = false;

    for section in sections {
        // Track when we're inside a Procedure section
        if section.kind == SectionKind::Procedure && section.level <= 3 {
            in_procedure = true;
        } else if section.level <= 2 {
            in_procedure = false;
        }

        // Sub-headings within a Procedure section that contain "Workflow"
        if in_procedure && section.level >= 3 {
            let t = section.title.to_lowercase();
            if t.contains("workflow") {
                order += 1;
                // Parse "Workflow N: Title" → extract just "Title"
                let text = if let Some(rest) = section.title.strip_prefix("Workflow ") {
                    rest.split_once(':')
                        .or_else(|| rest.split_once(' '))
                        .map(|(_, t)| t.trim().to_string())
                        .unwrap_or(rest.trim().to_string())
                } else {
                    section.title.clone()
                };
                steps.push(RawProcedureStep {
                    order,
                    text,
                    is_critical: section.title.contains("[CRITICAL]"),
                    body: section.content.clone(),
                });
            }
        }
    }

    steps
}

/// Strategy 3: Extract phase sections as sequential steps.
///
/// Identifies ##-level sections whose titles imply sequential phases:
/// - Contains phase keywords: "step", "phase", "creation", "process", etc.
/// - Starts with action verbs: "Creating", "Editing", "Analyzing", etc.
/// Excludes Context-type sections and already-classified sections.
fn extract_phase_sections(sections: &[Section]) -> Vec<RawProcedureStep> {
    let mut steps = Vec::new();
    let mut order = 0u32;

    let phase_keywords = [
        "step", "phase", "creation", "process", "final",
        "setup", "editing", "deducing",
    ];

    let action_verbs = [
        "creating", "editing", "analyzing", "extracting",
        "designing", "building", "generating", "processing",
        "implementing", "deploying",
    ];

    for section in sections {
        // Only ##-level sections (top-level phases)
        if section.level != 2 {
            continue;
        }

        // Exclude Context-type sections (Overview, Prerequisites, etc.)
        if section.kind == SectionKind::Context {
            continue;
        }

        // Exclude already-classified sections (Procedure, ModeSelector, etc.)
        if section.kind != SectionKind::Unknown {
            continue;
        }

        let t = section.title.to_lowercase();

        // Check phase keywords in title
        let has_phase_keyword = phase_keywords.iter().any(|k| t.contains(k));

        // Check action verbs at title start
        let starts_with_verb = action_verbs.iter().any(|v| t.starts_with(v));

        if has_phase_keyword || starts_with_verb {
            order += 1;
            steps.push(RawProcedureStep {
                order,
                text: section.title.clone(),
                is_critical: section.title.contains("[CRITICAL]"),
                body: section.content.clone(),
            });
        }
    }

    steps
}

/// Strategy 4: Extract alternative approaches from mode-selector sections.
///
/// Finds ModeSelector sections and treats their ###-level sub-headings
/// as distinct approaches. Each approach has:
/// - name: sub-heading title
/// - description: first non-empty line of content
/// - instructions: full section content
fn extract_mode_approaches(sections: &[Section]) -> Vec<RawApproach> {
    let mut approaches = Vec::new();
    let mut in_mode_selector = false;

    for section in sections {
        // Track when we're inside a ModeSelector section
        if section.kind == SectionKind::ModeSelector {
            in_mode_selector = true;
            // If the ModeSelector section itself has content (no sub-headings),
            // treat it as a single approach
            if section.level == 2 && !section.content.trim().is_empty() {
                let desc = section.content.lines()
                    .find(|l| !l.trim().is_empty())
                    .map(|l| l.trim().to_string())
                    .unwrap_or_default();
                approaches.push(RawApproach {
                    name: section.title.clone(),
                    description: desc,
                    instructions: section.content.clone(),
                });
            }
        } else if section.level <= 2 {
            // Reset when exiting the ModeSelector section
            in_mode_selector = false;
        }

        // ###-level sub-headings within a ModeSelector section
        if in_mode_selector && section.level == 3 {
            let desc = section.content.lines()
                .find(|l| !l.trim().is_empty())
                .map(|l| l.trim().to_string())
                .unwrap_or_default();
            approaches.push(RawApproach {
                name: section.title.clone(),
                description: desc,
                instructions: section.content.clone(),
            });
        }
    }

    approaches
}

/// Strategy 5: Extract operations from reference sections.
///
/// Finds Reference sections and treats their ###+ level sub-headings
/// as individual operations (procedure steps with code examples).
fn extract_reference_operations(sections: &[Section]) -> Vec<RawProcedureStep> {
    let mut steps = Vec::new();
    let mut order = 0u32;
    let mut in_reference = false;

    for section in sections {
        // Track when we're inside a Reference section
        if section.kind == SectionKind::Reference {
            in_reference = true;
        } else if section.level <= 2 {
            in_reference = false;
        }

        // Sub-headings within a Reference section become operation steps
        if in_reference && section.level >= 3 {
            order += 1;
            steps.push(RawProcedureStep {
                order,
                text: section.title.clone(),
                is_critical: false,
                body: section.content.clone(),
            });
        }
    }

    steps
}

/// Strategy 6: Full-body fallback — create a single catch-all step.
///
/// Used when no other strategy can extract structured procedures.
/// Creates a single step with the entire body content, truncated to 2000 chars.
fn extract_full_body_step(body_text: &str) -> RawProcedureStep {
    let truncated = if body_text.len() > 2000 {
        &body_text[..2000]
    } else {
        body_text
    };

    RawProcedureStep {
        order: 1,
        text: "Follow the skill instructions".to_string(),
        is_critical: false,
        body: truncated.to_string(),
    }
}

/// Multi-strategy procedure extraction pipeline.
///
/// Tries extraction strategies in priority order, returning the first
/// non-empty result. This cascade handles the 6 different skill structure
/// patterns found in real-world skills.
fn extract_procedures_multi_strategy(body: &MarkdownBody) -> (Vec<RawProcedureStep>, Vec<RawApproach>) {
    // Strategy 0: List-based procedures already extracted during parsing
    if !body.procedures.is_empty() {
        return (body.procedures.clone(), vec![]);
    }

    // Strategy 1: Explicit heading steps (existing extract_heading_steps)
    let steps = extract_heading_steps(&body.sections);
    if !steps.is_empty() {
        return (steps, vec![]);
    }

    // Strategy 2: Workflow extraction
    let steps = extract_workflow_steps(&body.sections);
    if !steps.is_empty() {
        return (steps, vec![]);
    }

    // Strategy 3: Section-as-phase
    let steps = extract_phase_sections(&body.sections);
    if !steps.is_empty() {
        return (steps, vec![]);
    }

    // Strategy 4: Mode-as-approach
    let approaches = extract_mode_approaches(&body.sections);
    if !approaches.is_empty() {
        let step = RawProcedureStep {
            order: 1,
            text: "Select the appropriate approach based on user needs".to_string(),
            is_critical: false,
            body: String::new(),
        };
        return (vec![step], approaches);
    }

    // Strategy 5: Reference-as-operation
    let steps = extract_reference_operations(&body.sections);
    if !steps.is_empty() {
        return (steps, vec![]);
    }

    // Strategy 6: Full-body fallback
    let full_body = body.sections.iter()
        .map(|s| s.content.clone())
        .collect::<Vec<_>>()
        .join("\n");
    let step = extract_full_body_step(&full_body);
    (vec![step], vec![])
}

/// Parse Markdown body content
///
/// # Arguments
///
/// * `body` - The Markdown body text (after frontmatter)
///
/// # Returns
///
/// A `MarkdownBody` containing parsed sections, procedures, approaches, examples, and code blocks.
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
    let mut current_heading_level: u8 = 0;
    let mut procedure_counter = 0u32;
    let mut current_code_block = String::new();
    let mut current_code_lang: Option<String> = None;

    for event in parser {
        match event {
            Event::Start(Tag::Heading { level, .. }) => {
                // Save previous section with its heading level
                if !current_section_title.is_empty() {
                    result.sections.push(Section {
                        level: current_heading_level,
                        title: current_section_title.clone(),
                        content: current_content.trim().to_string(),
                        kind: classify_section(&current_section_title),
                    });
                }
                current_section_title.clear();
                current_content.clear();

                current_heading_level = match level {
                    HeadingLevel::H1 => 1,
                    HeadingLevel::H2 => 2,
                    HeadingLevel::H3 => 3,
                    HeadingLevel::H4 => 4,
                    HeadingLevel::H5 => 5,
                    HeadingLevel::H6 => 6,
                };
                state = ParseState::InHeading(current_heading_level);
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
                // Ordered list — trigger InProcedureList for Procedure-kind sections.
                // Uses classify_section() instead of deprecated is_procedure_section().
                if classify_section(&current_section_title) == SectionKind::Procedure
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

            Event::SoftBreak => {
                current_content.push('\n');
            }

            Event::HardBreak => {
                current_content.push('\n');
            }

            Event::Start(Tag::Paragraph) => {
                // Add paragraph separator when not in heading/code/list/blockquote state
                if state == ParseState::Default && !current_content.is_empty() {
                    current_content.push_str("\n\n");
                }
            }

            _ => {}
        }
    }

    // Save last section
    if !current_section_title.is_empty() {
        let kind = classify_section(&current_section_title);
        result.sections.push(Section {
            level: current_heading_level,
            title: current_section_title,
            content: current_content.trim().to_string(),
            kind,
        });
    }

    // Multi-strategy extraction replaces the previous single strategy
    let (procedures, approaches) = extract_procedures_multi_strategy(&result);
    result.procedures = procedures;
    result.approaches = approaches;

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    // ===== classify_section tests =====

    #[test]
    fn test_classify_procedure() {
        assert_eq!(classify_section("Procedures"), SectionKind::Procedure);
        assert_eq!(classify_section("Core Workflow Pattern"), SectionKind::Procedure);
        assert_eq!(classify_section("步骤"), SectionKind::Procedure);
        assert_eq!(classify_section("Editing Workflow"), SectionKind::Procedure);
        assert_eq!(classify_section("Instructions"), SectionKind::Procedure);
    }

    #[test]
    fn test_classify_mode_selector() {
        assert_eq!(classify_section("How to Use"), SectionKind::ModeSelector);
        assert_eq!(classify_section("Usage"), SectionKind::ModeSelector);
        assert_eq!(classify_section("Ways to Use This Skill"), SectionKind::ModeSelector);
        assert_eq!(classify_section("Using the Skill"), SectionKind::ModeSelector);
    }

    #[test]
    fn test_classify_context() {
        assert_eq!(classify_section("When to Use"), SectionKind::Context);
        assert_eq!(classify_section("Prerequisites"), SectionKind::Context);
        assert_eq!(classify_section("Overview"), SectionKind::Context);
        assert_eq!(classify_section("Setup"), SectionKind::Context);
    }

    #[test]
    fn test_classify_reference() {
        assert_eq!(classify_section("Common Tasks"), SectionKind::Reference);
        assert_eq!(classify_section("Quick Start"), SectionKind::Reference);
        assert_eq!(classify_section("Quick Reference"), SectionKind::Reference);
        assert_eq!(classify_section("Libraries"), SectionKind::Reference);
    }

    #[test]
    fn test_classify_guideline() {
        assert_eq!(classify_section("Tips"), SectionKind::Guideline);
        assert_eq!(classify_section("Best Practices"), SectionKind::Guideline);
        assert_eq!(classify_section("Known Pitfalls"), SectionKind::Guideline);
        assert_eq!(classify_section("Guidelines"), SectionKind::Guideline);
    }

    #[test]
    fn test_classify_example() {
        assert_eq!(classify_section("Example"), SectionKind::Example);
        assert_eq!(classify_section("Examples"), SectionKind::Example);
    }

    #[test]
    fn test_classify_example_workflow_is_mode_selector() {
        // "example workflow" should be ModeSelector, not Example or Procedure
        assert_eq!(classify_section("Example Workflow"), SectionKind::ModeSelector);
    }

    #[test]
    fn test_classify_metadata() {
        assert_eq!(classify_section("Dependencies"), SectionKind::Metadata);
        assert_eq!(classify_section("See Also"), SectionKind::Metadata);
        assert_eq!(classify_section("Resources"), SectionKind::Metadata);
    }

    #[test]
    fn test_classify_unknown() {
        assert_eq!(classify_section("Custom Section"), SectionKind::Unknown);
        assert_eq!(classify_section("Random Title"), SectionKind::Unknown);
    }

    // ===== Existing tests (preserved) =====

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

    // ===== New strategy tests =====

    #[test]
    fn test_extract_workflow_steps() {
        let body = r#"## Core Workflow Pattern

### Workflow 1: Discover Available Tools

Search for tools matching your use case.

### Workflow 2: Check Connection

Verify the connection status.

### Workflow 3: Execute Tools

Run the selected tools.
"#;

        let result = parse_markdown_body(body);
        // Strategy 1 (heading_steps) would not match "Workflow N:" patterns,
        // so Strategy 2 (workflow_steps) should be used.
        assert!(!result.procedures.is_empty(), "Should extract workflow steps");
        // Workflow sub-headings should become procedure steps with extracted titles
        let has_discover = result.procedures.iter().any(|p| p.text.contains("Discover"));
        assert!(has_discover, "Should find 'Discover' in workflow step text");
        let has_connection = result.procedures.iter().any(|p| p.text.contains("Check Connection"));
        assert!(has_connection, "Should find 'Check Connection' in workflow step text");
    }

    #[test]
    fn test_extract_phase_sections() {
        let body = r#"## Creating the Document

First, create the document structure.

## Editing the Document

Then, modify the content.

## Final Review

Finally, review the output.
"#;

        let result = parse_markdown_body(body);
        // Should extract phase sections via Strategy 3
        assert!(!result.procedures.is_empty(), "Should extract phase sections");
        assert!(result.procedures.iter().any(|p| p.text.contains("Creating")), "Should find Creating phase");
        assert!(result.procedures.iter().any(|p| p.text.contains("Editing")), "Should find Editing phase");
        assert!(result.procedures.iter().any(|p| p.text.contains("Final Review")), "Should find Final Review phase");
    }

    #[test]
    fn test_extract_phase_sections_excludes_context() {
        let body = r#"## Overview

This is an overview.

## Creating the Document

First, create the document structure.
"#;

        let result = parse_markdown_body(body);
        // "Overview" (Context) should be excluded from phase extraction
        assert!(result.procedures.iter().all(|p| !p.text.contains("Overview")),
            "Overview should not be a phase step");
        assert!(result.procedures.iter().any(|p| p.text.contains("Creating")),
            "Creating should be a phase step");
    }

    #[test]
    fn test_extract_mode_approaches() {
        let body = r#"## How to Use

### Basic Extraction

For simple extraction tasks, follow this approach.

This is the basic way to extract data.

### Advanced Analysis

For complex analysis, use this approach.

This provides deeper insights.
"#;

        let result = parse_markdown_body(body);
        // Should extract approaches via Strategy 4
        assert!(!result.approaches.is_empty(), "Should extract approaches");
        assert!(result.approaches.iter().any(|a| a.name.contains("Basic")), "Should find Basic approach");
        assert!(result.approaches.iter().any(|a| a.name.contains("Advanced")), "Should find Advanced approach");
        // Should also have a selector procedure step
        assert!(!result.procedures.is_empty(), "Should have selector procedure step");
        assert_eq!(result.procedures[0].text, "Select the appropriate approach based on user needs");
    }

    #[test]
    fn test_extract_reference_operations() {
        let body = r#"## Common Tasks

### Extract Data

Use the extract command to pull data.

```
nsc extract --source input.md
```

### Merge Files

Combine multiple files together.

```
nsc merge --files a.md b.md
```
"#;

        let result = parse_markdown_body(body);
        // Should extract operations via Strategy 5
        assert!(!result.procedures.is_empty(), "Should extract reference operations");
        assert!(result.procedures.iter().any(|p| p.text.contains("Extract")), "Should find Extract operation");
        assert!(result.procedures.iter().any(|p| p.text.contains("Merge")), "Should find Merge operation");
    }

    #[test]
    fn test_full_body_fallback() {
        let body = r#"## Some Custom Section

This is a skill with no recognizable structure.
Just some plain text instructions.

## Another Section

More content here.
"#;

        let result = parse_markdown_body(body);
        // Should fall back to Strategy 6
        assert!(!result.procedures.is_empty(), "Should have fallback step");
        assert_eq!(result.procedures[0].text, "Follow the skill instructions");
        assert!(!result.procedures[0].body.is_empty(), "Fallback body should contain content");
    }

    #[test]
    fn test_section_kind_field_populated() {
        let body = r#"## Procedures

1. Step one.

## Overview

Some overview text.

## Tips

Some tips.
"#;

        let result = parse_markdown_body(body);
        let proc_section = result.sections.iter().find(|s| s.title == "Procedures");
        assert!(proc_section.is_some());
        assert_eq!(proc_section.unwrap().kind, SectionKind::Procedure);

        let overview_section = result.sections.iter().find(|s| s.title == "Overview");
        assert!(overview_section.is_some());
        assert_eq!(overview_section.unwrap().kind, SectionKind::Context);

        let tips_section = result.sections.iter().find(|s| s.title == "Tips");
        assert!(tips_section.is_some());
        assert_eq!(tips_section.unwrap().kind, SectionKind::Guideline);
    }

    #[test]
    fn test_section_levels_correct() {
        let body = r#"## Procedures

### Step 1: Do Something

Content here.

## Overview

Some overview.
"#;

        let result = parse_markdown_body(body);
        let proc_section = result.sections.iter().find(|s| s.title == "Procedures");
        assert!(proc_section.is_some());
        assert_eq!(proc_section.unwrap().level, 2, "Procedures should be level 2");

        let step_section = result.sections.iter().find(|s| s.title == "Step 1: Do Something");
        assert!(step_section.is_some());
        assert_eq!(step_section.unwrap().level, 3, "Step sub-heading should be level 3");
    }

    #[test]
    fn test_multi_strategy_prefers_list_over_heading() {
        // When a Procedure section has both an ordered list and heading steps,
        // the list-based extraction (Strategy 0) should win.
        let body = r#"## Procedures

1. First list step.
2. Second list step.

### Step 1: Heading Step

Some content.
"#;

        let result = parse_markdown_body(body);
        // Strategy 0 (list-based) should produce 2 steps
        assert_eq!(result.procedures.len(), 2, "Should extract 2 list-based steps");
        assert_eq!(result.procedures[0].text, "First list step.");
        assert_eq!(result.procedures[1].text, "Second list step.");
    }

    #[test]
    fn test_approach_description_extraction() {
        let body = r#"## How to Use

### Basic Mode

This is the basic mode description.

More details here.

### Advanced Mode

Advanced mode for complex tasks.
"#;

        let result = parse_markdown_body(body);
        assert_eq!(result.approaches.len(), 2);
        let basic = result.approaches.iter().find(|a| a.name == "Basic Mode").unwrap();
        assert_eq!(basic.description, "This is the basic mode description.");
        assert!(basic.instructions.contains("More details here"));
    }
}