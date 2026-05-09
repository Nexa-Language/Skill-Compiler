//! LLM Semantic Checker
//!
//! Optional compile-time semantic validation using an LLM (e.g., OpenAI API).
//! Validates that skill descriptions, procedures, and constraints are
//! semantically coherent and free of contradictions.
//!
//! This is an optional feature gated behind `CompilerConfig::semantic_check`.
//! When enabled, the checker sends the SkillIR content to an LLM for
//! semantic analysis and returns diagnostics for any issues found.

use crate::error::Diagnostic;
use crate::ir::SkillIR;
use serde::Deserialize;

/// Configuration for the LLM semantic checker
#[derive(Debug, Clone)]
pub struct SemanticCheckerConfig {
    /// OpenAI-compatible API key
    pub api_key: String,
    /// API base URL (default: https://api.openai.com/v1)
    pub api_base: String,
    /// Model to use (default: gpt-4o-mini)
    pub model: String,
    /// Maximum tokens for the response
    pub max_tokens: u32,
}

impl Default for SemanticCheckerConfig {
    fn default() -> Self {
        Self {
            api_key: String::new(),
            api_base: "https://api.openai.com/v1".to_string(),
            model: "gpt-4o-mini".to_string(),
            max_tokens: 512,
        }
    }
}

/// LLM Semantic Checker
///
/// Sends skill content to an LLM for semantic validation.
/// Returns diagnostics for issues like:
/// - Contradictory constraints
/// - Missing critical safety information
/// - Ambiguous or unclear procedure steps
/// - Inconsistent security level vs. declared operations
pub struct SemanticChecker {
    config: SemanticCheckerConfig,
}

/// Response from the LLM semantic check
#[derive(Debug, Deserialize)]
struct SemanticCheckResponse {
    choices: Vec<Choice>,
}

#[derive(Debug, Deserialize)]
struct Choice {
    message: Message,
}

#[derive(Debug, Deserialize)]
struct Message {
    content: String,
}

/// Structured issues returned by the LLM
#[derive(Debug, Deserialize)]
struct SemanticIssues {
    issues: Vec<SemanticIssue>,
}

#[derive(Debug, Deserialize)]
struct SemanticIssue {
    severity: String,
    category: String,
    description: String,
}

impl SemanticChecker {
    /// Create a new semantic checker with the given configuration
    #[must_use]
    pub fn new(config: SemanticCheckerConfig) -> Self {
        Self { config }
    }

    /// Check if the checker is properly configured (has an API key)
    #[must_use]
    pub fn is_configured(&self) -> bool {
        !self.config.api_key.is_empty()
    }

    /// Run semantic validation on a SkillIR
    ///
    /// Returns a list of diagnostics. If the LLM call fails (network error,
    /// rate limit, etc.), returns a warning diagnostic instead of failing
    /// the compilation — semantic check is advisory, not blocking.
    pub async fn check(&self, ir: &SkillIR) -> Vec<Diagnostic> {
        if !self.is_configured() {
            return vec![Diagnostic::warning(
                "semantic_check",
                "LLM semantic check enabled but no API key configured",
            )];
        }

        let prompt = self.build_prompt(ir);
        match self.call_llm(&prompt).await {
            Ok(issues) => self.issues_to_diagnostics(&issues),
            Err(e) => vec![Diagnostic::warning(
                "semantic_check",
                format!("LLM semantic check failed (non-blocking): {}", e),
            )],
        }
    }

    /// Build the prompt for semantic validation
    fn build_prompt(&self, ir: &SkillIR) -> String {
        let procedures_text: String = ir
            .procedures
            .iter()
            .map(|p| format!("{}. {}", p.order, p.instruction))
            .collect::<Vec<_>>()
            .join("\n");

        let constraints_text: String = ir
            .anti_skill_constraints
            .iter()
            .map(|c| format!("- [{}] {}", c.source, c.content))
            .collect::<Vec<_>>()
            .join("\n");

        format!(
            r#"You are a security auditor for AI agent skills. Analyze the following skill definition for semantic issues.

Skill Name: {}
Description: {}
Security Level: {}
HITL Required: {}

Procedures:
{}

Safety Constraints:
{}

Check for:
1. Contradictory constraints (e.g., one constraint says "always do X" but another implies "never do X")
2. Missing critical safety information (e.g., file operations without backup warnings)
3. Ambiguous procedure steps (e.g., "handle error" without specifying how)
4. Inconsistent security level vs. declared operations (e.g., "Low" security but destructive operations)

Respond with a JSON object containing an "issues" array. Each issue has:
- "severity": "error" or "warning"
- "category": one of "contradiction", "missing_safety", "ambiguity", "security_mismatch"
- "description": brief explanation

If no issues found, return: {{"issues": []}}

Respond ONLY with the JSON object, no other text."#,
            ir.name,
            ir.description,
            ir.security_level.to_string(),
            ir.hitl_required,
            procedures_text,
            constraints_text,
        )
    }

    /// Call the LLM API
    async fn call_llm(&self, prompt: &str) -> Result<Vec<SemanticIssue>, String> {
        let client = reqwest::Client::new();

        let body = serde_json::json!({
            "model": self.config.model,
            "messages": [
                {
                    "role": "system",
                    "content": "You are a security auditor. Respond only with valid JSON."
                },
                {
                    "role": "user",
                    "content": prompt
                }
            ],
            "max_tokens": self.config.max_tokens,
            "temperature": 0.0,
            "response_format": { "type": "json_object" }
        });

        let url = format!("{}/chat/completions", self.config.api_base.trim_end_matches('/'));

        let response = client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("HTTP request failed: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_body = response.text().await.unwrap_or_default();
            return Err(format!("API error {}: {}", status, error_body));
        }

        let check_response: SemanticCheckResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse API response: {}", e))?;

        let content = check_response
            .choices
            .first()
            .map(|c| c.message.content.clone())
            .unwrap_or_default();

        // Parse the JSON issues from the LLM response
        let issues: SemanticIssues = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse LLM response as JSON: {} — raw: {}", e, &content[..200.min(content.len())]))?;

        Ok(issues.issues)
    }

    /// Convert semantic issues to diagnostics
    fn issues_to_diagnostics(&self, issues: &[SemanticIssue]) -> Vec<Diagnostic> {
        issues
            .iter()
            .map(|issue| {
                let code = format!("semantic_{}", issue.category);
                let message = format!("[{}] {}", issue.severity.to_uppercase(), issue.description);
                match issue.severity.as_str() {
                    "error" => Diagnostic::error(&code, message),
                    _ => Diagnostic::warning(&code, message),
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::{ProcedureStep, SecurityLevel};
    use std::sync::Arc;

    fn make_test_ir() -> SkillIR {
        SkillIR {
            name: Arc::from("test-skill"),
            version: Arc::from("1.0.0"),
            description: "A test skill".to_string(),
            security_level: SecurityLevel::Medium,
            hitl_required: false,
            procedures: vec![ProcedureStep {
                order: 1,
                instruction: "Do something safely".to_string(),
                is_critical: false,
                constraints: vec![],
                expected_output: None,
                on_error: None,
            }],
            ..Default::default()
        }
    }

    #[test]
    fn test_not_configured_returns_warning() {
        let checker = SemanticChecker::new(SemanticCheckerConfig::default());
        assert!(!checker.is_configured());
    }

    #[test]
    fn test_configured_with_api_key() {
        let config = SemanticCheckerConfig {
            api_key: "test-key".to_string(),
            ..Default::default()
        };
        let checker = SemanticChecker::new(config);
        assert!(checker.is_configured());
    }

    #[test]
    fn test_build_prompt_contains_skill_info() {
        let config = SemanticCheckerConfig {
            api_key: "test-key".to_string(),
            ..Default::default()
        };
        let checker = SemanticChecker::new(config);
        let ir = make_test_ir();
        let prompt = checker.build_prompt(&ir);
        assert!(prompt.contains("test-skill"));
        assert!(prompt.contains("A test skill"));
        assert!(prompt.contains("Do something safely"));
    }

    #[test]
    fn test_issues_to_diagnostics() {
        let config = SemanticCheckerConfig::default();
        let checker = SemanticChecker::new(config);
        let issues = vec![
            SemanticIssue {
                severity: "warning".to_string(),
                category: "ambiguity".to_string(),
                description: "Step 1 is ambiguous".to_string(),
            },
            SemanticIssue {
                severity: "error".to_string(),
                category: "security_mismatch".to_string(),
                description: "Security level too low".to_string(),
            },
        ];
        let diagnostics = checker.issues_to_diagnostics(&issues);
        assert_eq!(diagnostics.len(), 2);
        assert!(diagnostics[0].code.contains("ambiguity"));
        assert!(diagnostics[1].code.contains("security_mismatch"));
    }
}