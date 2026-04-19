//! Emitter Registry
//!
//! Manages available emitter instances and provides lookup by target platform.

use std::collections::HashMap;

use crate::analyzer::ValidatedSkillIR;
use crate::error::EmitError;

use super::{Emitter, TargetPlatform};

/// Emitter registry
///
/// Manages all available Emitter instances and provides
/// lookup by target platform, along with convenience methods
/// for the full emission pipeline (pre_process → emit → post_process).
pub struct EmitterRegistry {
    emitters: HashMap<TargetPlatform, Box<dyn Emitter>>,
}

impl EmitterRegistry {
    /// Create a registry with default emitters
    ///
    /// Registers built-in emitters for Claude, Codex, Gemini, and Kimi.
    pub fn new() -> Self {
        let mut emitters: HashMap<TargetPlatform, Box<dyn Emitter>> = HashMap::new();
        emitters.insert(TargetPlatform::Claude, Box::new(super::ClaudeEmitter::new()));
        emitters.insert(TargetPlatform::Codex, Box::new(super::CodexEmitter::new()));
        emitters.insert(TargetPlatform::Gemini, Box::new(super::GeminiEmitter::new()));
        emitters.insert(TargetPlatform::Kimi, Box::new(super::KimiEmitter::new()));
        Self { emitters }
    }

    /// Register a custom emitter
    ///
    /// If an emitter for the same target platform already exists,
    /// it will be replaced.
    pub fn register(&mut self, emitter: Box<dyn Emitter>) {
        self.emitters.insert(emitter.target(), emitter);
    }

    /// Get an emitter for a specific target platform
    ///
    /// # Errors
    ///
    /// Returns `EmitError::UnsupportedTarget` if no emitter is registered
    /// for the given platform.
    pub fn get(&self, target: &TargetPlatform) -> Result<&dyn Emitter, EmitError> {
        self.emitters
            .get(target)
            .map(|e| e.as_ref())
            .ok_or_else(|| EmitError::UnsupportedTarget(target.display_name().to_string()))
    }

    /// Get all supported platforms
    pub fn supported_platforms(&self) -> Vec<TargetPlatform> {
        self.emitters.keys().cloned().collect()
    }

    /// Emit for a target using the registered emitter
    ///
    /// Convenience method that calls pre_process, emit, and post_process in sequence.
    pub fn emit_for_target(
        &self,
        target: &TargetPlatform,
        ir: &ValidatedSkillIR,
    ) -> Result<String, EmitError> {
        let emitter = self.get(target)?;
        emitter.pre_process(ir)?;
        let content = emitter.emit(ir)?;
        emitter.post_process(&content)
    }

    /// Generate assets for a target using the registered emitter
    pub fn assets_for_target(
        &self,
        target: &TargetPlatform,
        ir: &ValidatedSkillIR,
    ) -> Result<Vec<(String, String)>, EmitError> {
        let emitter = self.get(target)?;
        Ok(emitter.generate_assets(ir))
    }
}

impl Default for EmitterRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::SkillIR;

    /// A mock emitter for testing registry operations
    struct MockEmitter {
        target_platform: TargetPlatform,
    }

    impl MockEmitter {
        fn new(target: TargetPlatform) -> Self {
            Self { target_platform: target }
        }
    }

    impl Emitter for MockEmitter {
        fn target(&self) -> TargetPlatform {
            self.target_platform
        }

        fn emit(&self, ir: &ValidatedSkillIR) -> Result<String, EmitError> {
            Ok(format!("mock-output: {}", ir.as_ref().name))
        }
    }

    #[test]
    fn default_registry_has_all_platforms() {
        let registry = EmitterRegistry::new();
        assert!(registry.get(&TargetPlatform::Claude).is_ok());
        assert!(registry.get(&TargetPlatform::Codex).is_ok());
        assert!(registry.get(&TargetPlatform::Gemini).is_ok());
        assert!(registry.get(&TargetPlatform::Kimi).is_ok());
    }

    #[test]
    fn get_claude_emitter() {
        let registry = EmitterRegistry::new();
        let emitter = registry.get(&TargetPlatform::Claude);
        assert!(emitter.is_ok());
        assert_eq!(emitter.unwrap().target(), TargetPlatform::Claude);
    }

    #[test]
    fn get_unknown_platform() {
        // EmitterRegistry doesn't have a "Custom" platform by default
        // We test that get() returns UnsupportedTarget for a platform
        // that has no registered emitter
        let mut registry = EmitterRegistry::new();
        // Remove all emitters to test unsupported target
        registry.emitters.clear();
        let result = registry.get(&TargetPlatform::Claude);
        assert!(result.is_err());
        match result.err().unwrap() {
            EmitError::UnsupportedTarget(name) => {
                assert_eq!(name, "Claude Code");
            }
            _ => panic!("Expected UnsupportedTarget error"),
        }
    }

    #[test]
    fn register_custom_emitter() {
        let mut registry = EmitterRegistry::new();
        // Replace Claude emitter with a mock
        let mock = MockEmitter::new(TargetPlatform::Claude);
        registry.register(Box::new(mock));

        let emitter = registry.get(&TargetPlatform::Claude).unwrap();
        // Verify it's our mock by checking emit output
        let ir = ValidatedSkillIR::new(SkillIR::default(), vec![]);
        let output = emitter.emit(&ir).unwrap();
        assert!(output.contains("mock-output"));
    }

    #[test]
    fn supported_platforms() {
        let registry = EmitterRegistry::new();
        let platforms = registry.supported_platforms();
        assert_eq!(platforms.len(), 4);
        assert!(platforms.contains(&TargetPlatform::Claude));
        assert!(platforms.contains(&TargetPlatform::Codex));
        assert!(platforms.contains(&TargetPlatform::Gemini));
        assert!(platforms.contains(&TargetPlatform::Kimi));
    }

    #[test]
    fn emit_for_target() {
        let registry = EmitterRegistry::new();
        let ir = ValidatedSkillIR::new(SkillIR::default(), vec![]);
        let result = registry.emit_for_target(&TargetPlatform::Claude, &ir);
        assert!(result.is_ok());
        // Claude emitter produces XML output
        assert!(result.unwrap().contains("<agent_skill>"));
    }

    #[test]
    fn fixed_output_filename_claude_and_kimi() {
        // Claude and Kimi must return SKILL.md (per Agent Skills spec)
        assert_eq!(TargetPlatform::Claude.fixed_output_filename(), Some("SKILL.md"));
        assert_eq!(TargetPlatform::Kimi.fixed_output_filename(), Some("SKILL.md"));
        // Codex and Gemini use skill-name-based filenames (no fixed name)
        assert_eq!(TargetPlatform::Codex.fixed_output_filename(), None);
        assert_eq!(TargetPlatform::Gemini.fixed_output_filename(), None);
    }

    #[test]
    fn replace_emitter() {
        let mut registry = EmitterRegistry::new();

        // First emit with default Claude emitter
        let ir = ValidatedSkillIR::new(SkillIR::default(), vec![]);
        let original_output = registry
            .emit_for_target(&TargetPlatform::Claude, &ir)
            .unwrap();
        assert!(original_output.contains("<agent_skill>"));

        // Replace with mock emitter
        registry.register(Box::new(MockEmitter::new(TargetPlatform::Claude)));

        // Verify replacement works
        let replaced_output = registry
            .emit_for_target(&TargetPlatform::Claude, &ir)
            .unwrap();
        assert!(replaced_output.contains("mock-output"));
        assert!(!replaced_output.contains("<agent_skill>"));
    }
}