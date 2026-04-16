//! Security Baseline
//!
//! Defines allowed security boundaries for permission validation.

use crate::ir::PermissionKind;

/// Database operation definition
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DbOperation {
    /// Database type (e.g., "postgres", "mysql", "sqlite")
    pub db_type: String,
    /// Database name (e.g., "staging", "production", "*")
    pub db_name: String,
    /// Allowed operation (e.g., "SELECT", "INSERT", "ALL")
    pub operation: String,
}

/// Security baseline configuration
///
/// Defines the allowed boundaries for each permission kind.
/// Skills whose permissions exceed these boundaries will receive warnings or errors.
#[derive(Debug, Clone)]
pub struct SecurityBaseline {
    /// Allowed network URL patterns (e.g., "https://api.github.com/*")
    pub allowed_networks: Vec<String>,
    /// Allowed file path patterns (e.g., "/tmp/*", "/home/user/.config/**")
    pub allowed_paths: Vec<String>,
    /// Allowed database operations
    pub allowed_db_operations: Vec<DbOperation>,
    /// Allowed command patterns (e.g., "git:*", "npm:install")
    pub allowed_commands: Vec<String>,
    /// Allowed MCP server names
    pub allowed_mcp_servers: Vec<String>,
    /// Allowed environment variable patterns (e.g., "API_KEY_*")
    pub allowed_env_patterns: Vec<String>,
}

impl SecurityBaseline {
    /// Create default security baseline with sensible defaults
    pub fn default_baseline() -> Self {
        Self {
            allowed_networks: vec![
                "https://api.github.com/*".to_string(),
                "https://api.openai.com/*".to_string(),
                "https://*.amazonaws.com/*".to_string(),
            ],
            allowed_paths: vec![
                "/tmp/*".to_string(),
                "/home/*".to_string(),
                "/var/log/*".to_string(),
            ],
            allowed_db_operations: vec![
                DbOperation {
                    db_type: "postgres".into(),
                    db_name: "*".into(),
                    operation: "SELECT".into(),
                },
                DbOperation {
                    db_type: "postgres".into(),
                    db_name: "staging".into(),
                    operation: "ALL".into(),
                },
                DbOperation {
                    db_type: "mysql".into(),
                    db_name: "*".into(),
                    operation: "SELECT".into(),
                },
                DbOperation {
                    db_type: "sqlite".into(),
                    db_name: "*".into(),
                    operation: "ALL".into(),
                },
            ],
            allowed_commands: vec![
                "git:*".to_string(),
                "npm:install".to_string(),
                "pip:install".to_string(),
                "curl:*".to_string(),
            ],
            allowed_mcp_servers: vec![
                "filesystem-server".to_string(),
                "github-server".to_string(),
                "postgres-server".to_string(),
                "brave-search".to_string(),
                "fetch".to_string(),
            ],
            allowed_env_patterns: vec![
                "API_KEY_*".to_string(),
                "DB_URL_*".to_string(),
                "LOG_LEVEL".to_string(),
            ],
        }
    }

    /// Create an empty baseline (everything restricted)
    pub fn empty() -> Self {
        Self {
            allowed_networks: Vec::new(),
            allowed_paths: Vec::new(),
            allowed_db_operations: Vec::new(),
            allowed_commands: Vec::new(),
            allowed_mcp_servers: Vec::new(),
            allowed_env_patterns: Vec::new(),
        }
    }

    /// Create a permissive baseline (everything allowed)
    pub fn permissive() -> Self {
        Self {
            allowed_networks: vec!["*".to_string()],
            allowed_paths: vec!["*".to_string()],
            allowed_db_operations: vec![DbOperation {
                db_type: "*".into(),
                db_name: "*".into(),
                operation: "ALL".into(),
            }],
            allowed_commands: vec!["*".to_string()],
            allowed_mcp_servers: vec!["*".to_string()],
            allowed_env_patterns: vec!["*".to_string()],
        }
    }

    /// Check if a permission scope is within baseline bounds
    pub fn is_scope_allowed(&self, kind: PermissionKind, scope: &str) -> bool {
        match kind {
            PermissionKind::Network => Self::match_any_pattern(scope, &self.allowed_networks),
            PermissionKind::FileSystem => Self::match_any_pattern(scope, &self.allowed_paths),
            PermissionKind::Database => self.match_db_scope(scope),
            PermissionKind::Execute => Self::match_any_pattern(scope, &self.allowed_commands),
            PermissionKind::MCP => Self::match_any_pattern(scope, &self.allowed_mcp_servers),
            PermissionKind::Environment => Self::match_any_pattern(scope, &self.allowed_env_patterns),
        }
    }

    /// Validate scope format for a permission kind
    /// Returns None if valid, or an error message if invalid
    pub fn validate_scope_format(kind: PermissionKind, scope: &str) -> Option<String> {
        match kind {
            PermissionKind::Network => {
                if !scope.starts_with("http://")
                    && !scope.starts_with("https://")
                    && scope != "*"
                {
                    return Some(format!(
                        "Network scope '{}' should be a URL pattern (e.g., https://api.example.com/*)",
                        scope
                    ));
                }
            }
            PermissionKind::FileSystem => {
                if !scope.starts_with('/') && scope != "*" {
                    return Some(format!(
                        "FileSystem scope '{}' should be an absolute path pattern (e.g., /tmp/*)",
                        scope
                    ));
                }
            }
            PermissionKind::Database => {
                let parts: Vec<&str> = scope.split(':').collect();
                if parts.len() != 3 && scope != "*" {
                    return Some(format!(
                        "Database scope '{}' should follow db_type:db_name:operation format (e.g., postgres:staging:SELECT)",
                        scope
                    ));
                }
                if parts.len() == 3 {
                    let valid_ops = ["SELECT", "INSERT", "UPDATE", "DELETE", "ALTER", "ALL", "*"];
                    if !valid_ops.contains(&parts[2]) {
                        return Some(format!(
                            "Database operation '{}' is not a valid operation. Use one of: SELECT, INSERT, UPDATE, DELETE, ALTER, ALL",
                            parts[2]
                        ));
                    }
                }
            }
            PermissionKind::Execute => {
                if scope.is_empty() {
                    return Some("Execute scope should not be empty".to_string());
                }
            }
            PermissionKind::MCP => {
                if scope.is_empty() {
                    return Some("MCP scope should not be empty".to_string());
                }
            }
            PermissionKind::Environment => {
                if scope.is_empty() {
                    return Some("Environment scope should not be empty".to_string());
                }
            }
        }
        None
    }

    /// Derive read_only flag from filesystem scope
    ///
    /// If scope contains "/Read/" or ends with ":Read", mark as read-only
    pub fn derive_read_only(kind: PermissionKind, scope: &str) -> bool {
        if kind == PermissionKind::FileSystem {
            scope.contains("/Read/") || scope.ends_with(":Read")
        } else if kind == PermissionKind::Database {
            let parts: Vec<&str> = scope.split(':').collect();
            if parts.len() == 3 {
                parts[2] == "SELECT" || parts[2] == "READ"
            } else {
                false
            }
        } else {
            false
        }
    }

    // Private helper methods

    fn match_any_pattern(value: &str, patterns: &[String]) -> bool {
        patterns.iter().any(|p| Self::match_pattern(value, p))
    }

    fn match_pattern(value: &str, pattern: &str) -> bool {
        if pattern == "*" {
            return true;
        }
        if pattern.ends_with('*') {
            value.starts_with(&pattern[..pattern.len() - 1])
        } else if pattern.starts_with('*') {
            value.ends_with(&pattern[1..])
        } else {
            value == pattern
        }
    }

    fn match_db_scope(&self, scope: &str) -> bool {
        if scope == "*" {
            return true;
        }
        let parts: Vec<&str> = scope.split(':').collect();
        if parts.len() != 3 {
            return false;
        }
        self.allowed_db_operations.iter().any(|op| {
            (parts[0] == op.db_type || op.db_type == "*")
                && (parts[1] == op.db_name || op.db_name == "*")
                && (parts[2] == op.operation || op.operation == "ALL" || parts[2] == "ALL")
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // 1. default_baseline creation
    #[test]
    fn test_default_baseline_creation() {
        let baseline = SecurityBaseline::default_baseline();
        assert!(!baseline.allowed_networks.is_empty());
        assert!(!baseline.allowed_paths.is_empty());
        assert!(!baseline.allowed_db_operations.is_empty());
        assert!(!baseline.allowed_commands.is_empty());
        assert!(!baseline.allowed_mcp_servers.is_empty());
        assert!(!baseline.allowed_env_patterns.is_empty());

        // Verify specific default entries
        assert!(baseline.allowed_networks.contains(&"https://api.github.com/*".to_string()));
        assert!(baseline.allowed_paths.contains(&"/tmp/*".to_string()));
        assert!(baseline.allowed_commands.contains(&"git:*".to_string()));
        assert!(baseline.allowed_mcp_servers.contains(&"filesystem-server".to_string()));
        assert!(baseline.allowed_env_patterns.contains(&"API_KEY_*".to_string()));
    }

    // 2. is_scope_allowed for each kind (within baseline)
    #[test]
    fn test_is_scope_allowed_network_match() {
        let baseline = SecurityBaseline::default_baseline();
        assert!(baseline.is_scope_allowed(PermissionKind::Network, "https://api.github.com/v1"));
    }

    #[test]
    fn test_is_scope_allowed_filesystem_match() {
        let baseline = SecurityBaseline::default_baseline();
        assert!(baseline.is_scope_allowed(PermissionKind::FileSystem, "/tmp/output.txt"));
    }

    #[test]
    fn test_is_scope_allowed_database_match() {
        let baseline = SecurityBaseline::default_baseline();
        assert!(baseline.is_scope_allowed(PermissionKind::Database, "postgres:staging:SELECT"));
        assert!(baseline.is_scope_allowed(PermissionKind::Database, "postgres:*:SELECT"));
        assert!(baseline.is_scope_allowed(PermissionKind::Database, "sqlite:*:ALL"));
    }

    #[test]
    fn test_is_scope_allowed_execute_match() {
        let baseline = SecurityBaseline::default_baseline();
        assert!(baseline.is_scope_allowed(PermissionKind::Execute, "git:push"));
        assert!(baseline.is_scope_allowed(PermissionKind::Execute, "npm:install"));
    }

    #[test]
    fn test_is_scope_allowed_mcp_exact() {
        let baseline = SecurityBaseline::default_baseline();
        assert!(baseline.is_scope_allowed(PermissionKind::MCP, "filesystem-server"));
        assert!(baseline.is_scope_allowed(PermissionKind::MCP, "github-server"));
    }

    #[test]
    fn test_is_scope_allowed_env_match() {
        let baseline = SecurityBaseline::default_baseline();
        assert!(baseline.is_scope_allowed(PermissionKind::Environment, "API_KEY_GITHUB"));
        assert!(baseline.is_scope_allowed(PermissionKind::Environment, "LOG_LEVEL"));
    }

    // 3. is_scope_allowed for out-of-baseline scopes
    #[test]
    fn test_is_scope_allowed_out_of_baseline_network() {
        let baseline = SecurityBaseline::default_baseline();
        assert!(!baseline.is_scope_allowed(PermissionKind::Network, "https://evil.com/api"));
    }

    #[test]
    fn test_is_scope_allowed_out_of_baseline_filesystem() {
        let baseline = SecurityBaseline::default_baseline();
        assert!(!baseline.is_scope_allowed(PermissionKind::FileSystem, "/etc/passwd"));
    }

    #[test]
    fn test_is_scope_allowed_out_of_baseline_database() {
        let baseline = SecurityBaseline::default_baseline();
        assert!(!baseline.is_scope_allowed(PermissionKind::Database, "postgres:production:DELETE"));
    }

    #[test]
    fn test_is_scope_allowed_out_of_baseline_mcp() {
        let baseline = SecurityBaseline::default_baseline();
        assert!(!baseline.is_scope_allowed(PermissionKind::MCP, "unknown-server"));
    }

    // 4. validate_scope_format
    #[test]
    fn test_validate_scope_format_valid_network() {
        assert!(SecurityBaseline::validate_scope_format(PermissionKind::Network, "https://api.example.com/*").is_none());
        assert!(SecurityBaseline::validate_scope_format(PermissionKind::Network, "*").is_none());
    }

    #[test]
    fn test_validate_scope_format_invalid_network() {
        let result = SecurityBaseline::validate_scope_format(PermissionKind::Network, "not-a-url");
        assert!(result.is_some());
        assert!(result.unwrap().contains("should be a URL pattern"));
    }

    #[test]
    fn test_validate_scope_format_valid_filesystem() {
        assert!(SecurityBaseline::validate_scope_format(PermissionKind::FileSystem, "/tmp/*").is_none());
        assert!(SecurityBaseline::validate_scope_format(PermissionKind::FileSystem, "*").is_none());
    }

    #[test]
    fn test_validate_scope_format_invalid_filesystem() {
        let result = SecurityBaseline::validate_scope_format(PermissionKind::FileSystem, "relative/path");
        assert!(result.is_some());
        assert!(result.unwrap().contains("should be an absolute path pattern"));
    }

    #[test]
    fn test_validate_scope_format_valid_database() {
        assert!(SecurityBaseline::validate_scope_format(PermissionKind::Database, "postgres:staging:SELECT").is_none());
        assert!(SecurityBaseline::validate_scope_format(PermissionKind::Database, "*").is_none());
    }

    #[test]
    fn test_validate_scope_format_invalid_database_wrong_format() {
        let result = SecurityBaseline::validate_scope_format(PermissionKind::Database, "invalid");
        assert!(result.is_some());
        assert!(result.unwrap().contains("db_type:db_name:operation"));
    }

    #[test]
    fn test_validate_scope_format_invalid_database_wrong_operation() {
        let result = SecurityBaseline::validate_scope_format(PermissionKind::Database, "postgres:staging:INVALID");
        assert!(result.is_some());
        assert!(result.unwrap().contains("not a valid operation"));
    }

    #[test]
    fn test_validate_scope_format_valid_mcp() {
        assert!(SecurityBaseline::validate_scope_format(PermissionKind::MCP, "github-server").is_none());
    }

    #[test]
    fn test_validate_scope_format_empty_mcp() {
        let result = SecurityBaseline::validate_scope_format(PermissionKind::MCP, "");
        assert!(result.is_some());
        assert!(result.unwrap().contains("should not be empty"));
    }

    #[test]
    fn test_validate_scope_format_valid_env() {
        assert!(SecurityBaseline::validate_scope_format(PermissionKind::Environment, "API_KEY_*").is_none());
    }

    // 5. derive_read_only
    #[test]
    fn test_derive_read_only_filesystem_read_path() {
        assert!(SecurityBaseline::derive_read_only(PermissionKind::FileSystem, "/home/Read/data"));
        assert!(SecurityBaseline::derive_read_only(PermissionKind::FileSystem, "/config:Read"));
    }

    #[test]
    fn test_derive_read_only_filesystem_normal_path() {
        assert!(!SecurityBaseline::derive_read_only(PermissionKind::FileSystem, "/tmp/output"));
        assert!(!SecurityBaseline::derive_read_only(PermissionKind::FileSystem, "/home/user/.config"));
    }

    #[test]
    fn test_derive_read_only_database_select() {
        assert!(SecurityBaseline::derive_read_only(PermissionKind::Database, "postgres:staging:SELECT"));
    }

    #[test]
    fn test_derive_read_only_database_all() {
        assert!(!SecurityBaseline::derive_read_only(PermissionKind::Database, "postgres:staging:ALL"));
    }

    #[test]
    fn test_derive_read_only_other_kinds() {
        assert!(!SecurityBaseline::derive_read_only(PermissionKind::Network, "https://*"));
        assert!(!SecurityBaseline::derive_read_only(PermissionKind::Execute, "git:*"));
    }

    // 6. permissive baseline allows everything
    #[test]
    fn test_permissive_baseline_allows_everything() {
        let baseline = SecurityBaseline::permissive();
        assert!(baseline.is_scope_allowed(PermissionKind::Network, "https://evil.com/api"));
        assert!(baseline.is_scope_allowed(PermissionKind::FileSystem, "/etc/passwd"));
        assert!(baseline.is_scope_allowed(PermissionKind::Database, "postgres:production:DELETE"));
        assert!(baseline.is_scope_allowed(PermissionKind::Execute, "sudo"));
        assert!(baseline.is_scope_allowed(PermissionKind::MCP, "anything"));
        assert!(baseline.is_scope_allowed(PermissionKind::Environment, "SECRET_KEY"));
    }

    // 7. empty baseline denies everything
    #[test]
    fn test_empty_baseline_denies_everything() {
        let baseline = SecurityBaseline::empty();
        assert!(!baseline.is_scope_allowed(PermissionKind::Network, "https://api.github.com/v1"));
        assert!(!baseline.is_scope_allowed(PermissionKind::FileSystem, "/tmp/file"));
        assert!(!baseline.is_scope_allowed(PermissionKind::Database, "sqlite:*:ALL"));
        assert!(!baseline.is_scope_allowed(PermissionKind::Execute, "git:push"));
        assert!(!baseline.is_scope_allowed(PermissionKind::MCP, "filesystem-server"));
        assert!(!baseline.is_scope_allowed(PermissionKind::Environment, "LOG_LEVEL"));
    }

    // 8. match_pattern: exact, prefix wildcard, suffix wildcard, full wildcard
    #[test]
    fn test_match_pattern_exact() {
        assert!(SecurityBaseline::match_pattern("LOG_LEVEL", "LOG_LEVEL"));
        assert!(!SecurityBaseline::match_pattern("LOG_LEVEL_DEBUG", "LOG_LEVEL"));
    }

    #[test]
    fn test_match_pattern_prefix_wildcard() {
        assert!(SecurityBaseline::match_pattern("https://api.github.com/v1", "https://api.github.com/*"));
        assert!(SecurityBaseline::match_pattern("https://api.github.com/", "https://api.github.com/*"));
        assert!(!SecurityBaseline::match_pattern("https://evil.com/v1", "https://api.github.com/*"));
    }

    #[test]
    fn test_match_pattern_suffix_wildcard() {
        assert!(SecurityBaseline::match_pattern("my-key", "*-key"));
        assert!(!SecurityBaseline::match_pattern("my-key-extra", "*-key"));
    }

    #[test]
    fn test_match_pattern_full_wildcard() {
        assert!(SecurityBaseline::match_pattern("anything", "*"));
        assert!(SecurityBaseline::match_pattern("", "*"));
    }
}