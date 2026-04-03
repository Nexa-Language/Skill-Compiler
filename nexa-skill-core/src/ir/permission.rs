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
