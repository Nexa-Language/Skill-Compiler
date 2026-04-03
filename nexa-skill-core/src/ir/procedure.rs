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
