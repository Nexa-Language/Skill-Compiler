//! Schema Validator
//!
//! Validates JSON Schema definitions and parameter references.

use crate::error::AnalysisError;
use crate::ir::SkillIR;

/// Schema validator
pub struct SchemaValidator;

impl SchemaValidator {
    /// Create a new schema validator
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// Validate input/output schemas
    ///
    /// # Errors
    ///
    /// Returns an error if schema validation fails.
    pub fn validate(&self, ir: &SkillIR) -> Result<(), AnalysisError> {
        // Validate input schema if present
        if let Some(ref schema) = ir.input_schema {
            self.validate_json_schema(schema)?;
        }

        // Validate output schema if present
        if let Some(ref schema) = ir.output_schema {
            self.validate_json_schema(schema)?;
        }

        Ok(())
    }

    /// Validate a JSON Schema structure
    fn validate_json_schema(&self, schema: &serde_json::Value) -> Result<(), AnalysisError> {
        // Basic validation - ensure it's an object
        if !schema.is_object() {
            return Err(AnalysisError::SchemaValidationFailed(
                "Schema must be a JSON object".to_string(),
            ));
        }

        // TODO: Add more comprehensive JSON Schema validation
        Ok(())
    }
}

impl Default for SchemaValidator {
    fn default() -> Self {
        Self::new()
    }
}
