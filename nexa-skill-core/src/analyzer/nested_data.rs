//! NestedDataDetector — Analyzer-phase detection of deeply nested data
//!
//! Traverses input_schema and output_schema in SkillIR, computes the
//! maximum nesting depth via `compute_nested_depth`, and sets:
//! - `requires_yaml_optimization` (true when depth >= threshold)
//! - `nested_data_depth` (the computed maximum depth)
//!
//! This replaces GeminiEmitter's local `is_deeply_nested()` method,
//! moving the decision to the Analyzer phase for all backends to reuse.

use crate::ir::{compute_nested_depth, SkillIR};

/// Default threshold for YAML optimization activation.
///
/// When nesting depth >= this value, YAML format should be preferred
/// for nested data rendering (academic basis: YAML 51.9% vs JSON 43.1%).
pub const DEFAULT_YAML_OPTIMIZATION_THRESHOLD: usize = 3;

/// Detector for deeply nested data structures in SkillIR schemas.
///
/// Runs during the Analyzer phase and populates `requires_yaml_optimization`
/// and `nested_data_depth` fields on SkillIR.
pub struct NestedDataDetector {
    /// Depth threshold at which YAML optimization is activated.
    threshold: usize,
}

impl NestedDataDetector {
    /// Create a detector with the default threshold (3).
    #[must_use]
    pub fn new() -> Self {
        Self {
            threshold: DEFAULT_YAML_OPTIMIZATION_THRESHOLD,
        }
    }

    /// Create a detector with a custom threshold.
    #[must_use]
    pub fn with_threshold(threshold: usize) -> Self {
        Self { threshold }
    }

    /// Detect nested data depth in SkillIR schemas and return a modified IR.
    ///
    /// Computes the maximum nesting depth across `input_schema` and
    /// `output_schema`, then sets:
    /// - `nested_data_depth` to the computed max depth
    /// - `requires_yaml_optimization` to `true` if depth >= threshold
    ///
    /// This is a pure transformation: receives `SkillIR`, returns `SkillIR`.
    /// Follows the Analyzer convention of functional-style passes.
    pub fn detect(&self, ir: SkillIR) -> SkillIR {
        let input_depth = ir
            .input_schema
            .as_ref()
            .map(compute_nested_depth)
            .unwrap_or(0);

        let output_depth = ir
            .output_schema
            .as_ref()
            .map(compute_nested_depth)
            .unwrap_or(0);

        let max_depth = input_depth.max(output_depth);

        SkillIR {
            requires_yaml_optimization: max_depth >= self.threshold,
            nested_data_depth: if max_depth > 0 { Some(max_depth) } else { None },
            ..ir
        }
    }
}

impl Default for NestedDataDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::sync::Arc;

    fn make_ir_with_schemas(
        input: Option<serde_json::Value>,
        output: Option<serde_json::Value>,
    ) -> SkillIR {
        SkillIR {
            name: Arc::from("test-skill"),
            version: Arc::from("1.0.0"),
            description: "test".to_string(),
            input_schema: input,
            output_schema: output,
            ..Default::default()
        }
    }

    #[test]
    fn test_no_schema_no_optimization() {
        let ir = make_ir_with_schemas(None, None);
        let result = NestedDataDetector::new().detect(ir);
        assert!(!result.requires_yaml_optimization);
        assert_eq!(result.nested_data_depth, None);
    }

    #[test]
    fn test_flat_schema_no_optimization() {
        let ir = make_ir_with_schemas(Some(json!({"a": 1})), None);
        let result = NestedDataDetector::new().detect(ir);
        assert!(!result.requires_yaml_optimization);
        assert_eq!(result.nested_data_depth, Some(1));
    }

    #[test]
    fn test_deeply_nested_triggers_optimization() {
        let ir = make_ir_with_schemas(Some(json!({"a": {"b": {"c": 1}}})), None);
        let result = NestedDataDetector::new().detect(ir);
        assert!(result.requires_yaml_optimization);
        assert_eq!(result.nested_data_depth, Some(3));
    }

    #[test]
    fn test_output_schema_depth_wins() {
        let ir = make_ir_with_schemas(
            Some(json!({"a": 1})), // depth 1
            Some(json!({"x": {"y": {"z": {"w": 1}}}})), // depth 4
        );
        let result = NestedDataDetector::new().detect(ir);
        assert!(result.requires_yaml_optimization);
        assert_eq!(result.nested_data_depth, Some(4));
    }

    #[test]
    fn test_custom_threshold() {
        let ir = make_ir_with_schemas(Some(json!({"a": {"b": {"c": 1}}})), None);
        // depth 3, threshold 5 → no optimization
        let result = NestedDataDetector::with_threshold(5).detect(ir);
        assert!(!result.requires_yaml_optimization);
        assert_eq!(result.nested_data_depth, Some(3));
    }

    #[test]
    fn test_preserves_other_ir_fields() {
        let ir = SkillIR {
            name: Arc::from("my-skill"),
            version: Arc::from("2.0.0"),
            description: "important skill".to_string(),
            hitl_required: true,
            input_schema: Some(json!({"a": {"b": {"c": 1}}})),
            ..Default::default()
        };
        let result = NestedDataDetector::new().detect(ir);
        assert_eq!(result.name, Arc::from("my-skill"));
        assert_eq!(result.version, Arc::from("2.0.0"));
        assert_eq!(result.description, "important skill");
        assert!(result.hitl_required);
        assert!(result.requires_yaml_optimization);
    }
}