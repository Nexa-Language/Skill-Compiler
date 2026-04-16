//! Example Definition
//!
//! Represents few-shot examples for skill execution.

use serde::{Deserialize, Serialize};

/// Few-shot example
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Example {
    /// Example title
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// User input
    pub user_input: String,
    /// Agent response
    pub agent_response: String,
    /// Tags for categorization
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    /// Difficulty level
    #[serde(skip_serializing_if = "Option::is_none")]
    pub difficulty: Option<ExampleDifficulty>,
}

/// Example difficulty level
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ExampleDifficulty {
    /// Basic example
    Basic,
    /// Intermediate example
    Intermediate,
    /// Advanced example
    Advanced,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_example_difficulty_serde() {
        // ExampleDifficulty uses serde rename_all = "lowercase"
        assert_eq!(serde_json::to_string(&ExampleDifficulty::Basic).unwrap(), "\"basic\"");
        assert_eq!(
            serde_json::to_string(&ExampleDifficulty::Intermediate).unwrap(),
            "\"intermediate\""
        );
        assert_eq!(serde_json::to_string(&ExampleDifficulty::Advanced).unwrap(), "\"advanced\"");
    }
}
