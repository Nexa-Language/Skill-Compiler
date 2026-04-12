//! Analyzer Module
//!
//! This module provides semantic analysis and validation for SkillIR.

mod anti_skill;
mod mcp;
mod permission;
mod schema;

pub use anti_skill::AntiSkillInjector;
pub use mcp::MCPDependencyChecker;
pub use permission::PermissionAuditor;
pub use schema::SchemaValidator;

use crate::error::Diagnostic;
use crate::ir::SkillIR;

/// Validated SkillIR wrapper
#[derive(Debug, Clone)]
pub struct ValidatedSkillIR(SkillIR);

impl ValidatedSkillIR {
    /// Create a new validated IR wrapper
    #[must_use]
    pub fn new(ir: SkillIR) -> Self {
        Self(ir)
    }

    /// Get the inner IR
    #[must_use]
    pub fn as_ref(&self) -> &SkillIR {
        &self.0
    }

    /// Consume and get the inner IR
    #[must_use]
    pub fn into_inner(self) -> SkillIR {
        self.0
    }
}

/// Analyzer orchestrator
pub struct Analyzer {
    /// Schema validator
    schema_validator: SchemaValidator,
    /// MCP dependency checker
    mcp_checker: MCPDependencyChecker,
    /// Permission auditor
    permission_auditor: PermissionAuditor,
    /// Anti-skill injector
    anti_skill_injector: AntiSkillInjector,
}

impl Analyzer {
    /// Create a new analyzer
    #[must_use]
    pub fn new() -> Self {
        Self {
            schema_validator: SchemaValidator::new(),
            mcp_checker: MCPDependencyChecker::new(),
            permission_auditor: PermissionAuditor::new(),
            anti_skill_injector: AntiSkillInjector::new(),
        }
    }

    /// Analyze a SkillIR
    ///
    /// # Errors
    ///
    /// Returns an error if analysis fails with blocking errors.
    pub fn analyze(&self, ir: SkillIR) -> Result<ValidatedSkillIR, (SkillIR, Vec<Diagnostic>)> {
        let mut diagnostics = Vec::new();

        // Run all analyzers
        if let Err(e) = self.schema_validator.validate(&ir) {
            diagnostics.push(Diagnostic::error(e.to_string(), "nsc::analysis::schema"));
        }

        if let Err(e) = self.mcp_checker.check(&ir) {
            diagnostics.push(Diagnostic::error(e.to_string(), "nsc::analysis::mcp"));
        }

        diagnostics.extend(self.permission_auditor.audit(&ir).unwrap_or_default());

        // Inject anti-skill constraints
        let ir = self.anti_skill_injector.inject(ir);

        // Check for blocking errors
        if diagnostics.iter().any(|d| d.is_blocking()) {
            return Err((ir, diagnostics));
        }

        Ok(ValidatedSkillIR::new(ir))
    }
}

impl Default for Analyzer {
    fn default() -> Self {
        Self::new()
    }
}
