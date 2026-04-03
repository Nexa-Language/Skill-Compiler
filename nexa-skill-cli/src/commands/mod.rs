//! CLI Commands Module
//!
//! This module defines all CLI subcommands and their implementations.

mod build;
mod check;
mod clean;
mod init;
mod list;
mod validate;

use clap::Subcommand;
use miette::Result;

pub use build::BuildArgs;
pub use check::CheckArgs;
pub use clean::CleanArgs;
pub use init::InitArgs;
pub use list::ListArgs;
pub use validate::ValidateArgs;

/// Available CLI subcommands
#[derive(Subcommand)]
pub enum Commands {
    /// Compile SKILL.md file(s) to platform-native formats
    Build(BuildArgs),

    /// Validate SKILL.md format without generating output
    Check(CheckArgs),

    /// Detailed validation with diagnostic report
    Validate(ValidateArgs),

    /// Initialize a new skill template
    Init(InitArgs),

    /// List compiled skills
    List(ListArgs),

    /// Clean build artifacts
    Clean(CleanArgs),
}

/// Execute the given command
pub fn execute(command: Commands) -> Result<()> {
    match command {
        Commands::Build(args) => build::execute(args),
        Commands::Check(args) => check::execute(args),
        Commands::Validate(args) => validate::execute(args),
        Commands::Init(args) => init::execute(args),
        Commands::List(args) => list::execute(args),
        Commands::Clean(args) => clean::execute(args),
    }
}
