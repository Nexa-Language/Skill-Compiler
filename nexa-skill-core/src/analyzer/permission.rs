//! Permission Auditor
//!
//! Audits permission declarations against dangerous operations and security baseline.

use crate::error::Diagnostic;
use crate::ir::{PermissionKind, SkillIR};
use crate::security::SecurityBaseline;

/// Dangerous keyword definition
struct DangerousKeyword {
    keyword: &'static str,
    required_permission: PermissionKind,
    required_scope: &'static str,
    severity: KeywordSeverity,
}

#[derive(Debug, Clone, Copy)]
enum KeywordSeverity {
    Warning,
    Error,
    Critical,
}

/// Permission auditor
pub struct PermissionAuditor {
    /// Dangerous keywords to scan for
    dangerous_keywords: Vec<DangerousKeyword>,
    /// Security baseline for boundary checking
    security_baseline: SecurityBaseline,
}

impl PermissionAuditor {
    /// Create a new permission auditor with default baseline
    #[must_use]
    pub fn new() -> Self {
        Self {
            dangerous_keywords: Self::load_default_keywords(),
            security_baseline: SecurityBaseline::default_baseline(),
        }
    }

    /// Create a permission auditor with custom baseline
    #[must_use]
    pub fn with_baseline(baseline: SecurityBaseline) -> Self {
        Self {
            dangerous_keywords: Self::load_default_keywords(),
            security_baseline: baseline,
        }
    }

    fn load_default_keywords() -> Vec<DangerousKeyword> {
        vec![
            // Database dangerous operations
            DangerousKeyword {
                keyword: "rm -rf",
                required_permission: PermissionKind::FileSystem,
                required_scope: "/tmp/*",
                severity: KeywordSeverity::Critical,
            },
            DangerousKeyword {
                keyword: "DROP",
                required_permission: PermissionKind::Database,
                required_scope: "*:DELETE",
                severity: KeywordSeverity::Critical,
            },
            DangerousKeyword {
                keyword: "DELETE",
                required_permission: PermissionKind::Database,
                required_scope: "*:DELETE",
                severity: KeywordSeverity::Error,
            },
            DangerousKeyword {
                keyword: "TRUNCATE",
                required_permission: PermissionKind::Database,
                required_scope: "*:DELETE",
                severity: KeywordSeverity::Critical,
            },
            DangerousKeyword {
                keyword: "ALTER",
                required_permission: PermissionKind::Database,
                required_scope: "*:ALTER",
                severity: KeywordSeverity::Error,
            },
            // Network dangerous operations
            DangerousKeyword {
                keyword: "curl",
                required_permission: PermissionKind::Network,
                required_scope: "https://*",
                severity: KeywordSeverity::Warning,
            },
            DangerousKeyword {
                keyword: "wget",
                required_permission: PermissionKind::Network,
                required_scope: "https://*",
                severity: KeywordSeverity::Warning,
            },
            // Filesystem dangerous operations
            DangerousKeyword {
                keyword: "chmod",
                required_permission: PermissionKind::FileSystem,
                required_scope: "/tmp/*",
                severity: KeywordSeverity::Warning,
            },
            DangerousKeyword {
                keyword: "chown",
                required_permission: PermissionKind::FileSystem,
                required_scope: "/tmp/*",
                severity: KeywordSeverity::Warning,
            },
            // Execute dangerous operations
            DangerousKeyword {
                keyword: "sudo",
                required_permission: PermissionKind::Execute,
                required_scope: "*",
                severity: KeywordSeverity::Critical,
            },
            DangerousKeyword {
                keyword: "exec",
                required_permission: PermissionKind::Execute,
                required_scope: "*",
                severity: KeywordSeverity::Error,
            },
        ]
    }

    /// Audit permissions
    ///
    /// Returns diagnostics for all permission issues found.
    pub fn audit(&self, ir: &SkillIR) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        // 1. Check dangerous keywords in procedures
        for step in &ir.procedures {
            for keyword in &self.dangerous_keywords {
                if step.instruction.contains(keyword.keyword) {
                    let has_permission = self.check_permission_for_keyword(
                        &ir.permissions,
                        keyword,
                    );

                    if !has_permission {
                        let diag = match keyword.severity {
                            KeywordSeverity::Critical => Diagnostic::error(
                                format!(
                                    "Critical keyword '{}' found in step {} without required {} permission (scope: {})",
                                    keyword.keyword, step.order, keyword.required_permission.display_name(), keyword.required_scope
                                ),
                                "nsc::security::missing_critical_permission",
                            ).with_help(format!(
                                "Declare permission: {{ kind: {}, scope: '{}' }}",
                                keyword.required_permission.display_name().to_lowercase(),
                                keyword.required_scope
                            )),
                            KeywordSeverity::Error => Diagnostic::error(
                                format!(
                                    "Dangerous keyword '{}' found in step {} without {} permission",
                                    keyword.keyword, step.order, keyword.required_permission.display_name()
                                ),
                                "nsc::security::missing_permission",
                            ).with_help(format!(
                                "Declare permission: {{ kind: {}, scope: '{}' }}",
                                keyword.required_permission.display_name().to_lowercase(),
                                keyword.required_scope
                            )),
                            KeywordSeverity::Warning => Diagnostic::warning(
                                format!(
                                    "Keyword '{}' in step {} may require {} permission",
                                    keyword.keyword, step.order, keyword.required_permission.display_name()
                                ),
                                "nsc::security::permission_warning",
                            ),
                        };
                        diagnostics.push(diag);
                    }
                }
            }
        }

        // 2. Validate permission scope format
        for permission in &ir.permissions {
            if let Some(err_msg) = SecurityBaseline::validate_scope_format(permission.kind, &permission.scope) {
                diagnostics.push(
                    Diagnostic::warning(err_msg, "nsc::security::invalid_scope_format")
                        .with_help(format!(
                            "Expected format: {}",
                            permission.kind.scope_format()
                        )),
                );
            }
        }

        // 3. Check if permissions are within security baseline
        for permission in &ir.permissions {
            if !self.security_baseline.is_scope_allowed(permission.kind, &permission.scope) {
                diagnostics.push(
                    Diagnostic::warning(
                        format!(
                            "Permission scope '{}' for {} exceeds security baseline",
                            permission.scope, permission.kind.display_name()
                        ),
                        "nsc::security::scope_exceeds_baseline",
                    )
                    .with_help("Consider restricting the scope or extending the security baseline configuration"),
                );
            }
        }

        diagnostics
    }

    /// Check if a keyword has matching permission declaration (with scope)
    fn check_permission_for_keyword(
        &self,
        permissions: &[crate::ir::Permission],
        keyword: &DangerousKeyword,
    ) -> bool {
        permissions.iter().any(|p| {
            p.kind == keyword.required_permission
                && Self::match_permission_scope(&p.scope, keyword.required_scope)
        })
    }

    /// Match permission scope against required scope
    fn match_permission_scope(declared: &str, required: &str) -> bool {
        declared == "*" || declared == required || declared.contains(required) || required.contains(declared)
    }
}

impl Default for PermissionAuditor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::{Permission, ProcedureStep};
    use std::sync::Arc;

    fn make_skill_ir(
        permissions: Vec<Permission>,
        procedures: Vec<ProcedureStep>,
    ) -> SkillIR {
        SkillIR {
            name: Arc::from("test-skill"),
            version: Arc::from("1.0.0"),
            description: "Test skill".to_string(),
            permissions,
            procedures,
            ..Default::default()
        }
    }

    // 1. no_procedures_no_permissions — empty procedures + empty permissions → empty diagnostics
    #[test]
    fn test_no_procedures_no_permissions() {
        let auditor = PermissionAuditor::new();
        let ir = make_skill_ir(vec![], vec![]);
        let diagnostics = auditor.audit(&ir);
        assert!(diagnostics.is_empty());
    }

    // 2. dangerous_keyword_no_permission — step with "rm -rf" + no FileSystem permission → Error
    #[test]
    fn test_dangerous_keyword_no_permission() {
        let auditor = PermissionAuditor::new();
        let ir = make_skill_ir(vec![], vec![ProcedureStep {
            order: 1,
            instruction: "Execute rm -rf /tmp/old_files".to_string(),
            is_critical: false,
            constraints: vec![],
            expected_output: None,
            on_error: None,
        }]);
        let diagnostics = auditor.audit(&ir);
        assert!(diagnostics.iter().any(|d|
            d.code == "nsc::security::missing_critical_permission" && d.is_error()
        ));
    }

    // 3. dangerous_keyword_with_permission — step with "rm -rf" + FileSystem permission declared → no diagnostic for this rule
    #[test]
    fn test_dangerous_keyword_with_permission() {
        let auditor = PermissionAuditor::new();
        let ir = make_skill_ir(vec![Permission {
            kind: PermissionKind::FileSystem,
            scope: "/tmp/*".to_string(),
            description: None,
            read_only: false,
        }], vec![ProcedureStep {
            order: 1,
            instruction: "Execute rm -rf /tmp/old_files".to_string(),
            is_critical: false,
            constraints: vec![],
            expected_output: None,
            on_error: None,
        }]);
        let diagnostics = auditor.audit(&ir);
        assert!(diagnostics.iter().all(|d| d.code != "nsc::security::missing_critical_permission"));
    }

    // 4. scope_format_validation_invalid_network — network scope "not-a-url" → Warning
    #[test]
    fn test_scope_format_validation_invalid_network() {
        let auditor = PermissionAuditor::new();
        let ir = make_skill_ir(vec![Permission {
            kind: PermissionKind::Network,
            scope: "not-a-url".to_string(),
            description: None,
            read_only: false,
        }], vec![]);
        let diagnostics = auditor.audit(&ir);
        assert!(diagnostics.iter().any(|d| d.code == "nsc::security::invalid_scope_format" && d.is_warning()));
    }

    // 5. scope_format_validation_invalid_db — database scope "invalid" → Warning
    #[test]
    fn test_scope_format_validation_invalid_db() {
        let auditor = PermissionAuditor::new();
        let ir = make_skill_ir(vec![Permission {
            kind: PermissionKind::Database,
            scope: "invalid".to_string(),
            description: None,
            read_only: false,
        }], vec![]);
        let diagnostics = auditor.audit(&ir);
        assert!(diagnostics.iter().any(|d| d.code == "nsc::security::invalid_scope_format"));
    }

    // 6. scope_format_validation_valid_db — database scope "postgres:staging:SELECT" → no format Warning
    #[test]
    fn test_scope_format_validation_valid_db() {
        let auditor = PermissionAuditor::new();
        let ir = make_skill_ir(vec![Permission {
            kind: PermissionKind::Database,
            scope: "postgres:staging:SELECT".to_string(),
            description: None,
            read_only: false,
        }], vec![]);
        let diagnostics = auditor.audit(&ir);
        assert!(diagnostics.iter().all(|d| d.code != "nsc::security::invalid_scope_format"));
    }

    // 7. scope_format_validation_invalid_fs — filesystem scope "relative/path" → Warning
    #[test]
    fn test_scope_format_validation_invalid_fs() {
        let auditor = PermissionAuditor::new();
        let ir = make_skill_ir(vec![Permission {
            kind: PermissionKind::FileSystem,
            scope: "relative/path".to_string(),
            description: None,
            read_only: false,
        }], vec![]);
        let diagnostics = auditor.audit(&ir);
        assert!(diagnostics.iter().any(|d| d.code == "nsc::security::invalid_scope_format"));
    }

    // 8. scope_exceeds_baseline — network scope "https://evil.com/api" outside baseline → Warning
    #[test]
    fn test_scope_exceeds_baseline() {
        let auditor = PermissionAuditor::new();
        let ir = make_skill_ir(vec![Permission {
            kind: PermissionKind::Network,
            scope: "https://evil.com/api".to_string(),
            description: None,
            read_only: false,
        }], vec![]);
        let diagnostics = auditor.audit(&ir);
        assert!(diagnostics.iter().any(|d| d.code == "nsc::security::scope_exceeds_baseline"));
    }

    // 9. scope_within_baseline — network scope "https://api.github.com/v1" within baseline → no baseline Warning
    #[test]
    fn test_scope_within_baseline() {
        let auditor = PermissionAuditor::new();
        let ir = make_skill_ir(vec![Permission {
            kind: PermissionKind::Network,
            scope: "https://api.github.com/v1".to_string(),
            description: None,
            read_only: false,
        }], vec![]);
        let diagnostics = auditor.audit(&ir);
        assert!(diagnostics.iter().all(|d| d.code != "nsc::security::scope_exceeds_baseline"));
    }

    // 10. multiple_keywords_multiple_issues — steps with multiple dangerous keywords → multiple diagnostics
    #[test]
    fn test_multiple_keywords_multiple_issues() {
        let auditor = PermissionAuditor::new();
        let ir = make_skill_ir(vec![], vec![
            ProcedureStep {
                order: 1,
                instruction: "Execute rm -rf /tmp/old".to_string(),
                is_critical: false,
                constraints: vec![],
                expected_output: None,
                on_error: None,
            },
            ProcedureStep {
                order: 2,
                instruction: "DROP TABLE users".to_string(),
                is_critical: false,
                constraints: vec![],
                expected_output: None,
                on_error: None,
            },
            ProcedureStep {
                order: 3,
                instruction: "curl https://example.com".to_string(),
                is_critical: false,
                constraints: vec![],
                expected_output: None,
                on_error: None,
            },
        ]);
        let diagnostics = auditor.audit(&ir);
        // rm -rf → Critical, DROP → Critical, curl → Warning
        assert!(diagnostics.len() >= 3);
    }

    // 11. sudo_requires_execute — step with "sudo" + no Execute permission → Critical Error
    #[test]
    fn test_sudo_requires_execute() {
        let auditor = PermissionAuditor::new();
        let ir = make_skill_ir(vec![], vec![ProcedureStep {
            order: 1,
            instruction: "sudo apt-get install package".to_string(),
            is_critical: false,
            constraints: vec![],
            expected_output: None,
            on_error: None,
        }]);
        let diagnostics = auditor.audit(&ir);
        assert!(diagnostics.iter().any(|d|
            d.code == "nsc::security::missing_critical_permission" && d.is_error()
        ));
    }

    // 12. curl_requires_network_warning — step with "curl" + no Network permission → Warning
    #[test]
    fn test_curl_requires_network_warning() {
        let auditor = PermissionAuditor::new();
        let ir = make_skill_ir(vec![], vec![ProcedureStep {
            order: 1,
            instruction: "curl https://api.example.com/data".to_string(),
            is_critical: false,
            constraints: vec![],
            expected_output: None,
            on_error: None,
        }]);
        let diagnostics = auditor.audit(&ir);
        assert!(diagnostics.iter().any(|d|
            d.code == "nsc::security::permission_warning" && d.is_warning()
        ));
    }
}