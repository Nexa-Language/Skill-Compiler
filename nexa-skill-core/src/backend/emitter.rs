//! Emitter Trait
//!
//! Defines the interface for all backend emitters.
//! Supports dual-payload output for format-tax optimization.

use crate::analyzer::ValidatedSkillIR;
use crate::error::EmitError;

use super::TargetPlatform;

/// Emitter trait for code generation
///
/// All emitters are synchronous — they perform pure string formatting
/// without any async I/O. The trait provides a lifecycle pipeline:
///
/// 1. `pre_process` — validate/transform IR before emission
/// 2. `emit` — generate the primary output string
/// 3. `post_process` — clean up or transform the emitted content
/// 4. `generate_assets` — produce additional sidecar files (schemas, configs)
pub trait Emitter: Send + Sync {
    /// Get the target platform
    fn target(&self) -> TargetPlatform;

    /// Emit the IR to a string (primary output)
    ///
    /// # Errors
    ///
    /// Returns an error if emission fails.
    fn emit(&self, ir: &ValidatedSkillIR) -> Result<String, EmitError>;

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

    /// Pre-process the IR before emission
    ///
    /// Default implementation does nothing. Override for custom pre-processing logic.
    fn pre_process(&self, _ir: &ValidatedSkillIR) -> Result<(), EmitError> {
        Ok(())
    }

    /// Post-process the emitted content
    ///
    /// Default implementation returns the content unchanged.
    /// Override for custom post-processing logic (e.g., output cleanup).
    fn post_process(&self, content: &str) -> Result<String, EmitError> {
        Ok(content.to_string())
    }
}