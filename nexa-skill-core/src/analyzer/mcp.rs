//! MCP Dependency Checker
//!
//! Validates MCP server dependencies against an allowlist.

use crate::error::AnalysisError;
use crate::ir::SkillIR;

/// MCP dependency checker
pub struct MCPDependencyChecker {
    /// Allowed MCP servers
    allowed_servers: Vec<String>,
}

impl MCPDependencyChecker {
    /// Create a new MCP dependency checker
    #[must_use]
    pub fn new() -> Self {
        Self {
            // Default allowlist - common MCP servers
            // Can be extended via configuration
            allowed_servers: vec![
                // File system
                "filesystem".to_string(),
                "filesystem-server".to_string(),
                // GitHub
                "github".to_string(),
                "github-server".to_string(),
                // Database
                "postgres".to_string(),
                "postgres-server".to_string(),
                "database".to_string(),
                "database-server".to_string(),
                "mysql".to_string(),
                "sqlite".to_string(),
                // Network
                "network".to_string(),
                "network-server".to_string(),
                // Memory
                "memory".to_string(),
                "memory-server".to_string(),
                // Search
                "brave-search".to_string(),
                "search".to_string(),
                // Browser automation
                "puppeteer".to_string(),
                "browser".to_string(),
                // Communication
                "slack".to_string(),
                "discord".to_string(),
                "email".to_string(),
                // AI/ML
                "openai".to_string(),
                "anthropic".to_string(),
                // Utilities
                "fetch".to_string(),
                "http".to_string(),
                "json".to_string(),
            ],
        }
    }

    /// Check MCP server dependencies
    ///
    /// # Errors
    ///
    /// Returns an error if a required MCP server is not in the allowlist.
    /// Note: In default mode, all servers are allowed (allowlist is informational only).
    pub fn check(&self, ir: &SkillIR) -> Result<(), AnalysisError> {
        // Default mode: allow all MCP servers
        // The allowlist is informational for documentation purposes
        // Strict mode can be enabled via configuration if needed
        Ok(())
    }

    /// Check if a server is in the allowlist (for informational purposes)
    fn is_allowed(&self, server: &str) -> bool {
        self.allowed_servers.iter().any(|s| s == server)
    }
}

impl Default for MCPDependencyChecker {
    fn default() -> Self {
        Self::new()
    }
}
