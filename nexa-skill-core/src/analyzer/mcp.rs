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
            // Default allowlist - can be extended via configuration
            allowed_servers: vec![
                "filesystem-server".to_string(),
                "github-server".to_string(),
                "postgres-server".to_string(),
            ],
        }
    }

    /// Check MCP server dependencies
    ///
    /// # Errors
    ///
    /// Returns an error if a required MCP server is not in the allowlist.
    pub fn check(&self, ir: &SkillIR) -> Result<(), AnalysisError> {
        for server in &ir.mcp_servers {
            if !self.is_allowed(server) {
                return Err(AnalysisError::MCPNotAllowed(server.to_string()));
            }
        }
        Ok(())
    }

    /// Check if a server is in the allowlist
    fn is_allowed(&self, server: &str) -> bool {
        self.allowed_servers.iter().any(|s| s == server)
    }
}

impl Default for MCPDependencyChecker {
    fn default() -> Self {
        Self::new()
    }
}
