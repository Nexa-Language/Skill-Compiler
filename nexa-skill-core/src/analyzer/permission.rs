//! Permission Auditor
//!
//! Audits permission declarations against dangerous operations.

use crate::error::Diagnostic;
use crate::ir::{PermissionKind, SkillIR};

/// Permission auditor
pub struct PermissionAuditor {
    /// Dangerous keywords
    dangerous_keywords: Vec<DangerousKeyword>,
}

/// Dangerous keyword definition
struct DangerousKeyword {
    keyword: &'static str,
    required_permission: PermissionKind,
    severity: KeywordSeverity,
}

#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
enum KeywordSeverity {
    Warning,
    Error,
    Critical,
}

impl PermissionAuditor {
    /// Create a new permission auditor
    #[must_use]
    pub fn new() -> Self {
        Self {
            dangerous_keywords: Self::load_default_keywords(),
        }
    }

    /// Load default dangerous keywords
    fn load_default_keywords() -> Vec<DangerousKeyword> {
        vec![
            // File system dangerous operations
            DangerousKeyword {
                keyword: "rm -rf",
                required_permission: PermissionKind::FileSystem,
                severity: KeywordSeverity::Critical,
            },
            DangerousKeyword {
                keyword: "DROP",
                required_permission: PermissionKind::Database,
                severity: KeywordSeverity::Critical,
            },
            DangerousKeyword {
                keyword: "DELETE",
                required_permission: PermissionKind::Database,
                severity: KeywordSeverity::Error,
            },
            DangerousKeyword {
                keyword: "TRUNCATE",
                required_permission: PermissionKind::Database,
                severity: KeywordSeverity::Critical,
            },
            DangerousKeyword {
                keyword: "ALTER",
                required_permission: PermissionKind::Database,
                severity: KeywordSeverity::Error,
            },
        ]
    }

    /// Audit permissions
    ///
    /// # Errors
    ///
    /// Returns diagnostics for permission issues.
    pub fn audit(&self, ir: &SkillIR) -> Result<Vec<Diagnostic>, ()> {
        let mut diagnostics = Vec::new();

        // Check procedures for dangerous keywords
        for step in &ir.procedures {
            for keyword in &self.dangerous_keywords {
                if step.instruction.contains(keyword.keyword) {
                    let has_permission = ir
                        .permissions
                        .iter()
                        .any(|p| p.kind == keyword.required_permission);

                    if !has_permission {
                        let diag = match keyword.severity {
                            KeywordSeverity::Critical => Diagnostic::error(
                                format!(
                                    "Critical keyword '{}' found in step {} without required permission",
                                    keyword.keyword, step.order
                                ),
                                "nsc::security::missing_critical_permission",
                            ),
                            KeywordSeverity::Error => Diagnostic::error(
                                format!(
                                    "Dangerous keyword '{}' found in step {} without permission declaration",
                                    keyword.keyword, step.order
                                ),
                                "nsc::security::missing_permission",
                            ),
                            KeywordSeverity::Warning => Diagnostic::warning(
                                format!(
                                    "Keyword '{}' in step {} may require additional permission",
                                    keyword.keyword, step.order
                                ),
                                "nsc::security::permission_warning",
                            ),
                        };
                        diagnostics.push(diag);
                    }
                }
            }
        }

        Ok(diagnostics)
    }
}

impl Default for PermissionAuditor {
    fn default() -> Self {
        Self::new()
    }
}
