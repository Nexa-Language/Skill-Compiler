//! Analyzer Module
//!
//! This module provides semantic analysis and validation for SkillIR.

mod anti_skill;
mod mcp;
mod nested_data;
mod permission;
mod schema;

pub use anti_skill::AntiSkillInjector;
pub use mcp::MCPDependencyChecker;
pub use nested_data::{NestedDataDetector, DEFAULT_YAML_OPTIMIZATION_THRESHOLD};
pub use permission::PermissionAuditor;
pub use schema::SchemaValidator;

use crate::error::Diagnostic;
use crate::ir::SkillIR;

/// Validated SkillIR wrapper with non-blocking diagnostics
#[derive(Debug, Clone)]
pub struct ValidatedSkillIR(SkillIR, Vec<Diagnostic>);

impl ValidatedSkillIR {
    /// Create a new validated IR wrapper with non-blocking diagnostics
    ///
    /// Only stores non-blocking (warning) diagnostics; blocking diagnostics
    /// are returned via the `Err` path of `analyze()`.
    #[must_use]
    pub fn new(ir: SkillIR, warnings: Vec<Diagnostic>) -> Self {
        // Only store non-blocking diagnostics
        let warnings = warnings.into_iter().filter(|d| !d.is_blocking()).collect();
        Self(ir, warnings)
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

    /// Get non-blocking diagnostics (warnings)
    #[must_use]
    pub fn warnings(&self) -> &[Diagnostic] {
        &self.1
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
    /// Nested data detector
    nested_data_detector: NestedDataDetector,
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
            nested_data_detector: NestedDataDetector::new(),
        }
    }

    /// Analyze a SkillIR
    ///
    /// # Errors
    ///
    /// Returns an error if analysis fails with blocking errors.
    pub fn analyze(&self, ir: SkillIR) -> Result<ValidatedSkillIR, (SkillIR, Vec<Diagnostic>)> {
        let mut diagnostics = Vec::new();

        // Detect nested data depth and set optimization flags
        let ir = self.nested_data_detector.detect(ir);

        // Run all analyzers
        diagnostics.extend(self.schema_validator.validate(&ir));

        diagnostics.extend(self.mcp_checker.check(&ir));

        diagnostics.extend(self.permission_auditor.audit(&ir));

        // Inject anti-skill constraints
        let ir = self.anti_skill_injector.inject(ir);

        // Check for blocking errors
        if diagnostics.iter().any(|d| d.is_blocking()) {
            return Err((ir, diagnostics));
        }

        Ok(ValidatedSkillIR::new(ir, diagnostics))
    }
}

impl Default for Analyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::{ProcedureStep, SecurityLevel, SkillIR};
    use std::sync::Arc;

    fn make_step(order: u32, instruction: &str) -> ProcedureStep {
        ProcedureStep {
            order,
            instruction: instruction.to_string(),
            is_critical: false,
            constraints: vec![],
            expected_output: None,
            on_error: None,
        }
    }

    fn make_valid_ir() -> SkillIR {
        SkillIR {
            name: Arc::from("valid-skill"),
            version: Arc::from("1.0.0"),
            description: "A valid test skill".to_string(),
            procedures: vec![make_step(1, "Execute task")],
            ..Default::default()
        }
    }

    #[test]
    fn test_analyze_valid_ir() {
        let analyzer = Analyzer::new();
        let result = analyzer.analyze(make_valid_ir());
        assert!(result.is_ok());
        let validated = result.unwrap();
        assert_eq!(validated.as_ref().name, Arc::from("valid-skill"));
    }

    #[test]
    fn test_analyze_invalid_name() {
        let ir = SkillIR {
            name: Arc::from("INVALID-NAME"), // uppercase not kebab-case
            version: Arc::from("1.0.0"),
            description: "Test".to_string(),
            procedures: vec![make_step(1, "Do something")],
            ..Default::default()
        };
        let analyzer = Analyzer::new();
        let result = analyzer.analyze(ir);
        assert!(result.is_err());
        let (_, diagnostics) = result.unwrap_err();
        assert!(diagnostics.iter().any(|d| d.code.contains("invalid_name")));
    }

    #[test]
    fn test_analyze_empty_description() {
        let ir = SkillIR {
            name: Arc::from("test-skill"),
            version: Arc::from("1.0.0"),
            description: String::new(), // Empty description
            procedures: vec![make_step(1, "Do something")],
            ..Default::default()
        };
        let analyzer = Analyzer::new();
        let result = analyzer.analyze(ir);
        assert!(result.is_err());
        let (_, diagnostics) = result.unwrap_err();
        assert!(diagnostics.iter().any(|d| d.code.contains("description_length")));
    }

    #[test]
    fn test_analyze_nested_data_flag_set() {
        let ir = SkillIR {
            name: Arc::from("nested-skill"),
            version: Arc::from("1.0.0"),
            description: "Test".to_string(),
            input_schema: Some(serde_json::json!({
                "type": "object",
                "properties": {
                    "data": {
                        "type": "object",
                        "properties": {
                            "nested": {
                                "type": "object",
                                "properties": {
                                    "deep": { "type": "string" }
                                }
                            }
                        }
                    }
                }
            })),
            procedures: vec![make_step(1, "Test")],
            ..Default::default()
        };
        let analyzer = Analyzer::new();
        let result = analyzer.analyze(ir);
        // NestedDataDetector runs first, so flags are set regardless of validation outcome
        match result {
            Ok(validated) => {
                assert!(validated.as_ref().requires_yaml_optimization);
                assert!(validated.as_ref().nested_data_depth.is_some());
            }
            Err((ir, _diagnostics)) => {
                // Even on validation failure, nested data flags are preserved
                assert!(ir.requires_yaml_optimization);
                assert!(ir.nested_data_depth.is_some());
            }
        }
    }

    #[test]
    fn test_analyze_hitl_security_mismatch() {
        let ir = SkillIR {
            name: Arc::from("critical-skill"),
            version: Arc::from("1.0.0"),
            description: "Test".to_string(),
            security_level: SecurityLevel::Critical,
            hitl_required: false, // Mismatch: Critical requires HITL
            procedures: vec![make_step(1, "Test")],
            ..Default::default()
        };
        let analyzer = Analyzer::new();
        let result = analyzer.analyze(ir);
        assert!(result.is_err());
        let (_, diagnostics) = result.unwrap_err();
        assert!(diagnostics.iter().any(|d| d.is_blocking()));
    }
}
