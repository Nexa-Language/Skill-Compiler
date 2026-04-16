//! Error Handling Module
//!
//! This module provides unified error types and diagnostics for the Nexa Skill Compiler.
//! It uses `thiserror` for error derivation and `miette` for beautiful error reporting.
//!
//! # Error Hierarchy
//!
//! - [`CompileError`] - Top-level error type that wraps all other errors
//! - [`ParseError`] - Frontend parsing errors
//! - [`IRError`] - IR construction errors
//! - [`AnalysisError`] - Semantic analysis errors
//! - [`EmitError`] - Backend emission errors
//!
//! # Usage
//!
//! ```ignore
//! use nexa_skill_core::error::{CompileError, ParseError};
//!
//! fn parse_file(content: &str) -> Result<Skill, CompileError> {
//!     let ast = parse_frontmatter(content)?;
//!     Ok(build_skill(ast)?)
//! }
//! ```

mod diagnostic;

pub use diagnostic::{Diagnostic, DiagnosticCollector, DiagnosticSummary, ErrorLevel};

/// Compile error type
///
/// This is the top-level error type that wraps all other error types.
/// It implements `miette::Diagnostic` for beautiful error reporting.
#[derive(Debug, Clone, thiserror::Error, miette::Diagnostic)]
pub enum CompileError {
    /// Feature not yet implemented
    #[error("Feature not implemented: {0}")]
    #[diagnostic(code(nsc::not_implemented))]
    NotImplemented(String),

    /// Parse error from frontend
    #[error("Parse error: {0}")]
    #[diagnostic(code(nsc::parse_error))]
    ParseError(#[from] ParseError),

    /// IR construction error
    #[error("IR error: {0}")]
    #[diagnostic(code(nsc::ir_error))]
    IRError(#[from] IRError),

    /// Analysis error
    #[error("Analysis error: {0}")]
    #[diagnostic(code(nsc::analysis_error))]
    AnalysisError(#[from] AnalysisError),

    /// Emit error from backend
    #[error("Emit error: {0}")]
    #[diagnostic(code(nsc::emit_error))]
    EmitError(#[from] EmitError),

    /// IO error
    #[error("IO error: {0}")]
    #[diagnostic(code(nsc::io_error))]
    IOError(String),
}

/// Parse error types
///
/// Errors that occur during the frontend parsing phase.
#[derive(Debug, Clone, thiserror::Error, miette::Diagnostic)]
pub enum ParseError {
    /// Missing YAML frontmatter section
    #[error("Missing YAML frontmatter")]
    #[diagnostic(
        code(nsc::parse::missing_frontmatter),
        help("Add YAML frontmatter at the beginning of the file, enclosed by ---")
    )]
    MissingFrontmatter,

    /// Empty frontmatter section
    #[error("Empty frontmatter section")]
    #[diagnostic(
        code(nsc::parse::empty_frontmatter),
        help("Add required fields (name, description) to the frontmatter")
    )]
    EmptyFrontmatter,

    /// YAML syntax error
    #[error("YAML parse error: {0}")]
    #[diagnostic(
        code(nsc::parse::yaml_error),
        help("Check YAML syntax: indentation, quotes, and special characters")
    )]
    YamlError(String),

    /// XML tags detected in description field
    #[error("XML tags detected in description: {0}")]
    #[diagnostic(
        code(nsc::parse::xml_tags_in_description),
        help("Remove XML tags from description — they interfere with LLM parsing. Use plain text instead.")
    )]
    XmlTagsInDescription(String),

    /// Invalid name format (not kebab-case)
    #[error("Invalid name format: '{0}'")]
    #[diagnostic(
        code(nsc::parse::invalid_name),
        help("Name must be kebab-case: lowercase letters, numbers, hyphens. 1-64 chars. No leading/trailing/double hyphens.")
    )]
    InvalidNameFormat(String),

    /// File read error
    #[error("Failed to read file '{0}': {1}")]
    #[diagnostic(
        code(nsc::parse::file_read_error),
        help("Check if the file exists and you have read permissions")
    )]
    FileReadError(String, String),

    /// Missing required section in body
    #[error("Missing required section: {0}")]
    #[diagnostic(
        code(nsc::parse::missing_section),
        help("Add a '{0}' section to your SKILL.md")
    )]
    MissingSection(String),
}

/// IR construction error types
///
/// Errors that occur during IR construction phase.
#[derive(Debug, Clone, thiserror::Error, miette::Diagnostic)]
pub enum IRError {
    /// Missing required field in frontmatter
    #[error("Missing required field: {0}")]
    #[diagnostic(
        code(nsc::ir::missing_field),
        help("Add the '{0}' field to your frontmatter")
    )]
    MissingRequiredField(&'static str),

    /// Invalid name format (not kebab-case)
    #[error("Invalid name format: '{0}'")]
    #[diagnostic(
        code(nsc::ir::invalid_name),
        help("Name must be kebab-case: lowercase letters, numbers, hyphens. 1-64 chars.")
    )]
    InvalidNameFormat(String),

    /// Description exceeds maximum length
    #[error("Description too long: {0} characters (max 1024)")]
    #[diagnostic(code(nsc::ir::description_too_long))]
    DescriptionTooLong(usize),

    /// Security level inconsistency
    #[error("Inconsistent security level")]
    #[diagnostic(code(nsc::ir::inconsistent_security))]
    InconsistentSecurityLevel,
}

/// Analysis error types
///
/// Errors that occur during semantic analysis phase.
#[derive(Debug, Clone, thiserror::Error, miette::Diagnostic)]
pub enum AnalysisError {
    /// JSON Schema validation failed
    #[error("Schema validation failed: {0}")]
    #[diagnostic(code(nsc::analysis::schema_error))]
    SchemaValidationFailed(String),

    /// MCP server not in allowlist
    #[error("MCP server not allowed: {0}")]
    #[diagnostic(
        code(nsc::analysis::mcp_not_allowed),
        help("Add the MCP server to the allowlist or remove it from dependencies")
    )]
    MCPNotAllowed(String),

    /// Missing permission declaration
    #[error("Missing permission for operation: {0}")]
    #[diagnostic(
        code(nsc::analysis::missing_permission),
        help("Declare the required permission in the frontmatter")
    )]
    MissingPermission(String),

    /// Security level mismatch
    #[error("Security level mismatch: {0}")]
    #[diagnostic(code(nsc::analysis::security_mismatch))]
    SecurityLevelMismatch(String),
}

/// Emit error types
///
/// Errors that occur during backend emission phase.
#[derive(Debug, Clone, thiserror::Error, miette::Diagnostic)]
pub enum EmitError {
    /// Unsupported target platform
    #[error("Unsupported target platform: {0}")]
    #[diagnostic(code(nsc::emit::unsupported_target))]
    UnsupportedTarget(String),

    /// Template rendering error
    #[error("Template error: {0}")]
    #[diagnostic(code(nsc::emit::template_error))]
    TemplateError(String),

    /// Serialization error
    #[error("Serialization error: {0}")]
    #[diagnostic(code(nsc::emit::serialization_error))]
    SerializationError(String),

    /// File write error
    #[error("Write error: {0}")]
    #[diagnostic(code(nsc::emit::write_error))]
    WriteError(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_level_is_blocking() {
        assert!(ErrorLevel::Error.is_blocking());
        assert!(!ErrorLevel::Warning.is_blocking());
        assert!(!ErrorLevel::Advice.is_blocking());
    }

    #[test]
    fn test_diagnostic_creation() {
        let diag = Diagnostic::error("Test error", "E001")
            .with_help("Try this fix")
            .with_location("test.md", 10, 5);

        assert_eq!(diag.message, "Test error");
        assert_eq!(diag.code, "E001");
        assert_eq!(diag.level, ErrorLevel::Error);
        assert_eq!(diag.help, Some("Try this fix".to_string()));
        assert_eq!(diag.file_path, Some("test.md".to_string()));
        assert_eq!(diag.line, Some(10));
        assert_eq!(diag.column, Some(5));
    }

    #[test]
    fn test_parse_error_display() {
        let err = ParseError::MissingFrontmatter;
        assert!(err.to_string().contains("Missing YAML frontmatter"));
    }

    #[test]
    fn test_ir_error_display() {
        let err = IRError::InvalidNameFormat("InvalidName".to_string());
        assert!(err.to_string().contains("InvalidName"));
    }
}
