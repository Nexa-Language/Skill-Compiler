//! Schema Validator — 28-rule comprehensive validation
//!
//! Validates field formats, JSON Schema structures, cross-references,
//! MCP server names, and structural completeness of a [`SkillIR`].
//!
//! The validator collects **all** problems (Error + Warning) rather than
//! short-circuiting on the first error, returning a `Vec<Diagnostic>`.

use regex::Regex;

use crate::error::Diagnostic;
use crate::ir::{SecurityLevel, SkillIR};

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Schema validator — inspects a [`SkillIR`] and returns all diagnostics.
pub struct SchemaValidator;

impl SchemaValidator {
    /// Create a new schema validator.
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// Validate a [`SkillIR`], collecting all diagnostics (errors + warnings).
    ///
    /// Unlike the previous `Result<(), AnalysisError>` API, this method
    /// never short-circuits — every rule is evaluated so the caller gets
    /// a complete picture of all issues.
    #[must_use]
    pub fn validate(&self, ir: &SkillIR) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        diagnostics.extend(self.validate_field_formats(ir));
        diagnostics.extend(self.validate_json_schema_structure(
            ir.input_schema.as_ref(),
            "input_schema",
        ));
        diagnostics.extend(self.validate_json_schema_structure(
            ir.output_schema.as_ref(),
            "output_schema",
        ));
        diagnostics.extend(self.validate_cross_references(ir));
        diagnostics.extend(self.validate_mcp_names(ir));
        diagnostics.extend(self.validate_structural_completeness(ir));
        diagnostics
    }
}

impl Default for SchemaValidator {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Category 1: Field Format Validation (rules 1–10)
// ---------------------------------------------------------------------------

impl SchemaValidator {
    /// Rules 1–10: validate field-level format constraints.
    fn validate_field_formats(&self, ir: &SkillIR) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        // Rule 1: name 非空
        if ir.name.is_empty() {
            diagnostics.push(
                Diagnostic::error("Skill name must not be empty", "nsc::ir::invalid_name")
                    .with_help("Provide a non-empty kebab-case name in the frontmatter"),
            );
        } else {
            // Rule 2: name kebab-case (only when non-empty)
            if !SkillIR::is_valid_name(&ir.name) {
                diagnostics.push(
                    Diagnostic::error(
                        format!("Invalid name format: '{}'", ir.name),
                        "nsc::ir::invalid_name",
                    )
                    .with_help(
                        "Name must be kebab-case: lowercase a-z, digits 0-9, hyphens -. \
                         No leading/trailing hyphens, no consecutive hyphens. 1–64 characters.",
                    ),
                );
            }

            // Rule 3: name 长度 ≤ 64 (checked separately for clearer message)
            if ir.name.len() > 64 {
                diagnostics.push(
                    Diagnostic::error(
                        format!(
                            "Skill name too long: {} characters (max 64)",
                            ir.name.len()
                        ),
                        "nsc::ir::invalid_name",
                    )
                    .with_help("Shorten the name to at most 64 characters"),
                );
            }
        }

        // Rule 4: version 非空
        if ir.version.is_empty() {
            diagnostics.push(
                Diagnostic::warning("Version should not be empty", "nsc::ir::invalid_version")
                    .with_help("Provide a semantic version like '1.0.0'"),
            );
        } else {
            // Rule 5: version semver 格式
            let semver_re = Regex::new(r"^\d+\.\d+\.\d+(-[a-zA-Z0-9.\-]+)?$").unwrap();
            if !semver_re.is_match(&ir.version) {
                diagnostics.push(
                    Diagnostic::warning(
                        format!("Version '{}' is not valid semver (expected x.y.z format)", ir.version),
                        "nsc::ir::invalid_version",
                    )
                    .with_help("Use semantic versioning format: major.minor.patch, e.g. '1.0.0' or '1.0.0-alpha'"),
                );
            }
        }

        // Rule 6: description 非空
        if ir.description.is_empty() {
            diagnostics.push(
                Diagnostic::error("Description must not be empty", "nsc::ir::description_length")
                    .with_help("Add a description that explains what this skill does"),
            );
        } else {
            // Rule 7: description 长度 ≤ 1024
            if ir.description.len() > 1024 {
                diagnostics.push(
                    Diagnostic::error(
                        format!(
                            "Description too long: {} characters (max 1024)",
                            ir.description.len()
                        ),
                        "nsc::ir::description_length",
                    )
                    .with_help("Shorten the description to at most 1024 characters"),
                );
            }

            // Rule 8: description XML 标签检测
            let xml_re = Regex::new(r"<[a-zA-Z][a-zA-Z0-9\-]*[^>]*>").unwrap();
            if xml_re.is_match(&ir.description) {
                diagnostics.push(
                    Diagnostic::error(
                        "Description contains XML-like tags that may interfere with LLM parsing",
                        "nsc::ir::description_xml_tags",
                    )
                    .with_help(
                        "Remove XML tags from description — they interfere with LLM parsing. \
                         Use plain text or Markdown formatting instead.",
                    ),
                );
            }
        }

        // Rule 9: security_level 有效枚举 (defensive — serde already guarantees this)
        // We still emit a warning if somehow an unrecognized value slips through.
        // Since SecurityLevel is a Rust enum, this can only happen via serde skip.
        // We validate by checking it's one of the known variants.
        match ir.security_level {
            SecurityLevel::Low | SecurityLevel::Medium | SecurityLevel::High | SecurityLevel::Critical => {}
            // No unknown variants possible in current enum definition, but this
            // serves as a defensive checkpoint if new variants are added.
        }

        // Rule 10: procedures 非空 (warning — not blocking)
        // Many real-world skills lack explicit ## Procedures sections but still
        // contain valuable instruction content in other sections (Instructions,
        // How to Use, Core Workflow Pattern, etc.). Downgraded from error to
        // warning so compilation can proceed.
        if ir.procedures.is_empty() {
            diagnostics.push(
                Diagnostic::warning(
                    "No procedure steps found — compiled output will lack structured execution steps",
                    "nsc::ir::missing_procedures",
                )
                .with_help("Add a ## Procedures section with numbered steps, or use ## Instructions / ## How to Use / ## Core Workflow Pattern sections that NSC can extract steps from"),
            );
        }

        diagnostics
    }
}

// ---------------------------------------------------------------------------
// Category 2: JSON Schema Structure Validation (rules 11–20)
// ---------------------------------------------------------------------------

impl SchemaValidator {
    /// Rules 11–15 (input) and 16–20 (output): validate JSON Schema structure.
    ///
    /// `label` is either `"input_schema"` or `"output_schema"` for diagnostics.
    fn validate_json_schema_structure(
        &self,
        schema: Option<&serde_json::Value>,
        label: &str,
    ) -> Vec<Diagnostic> {
        let Some(schema) = schema else {
            // No schema present — that's valid (schemas are optional)
            return Vec::new();
        };

        let mut diagnostics = Vec::new();

        // Rule 11/16: schema 必须是 JSON object
        if !schema.is_object() {
            diagnostics.push(
                Diagnostic::error(
                    format!("{} must be a JSON object, found {}", label, json_type_name(schema)),
                    "nsc::ir::invalid_schema",
                )
                .with_help(format!("Wrap {} in a JSON object with a \"type\" key", label)),
            );
            // If it's not an object, remaining rules don't apply
            return diagnostics;
        }

        let obj = schema.as_object().unwrap();

        // Rule 12/17: schema 必须有 "type" 字段
        if !obj.contains_key("type") {
            diagnostics.push(
                Diagnostic::error(
                    format!("{} must contain a \"type\" field", label),
                    "nsc::ir::invalid_schema",
                )
                .with_help("Add a \"type\" field, e.g. \"type\": \"object\""),
            );
        } else {
            // If type is "object", check for properties
            let type_val = obj.get("type");
            if let Some(serde_json::Value::String(type_str)) = type_val {
                if type_str == "object" {
                    // Rule 13/18: type=object 时必须有 "properties"
                    if !obj.contains_key("properties") {
                        diagnostics.push(
                            Diagnostic::warning(
                                format!(
                                    "{} declares type=\"object\" but has no \"properties\" field",
                                    label
                                ),
                                "nsc::ir::invalid_schema",
                            )
                            .with_help("Add a \"properties\" field defining the object's keys"),
                        );
                    } else {
                        // Rule 15/20: "properties" 的值必须有 "type"
                        if let Some(serde_json::Value::Object(props)) = obj.get("properties") {
                            for (prop_name, prop_val) in props {
                                if !prop_val.is_object() {
                                    diagnostics.push(
                                        Diagnostic::warning(
                                            format!(
                                                "{} property '{}' must be a JSON object with a \"type\" field",
                                                label, prop_name
                                            ),
                                            "nsc::ir::invalid_schema",
                                        )
                                        .with_help(format!(
                                            "Define property '{}' as an object, e.g. {{\"type\": \"string\"}}",
                                            prop_name
                                        )),
                                    );
                                } else if !prop_val.as_object().unwrap().contains_key("type") {
                                    diagnostics.push(
                                        Diagnostic::warning(
                                            format!(
                                                "{} property '{}' is missing a \"type\" field",
                                                label, prop_name
                                            ),
                                            "nsc::ir::invalid_schema",
                                        )
                                        .with_help(format!(
                                            "Add a \"type\" field to property '{}'",
                                            prop_name
                                        )),
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }

        // Rule 14/19: "required" 必须是字符串数组
        if let Some(required_val) = obj.get("required") {
            if !required_val.is_array() {
                diagnostics.push(
                    Diagnostic::warning(
                        format!("{} \"required\" field must be an array of strings", label),
                        "nsc::ir::invalid_schema",
                    )
                    .with_help("Change \"required\" to an array of property name strings"),
                );
            } else {
                let arr = required_val.as_array().unwrap();
                for item in arr {
                    if !item.is_string() {
                        diagnostics.push(
                            Diagnostic::warning(
                                format!(
                                    "{} \"required\" array contains a non-string value: {}",
                                    label, item
                                ),
                                "nsc::ir::invalid_schema",
                            )
                            .with_help("Ensure all items in \"required\" are strings"),
                        );
                    }
                }
            }
        }

        diagnostics
    }
}

// ---------------------------------------------------------------------------
// Category 3: Cross-Validation (rules 21–24)
// ---------------------------------------------------------------------------

impl SchemaValidator {
    /// Rules 21–24: validate cross-references between different IR fields.
    fn validate_cross_references(&self, ir: &SkillIR) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        // Rule 21: Schema-Example 参数一致性
        diagnostics.extend(self.validate_schema_example_consistency(ir));

        // Rule 22: HITL-SecurityLevel 一致性
        if ir.security_level == SecurityLevel::Critical && !ir.hitl_required {
            diagnostics.push(
                Diagnostic::error(
                    "Critical security level requires hitl_required=true",
                    "nsc::analysis::security_mismatch",
                )
                .with_help("Set hitl_required to true for Critical-level skills"),
            );
        }
        if ir.security_level == SecurityLevel::High && !ir.hitl_required {
            diagnostics.push(
                Diagnostic::error(
                    "High security level requires hitl_required=true",
                    "nsc::analysis::security_mismatch",
                )
                .with_help("Set hitl_required to true for High-level skills"),
            );
        }

        // Rule 23: Procedure 步骤顺序编号
        if !ir.procedures.is_empty() {
            let orders: Vec<u32> = ir.procedures.iter().map(|s| s.order).collect();
            for (idx, expected) in (1u32..).enumerate() {
                if idx >= orders.len() {
                    break;
                }
                if orders[idx] != expected {
                    diagnostics.push(
                        Diagnostic::warning(
                            format!(
                                "Procedure step order is not sequential: expected step {} to have order {}, found order {}",
                                idx + 1, expected, orders[idx]
                            ),
                            "nsc::ir::missing_procedures",
                        )
                        .with_help("Number procedure steps consecutively starting from 1"),
                    );
                    break; // One warning is enough for ordering issues
                }
            }
        }

        // Rule 24: Procedure 关键步骤检查
        if !ir.procedures.is_empty()
            && (ir.security_level == SecurityLevel::High || ir.security_level == SecurityLevel::Critical)
        {
            let has_critical = ir.procedures.iter().any(|s| s.is_critical);
            if !has_critical {
                diagnostics.push(
                    Diagnostic::warning(
                        format!(
                            "Security level '{}' requires at least one critical procedure step",
                            ir.security_level
                        ),
                        "nsc::ir::missing_procedures",
                    )
                    .with_help("Mark at least one procedure step as is_critical=true"),
                );
            }
        }

        diagnostics
    }

    /// Extract `{{param}}` references from example `user_input` and compare
    /// against `input_schema.properties`.
    fn validate_schema_example_consistency(&self, ir: &SkillIR) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        // Collect declared parameter names from input_schema
        let declared_params: Vec<&str> = ir
            .input_schema
            .as_ref()
            .and_then(|s| s.get("properties"))
            .and_then(|p| p.as_object())
            .map(|obj| obj.keys().map(|k| k.as_str()).collect())
            .unwrap_or_default();

        // Extract {{param}} references from each example's user_input
        let param_ref_re = Regex::new(r"\{\{([^}]+)\}\}").unwrap();
        for example in &ir.few_shot_examples {
            for cap in param_ref_re.captures_iter(&example.user_input) {
                let param_name = cap[1].trim();
                if !declared_params.contains(&param_name) {
                    diagnostics.push(
                        Diagnostic::warning(
                            format!(
                                "Example references parameter '{}' not defined in input_schema.properties",
                                param_name
                            ),
                            "nsc::analysis::schema_example_mismatch",
                        )
                        .with_help(format!(
                            "Add property '{}' to input_schema, or remove the reference from the example",
                            param_name
                        )),
                    );
                }
            }
        }

        diagnostics
    }
}

// ---------------------------------------------------------------------------
// Category 4: MCP Server Name Format (rules 25–26)
// ---------------------------------------------------------------------------

impl SchemaValidator {
    /// Rules 25–26: validate MCP server name format.
    fn validate_mcp_names(&self, ir: &SkillIR) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        for server_name in &ir.mcp_servers {
            // Rule 25: MCP server name 非空
            if server_name.is_empty() {
                diagnostics.push(
                    Diagnostic::error(
                        "MCP server name must not be empty",
                        "nsc::ir::mcp_name_format",
                    )
                    .with_help("Provide a non-empty name for each MCP server dependency"),
                );
            } else {
                // Rule 26: MCP server name kebab-case
                if !is_valid_kebab_case(server_name) {
                    diagnostics.push(
                        Diagnostic::warning(
                            format!(
                                "MCP server name '{}' is not in kebab-case format",
                                server_name
                            ),
                            "nsc::ir::mcp_name_format",
                        )
                        .with_help(
                            "Use kebab-case for MCP server names: lowercase a-z, digits 0-9, hyphens",
                        ),
                    );
                }
            }
        }

        diagnostics
    }
}

// ---------------------------------------------------------------------------
// Category 5: Structural Completeness (rules 27–28)
// ---------------------------------------------------------------------------

impl SchemaValidator {
    /// Rules 27–28: validate structural completeness of conditions and fallbacks.
    fn validate_structural_completeness(&self, ir: &SkillIR) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        // Rule 27: pre_conditions / post_conditions 非空字符串
        for cond in &ir.pre_conditions {
            if cond.trim().is_empty() {
                diagnostics.push(
                    Diagnostic::warning(
                        "Pre-condition is an empty string",
                        "nsc::ir::empty_condition",
                    )
                    .with_help("Provide a meaningful pre-condition or remove the empty entry"),
                );
            }
        }
        for cond in &ir.post_conditions {
            if cond.trim().is_empty() {
                diagnostics.push(
                    Diagnostic::warning(
                        "Post-condition is an empty string",
                        "nsc::ir::empty_condition",
                    )
                    .with_help("Provide a meaningful post-condition or remove the empty entry"),
                );
            }
        }

        // Rule 28: fallbacks 非空字符串
        for fallback in &ir.fallbacks {
            if fallback.trim().is_empty() {
                diagnostics.push(
                    Diagnostic::warning(
                        "Fallback strategy is an empty string",
                        "nsc::ir::empty_fallback",
                    )
                    .with_help("Provide a meaningful fallback strategy or remove the empty entry"),
                );
            }
        }

        diagnostics
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Check whether a string is valid kebab-case:
/// lowercase a-z, digits 0-9, hyphens; no leading/trailing/double hyphens.
fn is_valid_kebab_case(s: &str) -> bool {
    !s.is_empty()
        && s.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
        && !s.starts_with('-')
        && !s.ends_with('-')
        && !s.contains("--")
}

/// Return a human-readable name for a JSON value type.
fn json_type_name(val: &serde_json::Value) -> &'static str {
    match val {
        serde_json::Value::Null => "null",
        serde_json::Value::Bool(_) => "boolean",
        serde_json::Value::Number(_) => "number",
        serde_json::Value::String(_) => "string",
        serde_json::Value::Array(_) => "array",
        serde_json::Value::Object(_) => "object",
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use serde_json::json;

    use crate::error::ErrorLevel;
    use crate::ir::{Example, ProcedureStep, SecurityLevel, SkillIR};

    use super::SchemaValidator;

    /// Helper: build a fully valid SkillIR for baseline tests.
    fn valid_ir() -> SkillIR {
        SkillIR {
            name: Arc::from("test-skill"),
            version: Arc::from("1.0.0"),
            description: "A valid test skill description.".to_string(),
            procedures: vec![ProcedureStep {
                order: 1,
                instruction: "Do something".to_string(),
                is_critical: true,
                constraints: Vec::new(),
                expected_output: None,
                on_error: None,
            }],
            security_level: SecurityLevel::Medium,
            hitl_required: false,
            input_schema: Some(json!({
                "type": "object",
                "properties": {
                    "query": { "type": "string" }
                },
                "required": ["query"]
            })),
            output_schema: Some(json!({
                "type": "object",
                "properties": {
                    "result": { "type": "string" }
                }
            })),
            ..Default::default()
        }
    }

    /// Helper: check that at least one diagnostic with the given code and level exists.
    fn has_diagnostic(diags: &[crate::error::Diagnostic], code: &str, level: ErrorLevel) -> bool {
        diags.iter().any(|d| d.code == code && d.level == level)
    }

    // --- Test 1: valid_ir → empty Vec ---
    #[test]
    fn valid_ir_returns_no_diagnostics() {
        let validator = SchemaValidator::new();
        let diags = validator.validate(&valid_ir());
        assert!(diags.is_empty(), "Expected no diagnostics for a valid IR, got: {diags:?}");
    }

    // --- Test 2: empty_name → Error ---
    #[test]
    fn empty_name_produces_error() {
        let ir = SkillIR {
            name: Arc::from(""),
            ..valid_ir()
        };
        let diags = SchemaValidator::new().validate(&ir);
        assert!(has_diagnostic(&diags, "nsc::ir::invalid_name", ErrorLevel::Error));
    }

    // --- Test 3: invalid_name_format → Error ---
    #[test]
    fn invalid_name_format_produces_error() {
        let ir = SkillIR {
            name: Arc::from("Invalid Name"),
            ..valid_ir()
        };
        let diags = SchemaValidator::new().validate(&ir);
        assert!(has_diagnostic(&diags, "nsc::ir::invalid_name", ErrorLevel::Error));
    }

    // --- Test 4: name_too_long → Error ---
    #[test]
    fn name_too_long_produces_error() {
        let long_name = "a".repeat(65);
        let ir = SkillIR {
            name: Arc::from(long_name.as_str()),
            ..valid_ir()
        };
        let diags = SchemaValidator::new().validate(&ir);
        assert!(has_diagnostic(&diags, "nsc::ir::invalid_name", ErrorLevel::Error));
    }

    // --- Test 5: empty_version → Warning ---
    #[test]
    fn empty_version_produces_warning() {
        let ir = SkillIR {
            version: Arc::from(""),
            ..valid_ir()
        };
        let diags = SchemaValidator::new().validate(&ir);
        assert!(has_diagnostic(&diags, "nsc::ir::invalid_version", ErrorLevel::Warning));
    }

    // --- Test 6: invalid_version → Warning ---
    #[test]
    fn invalid_version_produces_warning() {
        let ir = SkillIR {
            version: Arc::from("v1-beta"),
            ..valid_ir()
        };
        let diags = SchemaValidator::new().validate(&ir);
        assert!(has_diagnostic(&diags, "nsc::ir::invalid_version", ErrorLevel::Warning));
    }

    // --- Test 7: empty_description → Error ---
    #[test]
    fn empty_description_produces_error() {
        let ir = SkillIR {
            description: String::new(),
            ..valid_ir()
        };
        let diags = SchemaValidator::new().validate(&ir);
        assert!(has_diagnostic(&diags, "nsc::ir::description_length", ErrorLevel::Error));
    }

    // --- Test 8: description_too_long → Error ---
    #[test]
    fn description_too_long_produces_error() {
        let ir = SkillIR {
            description: "x".repeat(1025),
            ..valid_ir()
        };
        let diags = SchemaValidator::new().validate(&ir);
        assert!(has_diagnostic(&diags, "nsc::ir::description_length", ErrorLevel::Error));
    }

    // --- Test 9: description_with_xml_tags → Error ---
    #[test]
    fn description_with_xml_tags_produces_error() {
        let ir = SkillIR {
            description: "This skill <warning>may cause issues</warning>.".to_string(),
            ..valid_ir()
        };
        let diags = SchemaValidator::new().validate(&ir);
        assert!(has_diagnostic(&diags, "nsc::ir::description_xml_tags", ErrorLevel::Error));
    }

    // --- Test 10: empty_procedures → Warning (downgraded from Error) ---
    // Procedures are recommended but not mandatory — real-world skills may lack
    // explicit ## Procedures sections but still compile successfully.
    #[test]
    fn empty_procedures_produces_warning() {
        let ir = SkillIR {
            procedures: Vec::new(),
            ..valid_ir()
        };
        let diags = SchemaValidator::new().validate(&ir);
        assert!(has_diagnostic(&diags, "nsc::ir::missing_procedures", ErrorLevel::Warning));
    }

    // --- Test 11: invalid_input_schema_type → Error ---
    #[test]
    fn invalid_input_schema_type_produces_error() {
        let ir = SkillIR {
            input_schema: Some(json!([1, 2, 3])),
            ..valid_ir()
        };
        let diags = SchemaValidator::new().validate(&ir);
        assert!(has_diagnostic(&diags, "nsc::ir::invalid_schema", ErrorLevel::Error));
    }

    // --- Test 12: input_schema_missing_type → Error ---
    #[test]
    fn input_schema_missing_type_produces_error() {
        let ir = SkillIR {
            input_schema: Some(json!({"properties": {}})),
            ..valid_ir()
        };
        let diags = SchemaValidator::new().validate(&ir);
        assert!(has_diagnostic(&diags, "nsc::ir::invalid_schema", ErrorLevel::Error));
    }

    // --- Test 13: input_schema_object_no_properties → Warning ---
    #[test]
    fn input_schema_object_no_properties_produces_warning() {
        let ir = SkillIR {
            input_schema: Some(json!({"type": "object"})),
            ..valid_ir()
        };
        let diags = SchemaValidator::new().validate(&ir);
        assert!(has_diagnostic(&diags, "nsc::ir::invalid_schema", ErrorLevel::Warning));
    }

    // --- Test 14: output_schema_validation → same rules ---
    #[test]
    fn output_schema_object_no_properties_produces_warning() {
        let ir = SkillIR {
            output_schema: Some(json!({"type": "object"})),
            ..valid_ir()
        };
        let diags = SchemaValidator::new().validate(&ir);
        assert!(has_diagnostic(&diags, "nsc::ir::invalid_schema", ErrorLevel::Warning));
    }

    // --- Test 15: schema_example_mismatch → Warning ---
    #[test]
    fn schema_example_mismatch_produces_warning() {
        let ir = SkillIR {
            few_shot_examples: vec![Example {
                title: None,
                user_input: "Search for {{undefined_param}}".to_string(),
                agent_response: "Result".to_string(),
                tags: Vec::new(),
                difficulty: None,
            }],
            ..valid_ir()
        };
        let diags = SchemaValidator::new().validate(&ir);
        assert!(has_diagnostic(&diags, "nsc::analysis::schema_example_mismatch", ErrorLevel::Warning));
    }

    // --- Test 16: hitl_security_mismatch (Critical) → Error ---
    #[test]
    fn hitl_security_mismatch_critical_produces_error() {
        let ir = SkillIR {
            security_level: SecurityLevel::Critical,
            hitl_required: false,
            ..valid_ir()
        };
        let diags = SchemaValidator::new().validate(&ir);
        assert!(has_diagnostic(&diags, "nsc::analysis::security_mismatch", ErrorLevel::Error));
    }

    // --- Test 16b: hitl_security_mismatch (High) → Error ---
    #[test]
    fn hitl_security_mismatch_high_produces_error() {
        let ir = SkillIR {
            security_level: SecurityLevel::High,
            hitl_required: false,
            ..valid_ir()
        };
        let diags = SchemaValidator::new().validate(&ir);
        assert!(has_diagnostic(&diags, "nsc::analysis::security_mismatch", ErrorLevel::Error));
    }

    // --- Test 17: procedure_ordering → Warning ---
    #[test]
    fn procedure_ordering_produces_warning() {
        let ir = SkillIR {
            procedures: vec![
                ProcedureStep {
                    order: 1,
                    instruction: "Step one".to_string(),
                    is_critical: false,
                    constraints: Vec::new(),
                    expected_output: None,
                    on_error: None,
                },
                ProcedureStep {
                    order: 3, // gap: expected 2
                    instruction: "Step three".to_string(),
                    is_critical: false,
                    constraints: Vec::new(),
                    expected_output: None,
                    on_error: None,
                },
            ],
            ..valid_ir()
        };
        let diags = SchemaValidator::new().validate(&ir);
        assert!(has_diagnostic(&diags, "nsc::ir::missing_procedures", ErrorLevel::Warning));
    }

    // --- Test 18: critical_step_check → Warning ---
    #[test]
    fn critical_step_check_produces_warning() {
        let ir = SkillIR {
            security_level: SecurityLevel::High,
            hitl_required: true,
            procedures: vec![ProcedureStep {
                order: 1,
                instruction: "Non-critical step".to_string(),
                is_critical: false,
                constraints: Vec::new(),
                expected_output: None,
                on_error: None,
            }],
            ..valid_ir()
        };
        let diags = SchemaValidator::new().validate(&ir);
        assert!(has_diagnostic(&diags, "nsc::ir::missing_procedures", ErrorLevel::Warning));
    }

    // --- Test 19: mcp_empty_name → Error ---
    #[test]
    fn mcp_empty_name_produces_error() {
        let ir = SkillIR {
            mcp_servers: vec![Arc::from("")],
            ..valid_ir()
        };
        let diags = SchemaValidator::new().validate(&ir);
        assert!(has_diagnostic(&diags, "nsc::ir::mcp_name_format", ErrorLevel::Error));
    }

    // --- Test 20: mcp_invalid_format → Warning ---
    #[test]
    fn mcp_invalid_format_produces_warning() {
        let ir = SkillIR {
            mcp_servers: vec![Arc::from("My_Server")],
            ..valid_ir()
        };
        let diags = SchemaValidator::new().validate(&ir);
        assert!(has_diagnostic(&diags, "nsc::ir::mcp_name_format", ErrorLevel::Warning));
    }

    // --- Test 21: empty_condition → Warning ---
    #[test]
    fn empty_condition_produces_warning() {
        let ir = SkillIR {
            pre_conditions: vec![String::new()],
            post_conditions: vec!["   ".to_string()], // whitespace-only also triggers
            ..valid_ir()
        };
        let diags = SchemaValidator::new().validate(&ir);
        // At least two warnings: one for pre_condition, one for post_condition
        let cond_diags = diags
            .iter()
            .filter(|d| d.code == "nsc::ir::empty_condition")
            .count();
        assert!(cond_diags >= 2, "Expected >=2 empty_condition warnings, got {cond_diags}");
    }

    // --- Test 22: empty_fallback → Warning ---
    #[test]
    fn empty_fallback_produces_warning() {
        let ir = SkillIR {
            fallbacks: vec![String::new()],
            ..valid_ir()
        };
        let diags = SchemaValidator::new().validate(&ir);
        assert!(has_diagnostic(&diags, "nsc::ir::empty_fallback", ErrorLevel::Warning));
    }

    // --- Test 23: multiple_errors → does not short-circuit ---
    #[test]
    fn multiple_errors_all_collected() {
        let ir = SkillIR {
            name: Arc::from(""),          // Rule 1: empty name → Error
            description: String::new(),   // Rule 6: empty description → Error
            version: Arc::from(""),        // Rule 4: empty version → Warning
            procedures: Vec::new(),        // Rule 10: empty procedures → Warning (downgraded)
            ..Default::default()
        };
        let diags = SchemaValidator::new().validate(&ir);
        // Must have at least 2 Errors and 2 Warnings (procedures now Warning)
        let errors = diags.iter().filter(|d| d.is_error()).count();
        let warnings = diags.iter().filter(|d| d.is_warning()).count();
        assert!(errors >= 2, "Expected >=2 errors, got {errors}");
        assert!(warnings >= 2, "Expected >=2 warnings, got {warnings}");
    }

    // --- Test 24: valid_name_kebab → no name-related diagnostic ---
    #[test]
    fn valid_name_kebab_no_diagnostic() {
        let ir = SkillIR {
            name: Arc::from("my-cool-skill"),
            ..valid_ir()
        };
        let diags = SchemaValidator::new().validate(&ir);
        let name_diags = diags
            .iter()
            .filter(|d| d.code == "nsc::ir::invalid_name")
            .count();
        assert_eq!(name_diags, 0, "Expected no name diagnostics for valid kebab-case name");
    }

    // --- Additional: properties missing type → Warning ---
    #[test]
    fn schema_property_missing_type_produces_warning() {
        let ir = SkillIR {
            input_schema: Some(json!({
                "type": "object",
                "properties": {
                    "query": { "description": "A query string" }
                }
            })),
            ..valid_ir()
        };
        let diags = SchemaValidator::new().validate(&ir);
        assert!(has_diagnostic(&diags, "nsc::ir::invalid_schema", ErrorLevel::Warning));
    }

    // --- Additional: required field with non-string items → Warning ---
    #[test]
    fn schema_required_non_string_produces_warning() {
        let ir = SkillIR {
            input_schema: Some(json!({
                "type": "object",
                "properties": {
                    "query": { "type": "string" }
                },
                "required": [42]
            })),
            ..valid_ir()
        };
        let diags = SchemaValidator::new().validate(&ir);
        assert!(has_diagnostic(&diags, "nsc::ir::invalid_schema", ErrorLevel::Warning));
    }
}