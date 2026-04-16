//! MCP Dependency Checker
//!
//! Validates MCP server dependencies against an allowlist and detects duplicates.
//!
//! # Modes
//!
//! - **Warning mode** (default, `strict=false`): Unknown servers produce Warning diagnostics.
//! - **Strict mode** (`strict=true`): Unknown servers produce Error diagnostics (blocking).
//!
//! # Rules
//!
//! 1. **Allowlist check**: Server not in allowlist → Warning/Error depending on mode.
//! 2. **Duplicate check**: Same server declared multiple times → Warning (always).

use std::collections::HashSet;
use std::sync::Arc;

use crate::error::{Diagnostic, ErrorLevel};
use crate::ir::SkillIR;

/// MCP dependency checker with allowlist and strict/warning dual-mode support.
pub struct MCPDependencyChecker {
    /// Allowed MCP servers (default built-in allowlist).
    /// Uses `Arc<str>` for zero-copy comparison with `SkillIR.mcp_servers`.
    allowlist: Vec<Arc<str>>,
    /// Strict mode: unknown servers produce Error instead of Warning.
    strict: bool,
}

/// Default built-in allowlist — common MCP servers referenced in documentation.
const DEFAULT_ALLOWLIST: &[&str] = &[
    // File system
    "filesystem-server",
    "filesystem",
    // GitHub
    "github-server",
    "github-pr-creator",
    "github",
    // Database
    "postgres-server",
    "neon-postgres-admin",
    "postgres",
    "database-server",
    "database",
    "mysql",
    "sqlite",
    // Network
    "network-server",
    "network",
    // Memory
    "memory-server",
    "memory",
    // Search
    "brave-search",
    "search",
    // Browser automation
    "puppeteer",
    "browser",
    // Communication
    "slack",
    "discord",
    "email",
    // AI/ML
    "openai",
    "anthropic",
    // Utilities
    "fetch",
    "http",
    "json",
];

impl MCPDependencyChecker {
    /// Create a new MCP dependency checker with default allowlist and warning mode.
    #[must_use]
    pub fn new() -> Self {
        Self {
            allowlist: DEFAULT_ALLOWLIST.iter().map(|s| Arc::from(*s)).collect(),
            strict: false,
        }
    }

    /// Create a checker with a custom allowlist (warning mode by default).
    #[must_use]
    pub fn with_allowlist(allowlist: Vec<Arc<str>>) -> Self {
        Self { allowlist, strict: false }
    }

    /// Set strict mode on the current checker.
    ///
    /// When `strict=true`, unknown MCP servers produce Error diagnostics
    /// (blocking compilation) instead of Warning.
    #[must_use]
    pub fn with_strict_mode(self, strict: bool) -> Self {
        Self { strict, ..self }
    }

    /// Check MCP server dependencies, returning all diagnostics.
    ///
    /// Unlike the previous `Result<(), AnalysisError>` API, this returns
    /// `Vec<Diagnostic>` to collect all issues without early termination,
    /// consistent with `SchemaValidator` and `PermissionAuditor`.
    pub fn check(&self, ir: &SkillIR) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        let mut seen_servers: HashSet<Arc<str>> = HashSet::new();

        for server in &ir.mcp_servers {
            // Rule 1: Allowlist check
            if !self.is_allowed(server) {
                let level = if self.strict {
                    ErrorLevel::Error
                } else {
                    ErrorLevel::Warning
                };
                let diag = Diagnostic::new(
                    format!("MCP server '{}' is not in the allowlist", server),
                    "nsc::ir::mcp_not_allowed",
                    level,
                ).with_help(format!(
                    "Add '{}' to your MCP allowlist configuration, or use one of: {}",
                    server,
                    self.allowlist_suggestion()
                ));
                diagnostics.push(diag);
            }

            // Rule 2: Duplicate check (always Warning)
            if seen_servers.contains(server) {
                diagnostics.push(
                    Diagnostic::warning(
                        format!("MCP server '{}' is declared multiple times", server),
                        "nsc::ir::mcp_duplicate",
                    ).with_help("Remove duplicate MCP server declarations"),
                );
            }
            seen_servers.insert(server.clone());
        }

        diagnostics
    }

    /// Check if a server name is in the allowlist.
    ///
    /// `Arc<str>` equality first checks pointer identity (same allocation → equal),
    /// then falls back to character-by-character comparison. Semantically correct.
    fn is_allowed(&self, server: &Arc<str>) -> bool {
        self.allowlist.iter().any(|s| s == server)
    }

    /// Build a suggestion string from the top 5 allowlist entries.
    fn allowlist_suggestion(&self) -> String {
        self.allowlist
            .iter()
            .take(5)
            .map(|s| s.as_ref())
            .collect::<Vec<_>>()
            .join(", ")
    }
}

impl Default for MCPDependencyChecker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::SkillIR;

    /// Helper: build a minimal SkillIR with the given MCP servers.
    fn make_ir(mcp_servers: Vec<Arc<str>>) -> SkillIR {
        SkillIR {
            name: Arc::from("test-skill"),
            version: Arc::from("1.0.0"),
            description: "Test".to_string(),
            mcp_servers,
            ..Default::default()
        }
    }

    /// 1. No MCP servers → empty diagnostics.
    #[test]
    fn no_mcp_servers() {
        let checker = MCPDependencyChecker::new();
        let ir = make_ir(vec![]);
        let diags = checker.check(&ir);
        assert!(diags.is_empty());
    }

    /// 2. All servers in allowlist → empty diagnostics.
    #[test]
    fn all_allowed() {
        let checker = MCPDependencyChecker::new();
        let ir = make_ir(vec![Arc::from("filesystem"), Arc::from("github"), Arc::from("postgres")]);
        let diags = checker.check(&ir);
        assert!(diags.is_empty());
    }

    /// 3. Unknown server in default (warning) mode → Warning diagnostic.
    #[test]
    fn unknown_server_warning() {
        let checker = MCPDependencyChecker::new();
        let ir = make_ir(vec![Arc::from("unknown-server")]);
        let diags = checker.check(&ir);
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].code, "nsc::ir::mcp_not_allowed");
        assert_eq!(diags[0].level, ErrorLevel::Warning);
        assert!(diags[0].help.is_some());
    }

    /// 4. Unknown server in strict mode → Error diagnostic.
    #[test]
    fn unknown_server_error() {
        let checker = MCPDependencyChecker::new().with_strict_mode(true);
        let ir = make_ir(vec![Arc::from("unknown-server")]);
        let diags = checker.check(&ir);
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].code, "nsc::ir::mcp_not_allowed");
        assert_eq!(diags[0].level, ErrorLevel::Error);
        assert!(diags[0].help.is_some());
    }

    /// 5. Duplicate server → Warning diagnostic (always Warning, even in strict mode).
    #[test]
    fn duplicate_server() {
        let checker = MCPDependencyChecker::new();
        let ir = make_ir(vec![Arc::from("filesystem"), Arc::from("filesystem")]);
        let diags = checker.check(&ir);
        // Only 1 diagnostic: the duplicate. Both entries are in allowlist so no allowlist diag.
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].code, "nsc::ir::mcp_duplicate");
        assert_eq!(diags[0].level, ErrorLevel::Warning);
    }

    /// 6. Mixed: some allowed, some unknown → diagnostics only for unknown ones.
    #[test]
    fn mixed_allowed_unknown() {
        let checker = MCPDependencyChecker::new();
        let ir = make_ir(vec![Arc::from("filesystem"), Arc::from("unknown-server")]);
        let diags = checker.check(&ir);
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].code, "nsc::ir::mcp_not_allowed");
        assert_eq!(diags[0].level, ErrorLevel::Warning);
    }

    /// 7. Custom allowlist overrides the default.
    #[test]
    fn custom_allowlist() {
        let checker = MCPDependencyChecker::with_allowlist(vec![Arc::from("my-custom-server")]);
        // "filesystem" is no longer allowed
        let ir = make_ir(vec![Arc::from("filesystem")]);
        let diags = checker.check(&ir);
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].code, "nsc::ir::mcp_not_allowed");

        // "my-custom-server" is allowed
        let ir2 = make_ir(vec![Arc::from("my-custom-server")]);
        let diags2 = checker.check(&ir2);
        assert!(diags2.is_empty());
    }

    /// 8. Empty allowlist + strict → every server is Error.
    #[test]
    fn empty_allowlist_strict() {
        let checker = MCPDependencyChecker::with_allowlist(vec![]).with_strict_mode(true);
        let ir = make_ir(vec![Arc::from("filesystem"), Arc::from("github")]);
        let diags = checker.check(&ir);
        assert_eq!(diags.len(), 2);
        assert!(diags.iter().all(|d| d.level == ErrorLevel::Error));
        assert!(diags.iter().all(|d| d.code == "nsc::ir::mcp_not_allowed"));
    }

    /// 9. Multiple unknown servers → each gets its own diagnostic (no early return).
    #[test]
    fn multiple_unknowns() {
        let checker = MCPDependencyChecker::new();
        let ir = make_ir(vec![Arc::from("unknown-a"), Arc::from("unknown-b"), Arc::from("unknown-c")]);
        let diags = checker.check(&ir);
        assert_eq!(diags.len(), 3);
        assert!(diags.iter().all(|d| d.code == "nsc::ir::mcp_not_allowed"));
        // Verify each message mentions its specific server
        assert!(diags[0].message.contains("unknown-a"));
        assert!(diags[1].message.contains("unknown-b"));
        assert!(diags[2].message.contains("unknown-c"));
    }
}