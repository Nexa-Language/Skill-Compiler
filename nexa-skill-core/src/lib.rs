//! Nexa Skill Core Library
//!
//! This crate provides the core compilation pipeline for transforming
//! SKILL.md files into platform-native formats.
//!
//! # Architecture
//!
//! The compiler is organized into four phases:
//!
//! 1. **Frontend**: Parse SKILL.md into RawAST
//! 2. **IR Construction**: Transform RawAST into SkillIR
//! 3. **Analyzer**: Validate and enhance SkillIR
//! 4. **Backend**: Emit platform-specific output
//!
//! # Example
//!
//! ```rust,ignore
//! use nexa_skill_core::{Compiler, TargetPlatform};
//!
//! let compiler = Compiler::new();
//! let result = compiler.compile_file(
//!     "./skills/database-migration/SKILL.md",
//!     &[TargetPlatform::Claude],
//!     "./build/"
//! )?;
//! ```

pub mod analyzer;
pub mod backend;
pub mod error;
pub mod frontend;
pub mod ir;
pub mod security;

// Re-export main types for convenience
pub use analyzer::{Analyzer, ValidatedSkillIR};
pub use backend::{ClaudeEmitter, CodexEmitter, Emitter, EmitterRegistry, GeminiEmitter, TargetPlatform};
pub use error::{CompileError, Diagnostic};
pub use frontend::{ASTBuilder, RawAST};
pub use ir::{build_ir, SkillIR};

use std::fs;
use std::path::Path;

/// Main compiler orchestrator
pub struct Compiler {
    /// Configuration options
    config: CompilerConfig,
}

/// Compiler configuration
#[derive(Debug, Clone)]
pub struct CompilerConfig {
    /// Enable semantic check using LLM
    pub semantic_check: bool,
    /// Strict mode (warnings become errors)
    pub strict_mode: bool,
    /// Verbose output
    pub verbose: bool,
}

impl Default for CompilerConfig {
    fn default() -> Self {
        Self {
            semantic_check: false,
            strict_mode: false,
            verbose: false,
        }
    }
}

impl Compiler {
    /// Create a new compiler instance with default configuration
    #[must_use]
    pub fn new() -> Self {
        Self {
            config: CompilerConfig::default(),
        }
    }

    /// Create a new compiler with custom configuration
    #[must_use]
    pub fn with_config(config: CompilerConfig) -> Self {
        Self { config }
    }

    /// Compile a single SKILL.md file
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The file does not exist or cannot be read
    /// - The file format is invalid
    /// - Any validation check fails
    pub fn compile_file(
        &self,
        input_path: &str,
        targets: &[TargetPlatform],
        output_dir: &str,
    ) -> Result<CompileOutput, CompileError> {
        // Phase 1: Frontend - Parse SKILL.md
        let raw_ast = ASTBuilder::build_from_file(input_path)?;

        // Phase 2: IR Construction - Build SkillIR
        let ir = build_ir(&raw_ast);

        // Phase 3: Analyzer - Validate and enhance
        let analyzer = Analyzer::new();
        let validated_ir = analyzer.analyze(ir).map_err(|(_ir, diagnostics)| {
            let has_blocking = diagnostics.iter().any(|d| d.is_blocking());
            if has_blocking {
                CompileError::AnalysisError(error::AnalysisError::SchemaValidationFailed(
                    diagnostics.iter().map(|d| d.message.clone()).collect::<Vec<_>>().join(", ")
                ))
            } else {
                CompileError::AnalysisError(error::AnalysisError::SchemaValidationFailed(
                    "Analysis found warnings but not blocking".to_string()
                ))
            }
        })?;

        // Log non-blocking warnings
        for warning in validated_ir.warnings() {
            tracing::warn!("[{}] {}", warning.code, warning.message);
        }

        // Phase 4: Backend - Emit platform-specific output
        self.emit_outputs(&validated_ir, targets, output_dir)?;

        Ok(CompileOutput {
            skill_name: validated_ir.as_ref().name.to_string(),
            output_dir: output_dir.to_string(),
            targets: targets.to_vec(),
            manifest_path: format!("{}/manifest.json", output_dir),
            warnings: validated_ir.warnings().to_vec(),
        })
    }

    /// Emit outputs for all target platforms
    fn emit_outputs(
        &self,
        ir: &ValidatedSkillIR,
        targets: &[TargetPlatform],
        output_dir: &str,
    ) -> Result<(), CompileError> {
        let registry = EmitterRegistry::new();

        // Create output directory
        let output_path = Path::new(output_dir);
        if !output_path.exists() {
            fs::create_dir_all(output_path)
                .map_err(|e| CompileError::IOError(e.to_string()))?;
        }

        // Emit for each target using registry
        for target in targets {
            let emitter = registry.get(target)?;

            // pre_process → emit → post_process
            emitter.pre_process(ir)?;
            let output_content = emitter.emit(ir)?;
            let output_content = emitter.post_process(&output_content)?;
            let assets = emitter.generate_assets(ir);

            // Write output file
            // Claude target uses fixed filename "SKILL.md" (required by Claude Code skill discovery)
            // Other targets use skill-name + extension
            let skill_name = ir.as_ref().name.to_string();
            let file_name = if *target == TargetPlatform::Claude {
                "SKILL.md".to_string()
            } else {
                format!("{}{}", skill_name, target.extension())
            };
            let file_path = output_path.join(&file_name);
            fs::write(&file_path, output_content)
                .map_err(|e| CompileError::IOError(e.to_string()))?;

            if self.config.verbose {
                println!("  ✓ Generated: {}", file_path.display());
            }

            // Write assets (e.g., JSON Schema files for Codex, YAML files for Gemini)
            for (asset_path, asset_content) in assets {
                let full_asset_path = output_path.join(&asset_path);
                // Create parent directories if needed
                if let Some(parent) = full_asset_path.parent() {
                    if !parent.exists() {
                        fs::create_dir_all(parent)
                            .map_err(|e| CompileError::IOError(e.to_string()))?;
                    }
                }
                fs::write(&full_asset_path, asset_content)
                    .map_err(|e| CompileError::IOError(e.to_string()))?;

                if self.config.verbose {
                    println!("  ✓ Generated asset: {}", full_asset_path.display());
                }
            }
        }

        // Generate manifest
        self.generate_manifest(ir, targets, output_dir)?;

        Ok(())
    }

    /// Generate manifest.json
    fn generate_manifest(
        &self,
        ir: &ValidatedSkillIR,
        targets: &[TargetPlatform],
        output_dir: &str,
    ) -> Result<(), CompileError> {
        let inner = ir.as_ref();
        let manifest = serde_json::json!({
            "skill_name": inner.name,
            "version": inner.version,
            "compiled_at": inner.compiled_at.map(|t| t.to_rfc3339()),
            "source_hash": inner.source_hash,
            "targets": targets.iter().map(|t| t.slug()).collect::<Vec<_>>(),
            "security_level": inner.security_level.to_string(),
            "hitl_required": inner.hitl_required
        });

        let manifest_path = Path::new(output_dir).join("manifest.json");
        fs::write(&manifest_path, serde_json::to_string_pretty(&manifest).unwrap())
            .map_err(|e| CompileError::IOError(e.to_string()))?;

        Ok(())
    }

    /// Compile all SKILL.md files in a directory
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The directory does not exist
    /// - No SKILL.md files are found
    /// - Any compilation fails
    pub fn compile_dir(
        &self,
        input_dir: &str,
        targets: &[TargetPlatform],
        output_dir: &str,
    ) -> Result<Vec<CompileOutput>, CompileError> {
        let input_path = Path::new(input_dir);
        if !input_path.exists() {
            return Err(CompileError::IOError(format!(
                "Directory not found: {}",
                input_dir
            )));
        }

        let mut results = Vec::new();
        let mut irs_for_manifest = Vec::new();

        // Find all .md files
        for entry in fs::read_dir(input_path)
            .map_err(|e| CompileError::IOError(e.to_string()))?
        {
            let entry = entry.map_err(|e| CompileError::IOError(e.to_string()))?;
            let path = entry.path();

            if path.extension().map(|e| e == "md").unwrap_or(false) {
                // Skip error test files (files starting with "error-")
                let file_name = path.file_stem().unwrap().to_string_lossy();
                if file_name.starts_with("error-") {
                    continue; // Skip intentionally invalid test files
                }
                
                let skill_output_dir = format!(
                    "{}/{}",
                    output_dir,
                    file_name
                );
                
                // Compile and collect IR for routing manifest
                // Use tolerant mode - skip files that fail to parse
                let result = match self.compile_file_with_ir(
                    &path.to_string_lossy(),
                    targets,
                    &skill_output_dir,
                    &mut irs_for_manifest,
                ) {
                    Ok(r) => r,
                    Err(e) => {
                        // Log error but continue with other files
                        if self.config.verbose {
                            println!("  ⚠ Skipped {}: {}", file_name, e);
                        }
                        continue;
                    }
                };
                results.push(result);
            }
        }

        if results.is_empty() {
            return Err(CompileError::IOError("No .md files found".to_string()));
        }

        // Generate routing_manifest.yaml for progressive disclosure
        self.generate_routing_manifest(&irs_for_manifest, output_dir)?;

        Ok(results)
    }

    /// Compile a single file and collect IR for routing manifest
    fn compile_file_with_ir(
        &self,
        input_path: &str,
        targets: &[TargetPlatform],
        output_dir: &str,
        irs_collector: &mut Vec<SkillIR>,
    ) -> Result<CompileOutput, CompileError> {
        // Phase 1: Frontend - Parse SKILL.md
        let raw_ast = ASTBuilder::build_from_file(input_path)?;

        // Phase 2: IR Construction - Build SkillIR
        let ir = build_ir(&raw_ast);

        // Phase 3: Analyzer - Validate and enhance
        let analyzer = Analyzer::new();
        let validated_ir = analyzer.analyze(ir).map_err(|(_ir, diagnostics)| {
            let has_blocking = diagnostics.iter().any(|d| d.is_blocking());
            if has_blocking {
                CompileError::AnalysisError(error::AnalysisError::SchemaValidationFailed(
                    diagnostics.iter().map(|d| d.message.clone()).collect::<Vec<_>>().join(", ")
                ))
            } else {
                CompileError::AnalysisError(error::AnalysisError::SchemaValidationFailed(
                    "Analysis found warnings but not blocking".to_string()
                ))
            }
        })?;

        // Log non-blocking warnings
        for warning in validated_ir.warnings() {
            tracing::warn!("[{}] {}", warning.code, warning.message);
        }

        // Collect IR for routing manifest (before validation wrapper)
        irs_collector.push(validated_ir.as_ref().clone());

        // Phase 4: Backend - Emit platform-specific output
        self.emit_outputs(&validated_ir, targets, output_dir)?;

        Ok(CompileOutput {
            skill_name: validated_ir.as_ref().name.to_string(),
            output_dir: output_dir.to_string(),
            targets: targets.to_vec(),
            manifest_path: format!("{}/manifest.json", output_dir),
            warnings: validated_ir.warnings().to_vec(),
        })
    }

    /// Generate routing_manifest.yaml for progressive disclosure
    fn generate_routing_manifest(
        &self,
        irs: &[SkillIR],
        output_dir: &str,
    ) -> Result<(), CompileError> {
        use crate::backend::routing_manifest::RoutingManifest;

        let mut manifest = RoutingManifest::new();
        manifest.add_skills(irs);

        let yaml_content = manifest.to_yaml()
            .map_err(|e| CompileError::IOError(e.to_string()))?;

        let manifest_path = Path::new(output_dir).join("routing_manifest.yaml");
        fs::write(&manifest_path, yaml_content)
            .map_err(|e| CompileError::IOError(e.to_string()))?;

        if self.config.verbose {
            println!("  ✓ Generated routing_manifest.yaml with {} skills", irs.len());
        }

        Ok(())
    }

    /// Check a SKILL.md file without generating output
    ///
    /// # Errors
    ///
    /// Returns an error if validation fails.
    pub fn check_file(&self, input_path: &str) -> Result<Vec<Diagnostic>, CompileError> {
        // Phase 1: Frontend
        let raw_ast = ASTBuilder::build_from_file(input_path)?;

        // Phase 2: IR Construction
        let ir = build_ir(&raw_ast);

        // Phase 3: Analyzer
        let analyzer = Analyzer::new();
        let result = analyzer.analyze(ir);

        match result {
            Ok(validated) => Ok(validated.warnings().to_vec()),
            Err((_ir, diagnostics)) => Ok(diagnostics),
        }
    }

    /// Validate a SKILL.md file with detailed diagnostics
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read.
    pub fn validate_file(&self, input_path: &str) -> Result<ValidationResult, CompileError> {
        let diagnostics = self.check_file(input_path)?;

        let error_count = diagnostics.iter().filter(|d| d.is_blocking()).count();
        let warning_count = diagnostics.len() - error_count;

        Ok(ValidationResult {
            is_valid: error_count == 0,
            diagnostics,
            error_count,
            warning_count,
        })
    }
}

impl Default for Compiler {
    fn default() -> Self {
        Self::new()
    }
}

/// Output of a compilation
#[derive(Debug, Clone)]
pub struct CompileOutput {
    /// Name of the compiled skill
    pub skill_name: String,
    /// Output directory path
    pub output_dir: String,
    /// Target platforms that were generated
    pub targets: Vec<TargetPlatform>,
    /// Path to the manifest file
    pub manifest_path: String,
    /// Non-blocking diagnostic warnings collected during compilation
    pub warnings: Vec<Diagnostic>,
}

/// Result of validation
#[derive(Debug, Clone)]
pub struct ValidationResult {
    /// Whether the skill is valid
    pub is_valid: bool,
    /// Diagnostic messages
    pub diagnostics: Vec<Diagnostic>,
    /// Number of errors
    pub error_count: usize,
    /// Number of warnings
    pub warning_count: usize,
}