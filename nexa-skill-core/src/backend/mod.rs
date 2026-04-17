//! Backend Module
//!
//! This module provides code emission for different target platforms.
//!
//! Architecture based on empirical research ("高级提示词工程格式与智能体技能架构"):
//! - Claude: XML tags for semantic layering (23% higher accuracy in math reasoning)
//! - Codex: Decoupled Reasoning and Formatting (avoid JSON format tax)
//! - Gemini: Markdown + YAML for nested data (51.9% accuracy vs JSON 43.1%)

mod claude;
mod codex;
mod emitter;
mod gemini;
mod kimi;
mod registry;
pub mod routing_manifest;

pub use claude::ClaudeEmitter;
pub use codex::CodexEmitter;
pub use emitter::Emitter;
pub use gemini::GeminiEmitter;
pub use kimi::KimiEmitter;
pub use registry::EmitterRegistry;
pub use routing_manifest::{RoutingManifest, RoutingEntry, MinimalEntry};

/// Target platform enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TargetPlatform {
    /// Claude (Anthropic) - YAML frontmatter + Markdown/XML hybrid SKILL.md format
    Claude,
    /// Codex (OpenAI) - Markdown input, JSON Schema output (Decoupled)
    Codex,
    /// Gemini (Google) - Markdown + YAML for nested data
    Gemini,
    /// Kimi (Moonshot)
    Kimi,
}

impl TargetPlatform {
    /// Get the platform slug (for CLI flags)
    #[must_use]
    pub const fn slug(&self) -> &'static str {
        match self {
            Self::Claude => "claude",
            Self::Codex => "codex",
            Self::Gemini => "gemini",
            Self::Kimi => "kimi",
        }
    }

    /// Get the output file extension
    /// Note: Codex now outputs .md (Markdown) with separate schema files
    #[must_use]
    pub const fn extension(&self) -> &'static str {
        match self {
            Self::Claude => ".md",  // SKILL.md format (YAML frontmatter + Markdown/XML hybrid)
            Self::Codex => ".md",  // Changed from _schema.json to .md (dual-payload)
            Self::Gemini => ".md",
            Self::Kimi => ".md",
        }
    }

    /// Get the display name
    #[must_use]
    pub const fn display_name(&self) -> &'static str {
        match self {
            Self::Claude => "Claude Code",
            Self::Codex => "OpenAI Codex",
            Self::Gemini => "Gemini CLI",
            Self::Kimi => "Kimi CLI",
        }
    }

    /// Get format optimization description
    #[must_use]
    pub const fn format_optimization(&self) -> &'static str {
        match self {
            Self::Claude => "XML tags for semantic layering",
            Self::Codex => "Decoupled Reasoning and Formatting (Markdown + JSON Schema)",
            Self::Gemini => "Markdown + YAML for nested data optimization",
            Self::Kimi => "Standard Markdown",
        }
    }
}
