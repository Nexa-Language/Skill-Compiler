//! Build Command Implementation
//!
//! Compile SKILL.md file(s) to platform-native formats.

use clap::Args;
use miette::Result;
use tracing::info;

/// Arguments for the build command
#[derive(Args)]
pub struct BuildArgs {
    /// SKILL.md file path or skill directory
    #[arg(required = true)]
    pub input: String,

    /// Generate Claude XML output
    #[arg(long)]
    pub claude: bool,

    /// Generate Codex JSON Schema output
    #[arg(long)]
    pub codex: bool,

    /// Generate Gemini Markdown output
    #[arg(long)]
    pub gemini: bool,

    /// Generate Kimi Markdown output
    #[arg(long)]
    pub kimi: bool,

    /// Generate all supported platform outputs
    #[arg(long, conflicts_with_all = ["claude", "codex", "gemini", "kimi"])]
    pub all: bool,

    /// Output directory path
    #[arg(short, long, default_value = "./build/")]
    pub out_dir: String,

    /// Configuration file path
    #[arg(short, long)]
    pub config: Option<String>,

    /// Force overwrite existing outputs
    #[arg(short, long)]
    pub force: bool,
}

/// Execute the build command
pub fn execute(args: BuildArgs) -> Result<()> {
    info!("Building skill from: {}", args.input);

    let targets = resolve_targets(&args);

    if targets.is_empty() {
        miette::bail!(
            "No target platform specified. Use --claude, --codex, --gemini, --kimi, or --all"
        );
    }

    // Create compiler
    let compiler = nexa_skill_core::Compiler::new();
    
    // Check if input is a directory or file
    let input_path = std::path::Path::new(&args.input);
    
    if input_path.is_dir() {
        // Directory compilation - generates routing_manifest.yaml
        let results = compiler.compile_dir(&args.input, &targets, &args.out_dir)
            .map_err(|e| miette::miette!("Compilation failed: {}", e))?;
        
        println!("✅ Build completed successfully");
        println!("   Skills compiled: {}", results.len());
        println!("   Output: {}", args.out_dir);
        println!("   Targets: {:?}", targets.iter().map(|t| t.slug()).collect::<Vec<_>>());
        println!("   Routing manifest: {}/routing_manifest.yaml", args.out_dir);
    } else {
        // Single file compilation
        let result = compiler.compile_file(&args.input, &targets, &args.out_dir)
            .map_err(|e| miette::miette!("Compilation failed: {}", e))?;

        println!("✅ Build completed successfully");
        println!("   Skill: {}", result.skill_name);
        println!("   Output: {}", result.output_dir);
        println!("   Targets: {:?}", result.targets.iter().map(|t| t.slug()).collect::<Vec<_>>());
        println!("   Manifest: {}", result.manifest_path);
    }

    Ok(())
}

/// Resolve target platforms from CLI arguments
fn resolve_targets(args: &BuildArgs) -> Vec<nexa_skill_core::TargetPlatform> {
    use nexa_skill_core::TargetPlatform;
    
    if args.all {
        return vec![TargetPlatform::Claude, TargetPlatform::Codex, TargetPlatform::Gemini, TargetPlatform::Kimi];
    }

    let mut targets = Vec::new();
    if args.claude {
        targets.push(TargetPlatform::Claude);
    }
    if args.codex {
        targets.push(TargetPlatform::Codex);
    }
    if args.gemini {
        targets.push(TargetPlatform::Gemini);
    }
    if args.kimi {
        targets.push(TargetPlatform::Kimi);
    }
    targets
}
