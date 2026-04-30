//! Security Permission Types
//!
//! Defines permission request structures for security validation.

use crate::ir::{Permission, PermissionKind};

/// Permission request generated when a dangerous operation is detected
/// but no corresponding permission declaration exists.
#[derive(Debug, Clone)]
pub struct PermissionRequest {
    /// The permission that should be declared
    pub permission: Permission,
    /// Source step that triggered this request
    pub source_step: Option<u32>,
    /// Reason for the request
    pub reason: String,
}

impl PermissionRequest {
    /// Create a new permission request
    pub fn new(kind: PermissionKind, scope: &str, reason: &str) -> Self {
        Self {
            permission: Permission {
                kind,
                scope: scope.to_string(),
                description: None,
                read_only: false,
            },
            source_step: None,
            reason: reason.to_string(),
        }
    }

    /// Create a permission request with a source step
    pub fn with_step(kind: PermissionKind, scope: &str, step: u32, reason: &str) -> Self {
        Self {
            permission: Permission {
                kind,
                scope: scope.to_string(),
                description: None,
                read_only: false,
            },
            source_step: Some(step),
            reason: reason.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::PermissionKind;

    #[test]
    fn test_permission_request_new() {
        let req = PermissionRequest::new(
            PermissionKind::FileSystem,
            "/tmp/*",
            "Required for cache operations",
        );
        assert_eq!(req.permission.kind, PermissionKind::FileSystem);
        assert_eq!(req.permission.scope, "/tmp/*");
        assert!(req.source_step.is_none());
        assert_eq!(req.reason, "Required for cache operations");
    }

    #[test]
    fn test_permission_request_with_step() {
        let req = PermissionRequest::with_step(
            PermissionKind::Database,
            "postgres:*:SELECT",
            3,
            "Step 3 requires DB access",
        );
        assert_eq!(req.source_step, Some(3));
        assert_eq!(req.reason, "Step 3 requires DB access");
    }
}