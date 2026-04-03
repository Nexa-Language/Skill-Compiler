//! Backend Module
//!
//! This module provides code emission for different target platforms.

mod claude;
mod codex;
mod emitter;
mod gemini;

pub use claude::ClaudeEmitter;
pub use codex::CodexEmitter;
pub use emitter::Emitter;
pub use gemini::GeminiEmitter;

/// Target platform enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TargetPlatform {
    /// Claude (Anthropic)
    Claude,
    /// Codex (OpenAI)
    Codex,
    /// Gemini (Google)
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
    #[must_use]
    pub const fn extension(&self) -> &'static str {
        match self {
            Self::Claude => ".xml",
            Self::Codex => "_schema.json",
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
}
