//! Security Level Types
//!
//! Re-exports SecurityLevel from IR and provides level validation.

pub use crate::ir::SecurityLevel;

use crate::error::Diagnostic;
use crate::ir::{PermissionKind, SkillIR};

/// Audit check types corresponding to security levels
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuditCheck {
    /// Format validation
    FormatValidation,
    /// Schema validation
    SchemaValidation,
    /// Permission declaration check
    PermissionDeclaration,
    /// MCP allowlist check
    MCPAllowlist,
    /// Dangerous keyword scan
    DangerousKeywordScan,
    /// HITL requirement check
    HITLRequired,
    /// Manual approval requirement
    ManualApproval,
}

impl SecurityLevel {
    /// Get required audit checks for this security level
    pub fn audit_checks(&self) -> Vec<AuditCheck> {
        match self {
            SecurityLevel::Low => vec![
                AuditCheck::FormatValidation,
                AuditCheck::SchemaValidation,
            ],
            SecurityLevel::Medium => vec![
                AuditCheck::FormatValidation,
                AuditCheck::SchemaValidation,
                AuditCheck::PermissionDeclaration,
                AuditCheck::MCPAllowlist,
            ],
            SecurityLevel::High => vec![
                AuditCheck::FormatValidation,
                AuditCheck::SchemaValidation,
                AuditCheck::PermissionDeclaration,
                AuditCheck::MCPAllowlist,
                AuditCheck::DangerousKeywordScan,
                AuditCheck::HITLRequired,
            ],
            SecurityLevel::Critical => vec![
                AuditCheck::FormatValidation,
                AuditCheck::SchemaValidation,
                AuditCheck::PermissionDeclaration,
                AuditCheck::MCPAllowlist,
                AuditCheck::DangerousKeywordScan,
                AuditCheck::HITLRequired,
                AuditCheck::ManualApproval,
            ],
        }
    }
}

/// Security level validator
///
/// Validates consistency between declared security level and skill configuration.
pub struct SecurityLevelValidator;

impl SecurityLevelValidator {
    /// Validate security level consistency with skill configuration
    pub fn validate(ir: &SkillIR) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        // 1. HITL requirement check
        if ir.security_level.requires_hitl() && !ir.hitl_required {
            diagnostics.push(
                Diagnostic::error(
                    format!(
                        "Security level '{}' requires hitl_required to be true",
                        ir.security_level
                    ),
                    "nsc::security::hitl_required",
                )
                .with_help("Set hitl_required: true in the frontmatter"),
            );
        }

        // 2. Critical-level additional requirements
        if ir.security_level == SecurityLevel::Critical {
            if ir.permissions.is_empty() {
                diagnostics.push(
                    Diagnostic::error(
                        "Critical security level requires explicit permission declarations",
                        "nsc::security::missing_permissions",
                    )
                    .with_help("Declare at least one permission in the frontmatter"),
                );
            }

            if ir.pre_conditions.is_empty() {
                diagnostics.push(
                    Diagnostic::warning(
                        "Critical security level should have pre-conditions defined",
                        "nsc::security::missing_preconditions",
                    )
                    .with_help("Add pre_conditions to guard dangerous operations"),
                );
            }

            if ir.fallbacks.is_empty() {
                diagnostics.push(
                    Diagnostic::warning(
                        "Critical security level should have fallback strategies defined",
                        "nsc::security::missing_fallbacks",
                    )
                    .with_help("Add fallbacks for error recovery scenarios"),
                );
            }
        }

        // 3. Permission-security level mismatch
        for permission in &ir.permissions {
            if permission.kind == PermissionKind::Database
                && permission.scope.contains("ALL")
                && ir.security_level != SecurityLevel::High
                && ir.security_level != SecurityLevel::Critical
            {
                diagnostics.push(
                    Diagnostic::warning(
                        "Database ALL permission should be used with High or Critical security level",
                        "nsc::security::permission_level_mismatch",
                    )
                    .with_help("Consider upgrading security_level to 'high' or restricting database scope"),
                );
            }
        }

        diagnostics
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::Permission;
    use crate::ir::PermissionKind;
    use std::sync::Arc;

    fn make_skill_ir(
        security_level: SecurityLevel,
        hitl_required: bool,
        permissions: Vec<Permission>,
        pre_conditions: Vec<String>,
        fallbacks: Vec<String>,
    ) -> SkillIR {
        SkillIR {
            name: Arc::from("test-skill"),
            version: Arc::from("1.0.0"),
            description: "Test skill".to_string(),
            security_level,
            hitl_required,
            permissions,
            pre_conditions,
            fallbacks,
            ..Default::default()
        }
    }

    // 1. low_level_no_hitl_ok — Low + hitl_required=false → no diagnostic
    #[test]
    fn test_low_level_no_hitl_ok() {
        let ir = make_skill_ir(SecurityLevel::Low, false, vec![], vec![], vec![]);
        let diagnostics = SecurityLevelValidator::validate(&ir);
        let hitl_errors = diagnostics.iter().any(|d| d.code == "nsc::security::hitl_required");
        assert!(!hitl_errors);
    }

    // 2. high_level_requires_hitl — High + hitl_required=false → Error diagnostic
    #[test]
    fn test_high_level_requires_hitl() {
        let ir = make_skill_ir(SecurityLevel::High, false, vec![], vec![], vec![]);
        let diagnostics = SecurityLevelValidator::validate(&ir);
        assert!(diagnostics.iter().any(|d| d.code == "nsc::security::hitl_required" && d.is_error()));
    }

    // 3. critical_level_requires_hitl — Critical + hitl_required=false → Error diagnostic
    #[test]
    fn test_critical_level_requires_hitl() {
        let ir = make_skill_ir(SecurityLevel::Critical, false, vec![], vec![], vec![]);
        let diagnostics = SecurityLevelValidator::validate(&ir);
        assert!(diagnostics.iter().any(|d| d.code == "nsc::security::hitl_required" && d.is_error()));
    }

    // 4. critical_no_permissions — Critical + empty permissions → Error diagnostic
    #[test]
    fn test_critical_no_permissions() {
        let ir = make_skill_ir(SecurityLevel::Critical, true, vec![], vec![], vec![]);
        let diagnostics = SecurityLevelValidator::validate(&ir);
        assert!(diagnostics.iter().any(|d| d.code == "nsc::security::missing_permissions" && d.is_error()));
    }

    // 5. critical_no_preconditions — Critical + empty pre_conditions → Warning
    #[test]
    fn test_critical_no_preconditions() {
        let ir = make_skill_ir(SecurityLevel::Critical, true, vec![Permission {
            kind: PermissionKind::FileSystem,
            scope: "/tmp/*".to_string(),
            description: None,
            read_only: false,
        }], vec![], vec![]);
        let diagnostics = SecurityLevelValidator::validate(&ir);
        assert!(diagnostics.iter().any(|d| d.code == "nsc::security::missing_preconditions" && d.is_warning()));
    }

    // 6. critical_no_fallbacks — Critical + empty fallbacks → Warning
    #[test]
    fn test_critical_no_fallbacks() {
        let ir = make_skill_ir(SecurityLevel::Critical, true, vec![Permission {
            kind: PermissionKind::FileSystem,
            scope: "/tmp/*".to_string(),
            description: None,
            read_only: false,
        }], vec!["Check condition".to_string()], vec![]);
        let diagnostics = SecurityLevelValidator::validate(&ir);
        assert!(diagnostics.iter().any(|d| d.code == "nsc::security::missing_fallbacks" && d.is_warning()));
    }

    // 7. db_all_with_medium — Database ALL + Medium → Warning
    #[test]
    fn test_db_all_with_medium() {
        let ir = make_skill_ir(SecurityLevel::Medium, false, vec![Permission {
            kind: PermissionKind::Database,
            scope: "postgres:*:ALL".to_string(),
            description: None,
            read_only: false,
        }], vec![], vec![]);
        let diagnostics = SecurityLevelValidator::validate(&ir);
        assert!(diagnostics.iter().any(|d| d.code == "nsc::security::permission_level_mismatch" && d.is_warning()));
    }

    // 8. db_all_with_high — Database ALL + High → no diagnostic for this rule
    #[test]
    fn test_db_all_with_high() {
        let ir = make_skill_ir(SecurityLevel::High, true, vec![Permission {
            kind: PermissionKind::Database,
            scope: "postgres:*:ALL".to_string(),
            description: None,
            read_only: false,
        }], vec![], vec![]);
        let diagnostics = SecurityLevelValidator::validate(&ir);
        assert!(diagnostics.iter().all(|d| d.code != "nsc::security::permission_level_mismatch"));
    }

    // 9. all_valid_critical — Critical with all required fields → no blocking errors
    #[test]
    fn test_all_valid_critical() {
        let ir = make_skill_ir(
            SecurityLevel::Critical,
            true,
            vec![Permission {
                kind: PermissionKind::FileSystem,
                scope: "/tmp/*".to_string(),
                description: None,
                read_only: false,
            }],
            vec!["Check precondition".to_string()],
            vec!["Retry with smaller batch".to_string()],
        );
        let diagnostics = SecurityLevelValidator::validate(&ir);
        let blocking_errors = diagnostics.iter().filter(|d| d.is_blocking()).count();
        assert_eq!(blocking_errors, 0);
    }
}