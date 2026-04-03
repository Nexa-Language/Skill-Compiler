//! Init Command Implementation
//!
//! Initialize a new skill template directory structure.

use clap::Args;
use miette::Result;
use tracing::info;

/// Arguments for the init command
#[derive(Args)]
pub struct InitArgs {
    /// Skill name (kebab-case)
    #[arg(required = true)]
    pub name: String,

    /// Directory to create the skill in
    #[arg(short, long, default_value = ".")]
    pub dir: String,

    /// Template type (basic, advanced, or enterprise)
    #[arg(short, long, default_value = "basic")]
    pub template: String,

    /// Author name
    #[arg(short, long)]
    pub author: Option<String>,

    /// Initial version
    #[arg(short, long, default_value = "1.0.0")]
    pub version: String,
}

/// Execute the init command
pub fn execute(args: InitArgs) -> Result<()> {
    info!("Initializing skill: {}", args.name);

    // Validate skill name format
    if !is_valid_kebab_case(&args.name) {
        miette::bail!(
            "Invalid skill name '{}'. Must be kebab-case: lowercase letters, numbers, and hyphens only.",
            args.name
        );
    }

    // TODO: Implement actual template creation
    // This is a placeholder that will be implemented in later stages

    println!("✅ Init command executed successfully");
    println!("   Skill name: {}", args.name);
    println!("   Directory: {}", args.dir);
    println!("   Template: {}", args.template);
    println!("   Version: {}", args.version);
    if let Some(ref author) = args.author {
        println!("   Author: {}", author);
    }

    Ok(())
}

/// Check if a string is valid kebab-case
fn is_valid_kebab_case(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 64
        && s.chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
        && !s.starts_with('-')
        && !s.ends_with('-')
        && !s.contains("--")
}
