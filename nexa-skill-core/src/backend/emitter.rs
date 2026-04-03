//! Emitter Trait
//!
//! Defines the interface for all backend emitters.

use crate::analyzer::ValidatedSkillIR;
use crate::error::EmitError;
use async_trait::async_trait;

use super::TargetPlatform;

/// Emitter trait for code generation
#[async_trait]
pub trait Emitter: Send + Sync {
    /// Get the target platform
    fn target(&self) -> TargetPlatform;

    /// Emit the IR to a string
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
}
