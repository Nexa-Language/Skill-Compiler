//! Emitter Trait
//!
//! Defines the interface for all backend emitters.
//! Supports dual-payload output for format-tax optimization.

use crate::analyzer::ValidatedSkillIR;
use crate::error::EmitError;
use async_trait::async_trait;

use super::TargetPlatform;

/// Emitter trait for code generation
#[async_trait]
pub trait Emitter: Send + Sync {
    /// Get the target platform
    fn target(&self) -> TargetPlatform;

    /// Emit the IR to a string (primary output)
    ///
    /// # Errors
    ///
    /// Returns an error if emission fails.
    async fn emit(&self, ir: &ValidatedSkillIR) -> Result<String, EmitError>;

    /// Get the output file extension
    fn file_extension(&self) -> &'static str {
        self.target().extension()
    }

    /// Whether a manifest is required
    fn requires_manifest(&self) -> bool {
        true
    }

    /// Generate additional asset files (e.g., schema files for dual-payload mode)
    /// Returns a list of (relative_path, content) tuples
    fn generate_assets(&self, _ir: &ValidatedSkillIR) -> Vec<(String, String)> {
        Vec::new()
    }
}