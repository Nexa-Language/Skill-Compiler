//! Diagnostic System
//!
//! Provides structured diagnostic information for error reporting.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Error level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ErrorLevel {
    /// Error - blocks compilation
    Error,
    /// Warning - doesn't block but should be fixed
    Warning,
    /// Advice - suggestion for improvement
    Advice,
}

impl ErrorLevel {
    /// Check if this level blocks compilation
    #[must_use]
    pub const fn is_blocking(&self) -> bool {
        matches!(self, Self::Error)
    }

    /// Get miette severity
    #[must_use]
    pub const fn to_severity(&self) -> miette::Severity {
        match self {
            Self::Error => miette::Severity::Error,
            Self::Warning => miette::Severity::Warning,
            Self::Advice => miette::Severity::Advice,
        }
    }

    /// Get ANSI color code
    #[must_use]
    pub const fn ansi_color(&self) -> &'static str {
        match self {
            Self::Error => "\x1b[31m",   // Red
            Self::Warning => "\x1b[33m", // Yellow
            Self::Advice => "\x1b[36m",  // Cyan
        }
    }

    /// Get icon
    #[must_use]
    pub const fn icon(&self) -> &'static str {
        match self {
            Self::Error => "❌",
            Self::Warning => "⚠️",
            Self::Advice => "💡",
        }
    }
}

/// Diagnostic information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Diagnostic {
    /// Error message
    pub message: String,
    /// Error code
    pub code: String,
    /// Error level
    pub level: ErrorLevel,
    /// Fix suggestion
    #[serde(skip_serializing_if = "Option::is_none")]
    pub help: Option<String>,
    /// Source file path
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_path: Option<String>,
    /// Line number (1-based)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<usize>,
    /// Column number (1-based)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub column: Option<usize>,
    /// Documentation URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

impl Diagnostic {
    /// Create an error-level diagnostic
    #[must_use]
    pub fn error(message: impl Into<String>, code: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            code: code.into(),
            level: ErrorLevel::Error,
            help: None,
            file_path: None,
            line: None,
            column: None,
            url: None,
        }
    }

    /// Create a warning-level diagnostic
    #[must_use]
    pub fn warning(message: impl Into<String>, code: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            code: code.into(),
            level: ErrorLevel::Warning,
            help: None,
            file_path: None,
            line: None,
            column: None,
            url: None,
        }
    }

    /// Create an advice-level diagnostic
    #[must_use]
    pub fn advice(message: impl Into<String>, code: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            code: code.into(),
            level: ErrorLevel::Advice,
            help: None,
            file_path: None,
            line: None,
            column: None,
            url: None,
        }
    }

    /// Create a diagnostic with a specified error level
    ///
    /// Use this when the level needs to be determined dynamically
    /// (e.g., strict mode vs warning mode).
    #[must_use]
    pub fn new(message: impl Into<String>, code: impl Into<String>, level: ErrorLevel) -> Self {
        Self {
            message: message.into(),
            code: code.into(),
            level,
            help: None,
            file_path: None,
            line: None,
            column: None,
            url: None,
        }
    }

    /// Add help text
    #[must_use]
    pub fn with_help(mut self, help: impl Into<String>) -> Self {
        self.help = Some(help.into());
        self
    }

    /// Add source location
    #[must_use]
    pub fn with_location(
        mut self,
        file_path: impl Into<String>,
        line: usize,
        column: usize,
    ) -> Self {
        self.file_path = Some(file_path.into());
        self.line = Some(line);
        self.column = Some(column);
        self
    }

    /// Add documentation URL
    #[must_use]
    pub fn with_url(mut self, url: impl Into<String>) -> Self {
        self.url = Some(url.into());
        self
    }

    /// Check if this is a blocking error
    #[must_use]
    pub fn is_blocking(&self) -> bool {
        self.level.is_blocking()
    }

    /// Check if this is an error
    #[must_use]
    pub fn is_error(&self) -> bool {
        self.level == ErrorLevel::Error
    }

    /// Check if this is a warning
    #[must_use]
    pub fn is_warning(&self) -> bool {
        self.level == ErrorLevel::Warning
    }
}

/// Diagnostic collector
#[derive(Debug, Clone, Default)]
pub struct DiagnosticCollector {
    /// All diagnostics
    diagnostics: Vec<Diagnostic>,
    /// Diagnostics grouped by file
    by_file: HashMap<String, Vec<usize>>,
    /// Diagnostics grouped by code
    by_code: HashMap<String, Vec<usize>>,
}

impl DiagnosticCollector {
    /// Create a new collector
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a diagnostic
    pub fn add(&mut self, diagnostic: Diagnostic) {
        let index = self.diagnostics.len();
        self.diagnostics.push(diagnostic.clone());

        if let Some(ref file) = diagnostic.file_path {
            self.by_file.entry(file.clone()).or_default().push(index);
        }

        self.by_code
            .entry(diagnostic.code.clone())
            .or_default()
            .push(index);
    }

    /// Get all diagnostics
    #[must_use]
    pub fn all(&self) -> &[Diagnostic] {
        &self.diagnostics
    }

    /// Get all errors
    #[must_use]
    pub fn errors(&self) -> Vec<&Diagnostic> {
        self.diagnostics.iter().filter(|d| d.is_error()).collect()
    }

    /// Get all warnings
    #[must_use]
    pub fn warnings(&self) -> Vec<&Diagnostic> {
        self.diagnostics.iter().filter(|d| d.is_warning()).collect()
    }

    /// Get diagnostics for a file
    #[must_use]
    pub fn for_file(&self, file: &str) -> Vec<&Diagnostic> {
        self.by_file
            .get(file)
            .map(|indices| indices.iter().map(|i| &self.diagnostics[*i]).collect())
            .unwrap_or_default()
    }

    /// Check if there are blocking errors
    #[must_use]
    pub fn has_blocking_errors(&self) -> bool {
        self.diagnostics.iter().any(|d| d.is_blocking())
    }

    /// Get summary
    #[must_use]
    pub fn summary(&self) -> DiagnosticSummary {
        DiagnosticSummary {
            total: self.diagnostics.len(),
            errors: self.errors().len(),
            warnings: self.warnings().len(),
            files_affected: self.by_file.len(),
            unique_codes: self.by_code.len(),
        }
    }
}

/// Diagnostic summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticSummary {
    /// Total number of diagnostics
    pub total: usize,
    /// Number of errors
    pub errors: usize,
    /// Number of warnings
    pub warnings: usize,
    /// Number of affected files
    pub files_affected: usize,
    /// Number of unique error codes
    pub unique_codes: usize,
}
