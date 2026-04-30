//! Procedure Step Definition
//!
//! Represents a single step in the standard operating procedure.

use serde::{Deserialize, Serialize};

/// A procedure step in the SOP
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcedureStep {
    /// Step order (1-based)
    pub order: u32,
    /// Step instruction text
    pub instruction: String,
    /// Whether this is a critical step
    #[serde(default)]
    pub is_critical: bool,
    /// Step-level constraints
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub constraints: Vec<String>,
    /// Expected output
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expected_output: Option<String>,
    /// Error handling strategy
    #[serde(skip_serializing_if = "Option::is_none")]
    pub on_error: Option<ErrorHandlingStrategy>,
}

/// Error handling strategy for a step
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ErrorHandlingStrategy {
    /// Stop the entire flow
    Stop,
    /// Skip this step and continue
    Skip,
    /// Retry with specified parameters
    Retry {
        /// Maximum retry attempts
        max_attempts: u32,
        /// Delay between retries in milliseconds
        delay_ms: u64,
    },
    /// Execute alternative step
    Fallback {
        /// Alternative step description
        alternative_step: String,
    },
    /// Request human intervention
    RequestHumanInput,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_procedure_step_construction() {
        let step = ProcedureStep {
            order: 1,
            instruction: "Do task".to_string(),
            is_critical: true,
            constraints: vec![],
            expected_output: None,
            on_error: None,
        };
        assert_eq!(step.order, 1);
        assert!(step.is_critical);
        assert!(step.constraints.is_empty());
        assert!(step.expected_output.is_none());
        assert!(step.on_error.is_none());
    }

    #[test]
    fn test_error_handling_strategy_stop() {
        let strategy = ErrorHandlingStrategy::Stop;
        let serialized = serde_json::to_string(&strategy).unwrap();
        assert!(serialized.contains("stop"));
    }

    #[test]
    fn test_error_handling_strategy_retry() {
        let strategy = ErrorHandlingStrategy::Retry {
            max_attempts: 3,
            delay_ms: 1000,
        };
        let serialized = serde_json::to_string(&strategy).unwrap();
        assert!(serialized.contains("retry"));
        assert!(serialized.contains("3"));
        assert!(serialized.contains("1000"));
    }
}
