//! Index Command Implementation
//!
//! Generate routing manifest for progressive disclosure.

use clap::Args;
use miette::Result;
use tracing::info;

use nexa_skill_core::backend::routing_manifest::RoutingManifest;
use nexa_skill_core::frontend::ASTBuilder;
use nexa_skill_core::ir::build_ir;

/// Arguments for the index command
#[derive(Args)]
pub struct IndexArgs {
    /// Directory containing SKILL.md files
    #[arg(required = true)]
    pub input_dir: String,

    /// Output file path (default: routing_manifest.yaml in input directory)
    #[arg(short, long)]
    pub output: Option<String>,

    /// Output format: yaml (default) or json
    #[arg(long, default_value = "yaml")]
    pub format: String,
}

/// Execute the index command
pub fn execute(args: IndexArgs) -> Result<()> {
    let input_dir = &args.input_dir;
    let output_path = args
        .output
        .unwrap_or_else(|| format!("{}/routing_manifest.yaml", input_dir));

    info!("Generating routing manifest from: {}", input_dir);

    let mut manifest = RoutingManifest::new();

    // Scan directory for SKILL.md files
    let entries = std::fs::read_dir(input_dir)
        .map_err(|e| miette::miette!("Cannot read directory '{}': {}", input_dir, e))?;

    let mut count = 0usize;
    for entry in entries {
        let entry = entry.map_err(|e| miette::miette!("IO error: {}", e))?;
        let path = entry.path();

        if path.is_dir() {
            let skill_md = path.join("SKILL.md");
            if skill_md.exists() {
                match compile_skill_ir(&skill_md) {
                    Ok(ir) => {
                        manifest.add_skill(&ir);
                        count += 1;
                        info!("  ✓ {}", ir.name);
                    }
                    Err(e) => {
                        info!("  ⚠ Skipped {}: {}", path.display(), e);
                    }
                }
            }
        }
    }

    if count == 0 {
        println!("No SKILL.md files found in: {}", input_dir);
        return Ok(());
    }

    // Write output
    let content = match args.format.as_str() {
        "json" => manifest
            .to_json()
            .map_err(|e| miette::miette!("JSON serialization failed: {}", e))?,
        _ => manifest
            .to_yaml()
            .map_err(|e| miette::miette!("YAML serialization failed: {}", e))?,
    };

    std::fs::write(&output_path, &content)
        .map_err(|e| miette::miette!("Cannot write '{}': {}", output_path, e))?;

    println!(
        "✓ Generated routing manifest with {} skills: {}",
        count, output_path
    );

    Ok(())
}

/// Parse a SKILL.md file and build its SkillIR
fn compile_skill_ir(path: &std::path::Path) -> Result<nexa_skill_core::ir::SkillIR, String> {
    let raw_ast = ASTBuilder::build_from_file(&path.to_string_lossy())
        .map_err(|e| format!("Parse error: {}", e))?;
    Ok(build_ir(&raw_ast))
}