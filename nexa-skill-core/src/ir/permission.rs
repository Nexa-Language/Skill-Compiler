//! Permission Definition
//!
//! Represents permission declarations for skill execution.

use serde::{Deserialize, Serialize};

/// Permission declaration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Permission {
    /// Permission type
    pub kind: PermissionKind,
    /// Permission scope
    pub scope: String,
    /// Optional description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Whether this is read-only access
    #[serde(default)]
    pub read_only: bool,
}

/// Permission type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PermissionKind {
    /// Network access permission
    Network,
    /// File system permission
    #[serde(alias = "fs")]
    FileSystem,
    /// Database permission
    #[serde(alias = "db")]
    Database,
    /// Command execution permission
    #[serde(alias = "exec")]
    Execute,
    /// MCP server permission
    MCP,
    /// Environment variable permission
    Environment,
}

impl PermissionKind {
    /// Get display name
    #[must_use]
    pub const fn display_name(&self) -> &'static str {
        match self {
            Self::Network => "Network",
            Self::FileSystem => "FileSystem",
            Self::Database => "Database",
            Self::Execute => "Execute",
            Self::MCP => "MCP",
            Self::Environment => "Environment",
        }
    }

    /// Get scope format description
    #[must_use]
    pub const fn scope_format(&self) -> &'static str {
        match self {
            Self::Network => "URL pattern (e.g., https://api.example.com/*)",
            Self::FileSystem => "File path pattern (e.g., /tmp/skill-*)",
            Self::Database => "db_type:db_name:operation (e.g., postgres:staging:SELECT)",
            Self::Execute => "Command pattern (e.g., git:*)",
            Self::MCP => "MCP server name",
            Self::Environment => "Environment variable pattern (e.g., API_KEY_*)",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_permission_kind_display_name() {
        assert_eq!(PermissionKind::Network.display_name(), "Network");
        assert_eq!(PermissionKind::FileSystem.display_name(), "FileSystem");
        assert_eq!(PermissionKind::Database.display_name(), "Database");
        assert_eq!(PermissionKind::Execute.display_name(), "Execute");
        assert_eq!(PermissionKind::MCP.display_name(), "MCP");
        assert_eq!(PermissionKind::Environment.display_name(), "Environment");
    }

    #[test]
    fn test_permission_kind_scope_format() {
        assert!(PermissionKind::Network.scope_format().contains("URL"));
        assert!(PermissionKind::FileSystem.scope_format().contains("path"));
        assert!(PermissionKind::Database.scope_format().contains("db_type"));
        assert!(PermissionKind::Execute.scope_format().contains("Command"));
        assert!(PermissionKind::MCP.scope_format().contains("MCP server"));
        assert!(PermissionKind::Environment.scope_format().contains("variable"));
    }
}
