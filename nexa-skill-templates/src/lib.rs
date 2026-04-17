//! Nexa Skill Templates
//!
//! Askama template context structs for platform-specific output generation.
//! Each context struct uses simple types (String, bool, Vec) — no SkillIR references.
//! All complex transformations happen in the Emitter layer; context structs are flat DTOs.

use askama::Template;

/// Procedure step context (shared by all platforms)
#[derive(Debug, Clone)]
pub struct StepContext {
    /// Step order number (1-based)
    pub order: u32,
    /// Step instruction text
    pub instruction: String,
    /// Whether this is a critical step requiring HITL approval
    pub is_critical: bool,
}

/// Constraint context for anti-skill patterns
#[derive(Debug, Clone)]
pub struct ConstraintContext {
    /// Constraint source identifier (pattern ID)
    pub source: String,
    /// Constraint content description
    pub content: String,
    /// Platform-specific marker, e.g. " 🔴 [BLOCK]", " [BLOCK - Requires HITL]", or empty
    pub level_marker: String,
}

/// Permission context (for all platform templates)
#[derive(Debug, Clone)]
pub struct PermissionContext {
    /// Permission kind display name (e.g. "Database", "FileSystem")
    pub kind_name: String,
    /// Permission scope string (e.g. "postgres:staging:ALTER")
    pub scope: String,
    /// Whether this permission is read-only
    pub read_only: bool,
    /// Permission description (human-readable reason)
    pub description: String,
}

/// Approach context for mode-selector skills
#[derive(Debug, Clone)]
pub struct ApproachContext {
    /// Approach name (e.g. "Basic Extraction", "Advanced Analysis")
    pub name: String,
    /// One-line description
    pub description: String,
    /// Full instructions for this approach
    pub instructions: String,
}

/// Example context for few-shot demonstrations
#[derive(Debug, Clone)]
pub struct ExampleContext {
    /// Example title; empty string means no title
    pub title: String,
    /// User input text
    pub user_input: String,
    /// Agent response text
    pub agent_response: String,
}

/// Section context for extra Markdown sections not captured by specific fields
#[derive(Debug, Clone)]
pub struct SectionContext {
    /// Section heading level (1-6)
    pub level: u8,
    /// Section title (e.g. "Setup", "Known Pitfalls")
    pub title: String,
    /// Section content in original Markdown format
    pub content: String,
}

// ===== Claude Markdown+XML Template =====

/// Claude SKILL.md template context — produces YAML frontmatter + Markdown header + `<agent_skill>` XML body
#[derive(Debug, Clone, Template)]
#[template(path = "claude_md.j2", escape = "none")]
pub struct ClaudeContext {
    /// Skill name
    pub name: String,
    /// Skill version
    pub version: String,
    /// Skill description / intent
    pub description: String,
    /// Whether HITL (human-in-the-loop) is required
    pub hitl_required: bool,
    /// Procedure steps
    pub procedures: Vec<StepContext>,
    /// Anti-skill constraints
    pub anti_skill_constraints: Vec<ConstraintContext>,
    /// Pre-execution conditions
    pub pre_conditions: Vec<String>,
    /// Post-execution conditions
    pub post_conditions: Vec<String>,
    /// Error recovery strategies (fallbacks)
    pub fallbacks: Vec<String>,
    /// Context gathering steps
    pub context_gathering: Vec<String>,
    /// Few-shot examples
    pub examples: Vec<ExampleContext>,
    /// Permission declarations
    pub permissions: Vec<PermissionContext>,
    /// MCP server dependencies
    pub mcp_servers: Vec<String>,
    /// Security level in lowercase (e.g. "medium")
    pub security_level: String,
    /// Additional sections from the Markdown body
    pub extra_sections: Vec<SectionContext>,
    /// Alternative execution approaches (for mode-selector skills)
    pub approaches: Vec<ApproachContext>,
    /// Skill execution mode (e.g. "sequential", "alternative", "toolkit", "guideline")
    pub skill_mode: String,
}

// ===== Codex Markdown Template =====

/// Codex Markdown template context — produces pure Markdown for GPT
#[derive(Debug, Clone, Template)]
#[template(path = "codex_md.j2", escape = "none")]
pub struct CodexContext {
    /// Skill name
    pub name: String,
    /// Skill version
    pub version: String,
    /// Clean description (truncated to 1024 chars, XML tags stripped)
    pub clean_description: String,
    /// Full description (no truncation)
    pub description: String,
    /// Whether HITL is required
    pub hitl_required: bool,
    /// Security level in lowercase (e.g. "medium")
    pub security_level: String,
    /// Security level in uppercase (e.g. "MEDIUM")
    pub security_level_upper: String,
    /// Security instruction text matching the level
    pub security_instruction: String,
    /// MCP server dependencies
    pub mcp_servers: Vec<String>,
    /// Whether input_schema exists
    pub has_input_schema: bool,
    /// Whether output_schema exists
    pub has_output_schema: bool,
    /// Procedure steps
    pub procedures: Vec<StepContext>,
    /// Anti-skill constraints
    pub anti_skill_constraints: Vec<ConstraintContext>,
    /// Context gathering items (pre-filled with defaults when empty)
    pub context_gathering: Vec<String>,
    /// Few-shot examples
    pub examples: Vec<ExampleContext>,
    /// Fallback strategies
    pub fallbacks: Vec<String>,
    /// Pre-execution conditions
    pub pre_conditions: Vec<String>,
    /// Post-execution conditions
    pub post_conditions: Vec<String>,
    /// Permission declarations
    pub permissions: Vec<PermissionContext>,
    /// Additional sections from the Markdown body
    pub extra_sections: Vec<SectionContext>,
    /// Alternative execution approaches (for mode-selector skills)
    pub approaches: Vec<ApproachContext>,
    /// Skill execution mode (e.g. "sequential", "alternative", "toolkit", "guideline")
    pub skill_mode: String,
}

// ===== Gemini Markdown Template =====

/// Gemini Markdown template context — produces Markdown + YAML assets
#[derive(Debug, Clone, Template)]
#[template(path = "gemini_md.j2", escape = "none")]
pub struct GeminiContext {
    /// Skill name
    pub name: String,
    /// Skill version
    pub version: String,
    /// Clean description (truncated to 1024 chars, XML tags stripped)
    pub clean_description: String,
    /// Full description (no truncation)
    pub description: String,
    /// Whether HITL is required
    pub hitl_required: bool,
    /// Security level in lowercase (e.g. "medium")
    pub security_level: String,
    /// Security level display name (e.g. "CRITICAL", "HIGH")
    pub security_level_display: String,
    /// Security instruction text matching the level
    pub security_instruction: String,
    /// MCP server dependencies
    pub mcp_servers: Vec<String>,
    /// Whether input_schema exists (for asset reference)
    pub has_input_schema: bool,
    /// Procedure steps
    pub procedures: Vec<StepContext>,
    /// Anti-skill constraints
    pub anti_skill_constraints: Vec<ConstraintContext>,
    /// Context gathering steps (replace hardcoded defaults)
    pub context_gathering: Vec<String>,
    /// Pre-execution conditions
    pub pre_conditions: Vec<String>,
    /// Post-execution conditions
    pub post_conditions: Vec<String>,
    /// Permission declarations
    pub permissions: Vec<PermissionContext>,
    /// Few-shot examples
    pub examples: Vec<ExampleContext>,
    /// Error recovery strategies (fallbacks)
    pub fallbacks: Vec<String>,
    /// Additional sections from the Markdown body
    pub extra_sections: Vec<SectionContext>,
    /// Alternative execution approaches (for mode-selector skills)
    pub approaches: Vec<ApproachContext>,
    /// Skill execution mode (e.g. "sequential", "alternative", "toolkit", "guideline")
    pub skill_mode: String,
}

// ===== Kimi Full Markdown Template =====

/// Kimi Markdown template context — full Markdown with maximum context preservation
#[derive(Debug, Clone, Template)]
#[template(path = "kimi_md.j2", escape = "none")]
pub struct KimiContext {
    /// Skill name
    pub name: String,
    /// Skill version
    pub version: String,
    /// Security level display string
    pub security_level: String,
    /// Full description (no truncation, no XML stripping)
    pub description: String,
    /// Whether HITL is required
    pub hitl_required: bool,
    /// MCP server dependencies
    pub mcp_servers: Vec<String>,
    /// Permission declarations
    pub permissions: Vec<PermissionContext>,
    /// Pre-execution conditions
    pub pre_conditions: Vec<String>,
    /// Context gathering steps
    pub context_gathering: Vec<String>,
    /// Input schema as pretty-printed JSON; empty string means no schema
    pub input_schema_json: String,
    /// Procedure steps
    pub procedures: Vec<StepContext>,
    /// Anti-skill constraints
    pub anti_skill_constraints: Vec<ConstraintContext>,
    /// Fallback strategies
    pub fallbacks: Vec<String>,
    /// Post-execution conditions
    pub post_conditions: Vec<String>,
    /// Output schema as pretty-printed JSON; empty string means no schema
    pub output_schema_json: String,
    /// Few-shot examples
    pub examples: Vec<ExampleContext>,
    /// Additional sections from the Markdown body
    pub extra_sections: Vec<SectionContext>,
    /// Alternative execution approaches (for mode-selector skills)
    pub approaches: Vec<ApproachContext>,
    /// Skill execution mode (e.g. "sequential", "alternative", "toolkit", "guideline")
    pub skill_mode: String,
}