//! Nexa Skill Compiler CLI
//!
//! Command-line interface for compiling SKILL.md files to platform-native formats.
//!
//! # Usage
//!
//! ```bash
//! nexa-skill build --claude ./skills/database-migration/
//! nexa-skill check ./skills/
//! nexa-skill validate --report report.html ./skills/
//! ```

use clap::Parser;
use miette::Result;

mod commands;
mod config;
mod logging;

/// Nexa Skill Compiler - Transform SKILL.md to platform-native formats
#[derive(Parser)]
#[command(name = "nexa-skill")]
#[command(author = "Nexa Dev Team")]
#[command(version)]
#[command(about = "AI Agent Skill Compiler", long_about = None)]
struct Cli {
    /// Enable verbose output (debug level logging)
    #[arg(short, long, global = true)]
    verbose: bool,

    /// Quiet mode (only show errors)
    #[arg(short, long, global = true, conflicts_with = "verbose")]
    quiet: bool,

    /// JSON output format for logging (useful for CI/CD)
    #[arg(long, global = true)]
    json: bool,

    /// Subcommand to execute
    #[command(subcommand)]
    command: commands::Commands,
}

fn main() -> Result<()> {
    // Load .env file if present
    dotenvy::dotenv().ok();

    // Parse CLI arguments
    let cli = Cli::parse();

    // Initialize logging based on verbosity settings
    let log_level = if cli.quiet {
        Some("error")
    } else if cli.verbose {
        Some("debug")
    } else {
        None
    };

    logging::init_logging(log_level, cli.verbose, cli.json)?;

    tracing::debug!("CLI arguments parsed successfully");

    // Execute the subcommand
    commands::execute(cli.command)
}
