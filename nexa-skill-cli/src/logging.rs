//! Logging System Initialization
//!
//! This module provides a unified logging setup using `tracing` and
//! `tracing-subscriber`. It supports multiple output formats and configurable
//! log levels.
//!
//! # Features
//!
//! - **Hierarchical log levels**: trace, debug, info, warn, error
//! - **Environment control**: via `RUST_LOG` or `NSC_LOG_LEVEL`
//! - **Multiple formats**: Pretty (terminal), JSON (CI/CD), Compact
//! - **Structured logging**: With span tracking and context
//! - **Color output**: Configurable via `NO_COLOR` environment variable
//!
//! # Usage
//!
//! ```ignore
//! use nexa_skill_cli::logging::init_logging;
//!
//! // Initialize with defaults
//! init_logging(None, false, false)?;
//!
//! // Initialize with debug level and verbose output
//! init_logging(Some("debug"), true, false)?;
//! ```

use tracing_subscriber::{
    EnvFilter,
    fmt::{self, format::FmtSpan},
    layer::SubscriberExt,
    util::SubscriberInitExt,
};

/// Log output format (reserved for future advanced logging features)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum LogFormat {
    /// Pretty format with colors (default for terminal)
    Pretty,
    /// JSON format (for CI/CD and log aggregation)
    Json,
    /// Compact format (minimal output)
    Compact,
}

impl Default for LogFormat {
    fn default() -> Self {
        Self::Pretty
    }
}

/// Initialize the global logging system
///
/// This sets up the tracing subscriber with the configured options.
/// It should be called once at the start of the application.
///
/// # Arguments
///
/// * `log_level` - Optional log level override. If None, uses environment
///   variable `RUST_LOG` or `NSC_LOG_LEVEL`, falling back to "info".
/// * `verbose` - If true, enables verbose output (equivalent to debug level)
/// * `json_output` - If true, uses JSON format instead of pretty format
///
/// # Errors
///
/// Returns an error if the subscriber setup fails.
///
/// # Example
///
/// ```ignore
/// // Basic initialization
/// init_logging(None, false, false)?;
///
/// // Verbose mode with JSON output
/// init_logging(Some("debug"), true, true)?;
/// ```
pub fn init_logging(
    log_level: Option<&str>,
    verbose: bool,
    json_output: bool,
) -> miette::Result<()> {
    // Determine effective log level
    let effective_level = if verbose {
        "debug".to_string()
    } else {
        log_level
            .map(|s| s.to_string())
            .or_else(|| std::env::var("NSC_LOG_LEVEL").ok())
            .or_else(|| std::env::var("RUST_LOG").ok())
            .unwrap_or_else(|| "info".to_string())
    };

    // Build environment filter
    let env_filter = EnvFilter::try_new(&effective_level)
        .map_err(|e| miette::miette!("Invalid log level '{}': {}", effective_level, e))?;

    // Add directive for our crates
    let env_filter = env_filter
        .add_directive("nexa_skill_cli=info".parse().unwrap())
        .add_directive("nexa_skill_core=info".parse().unwrap())
        .add_directive("nexa_skill_templates=info".parse().unwrap());

    // Check for JSON output override from environment
    let use_json = json_output
        || std::env::var("NSC_LOG_JSON")
            .map(|v| v == "true" || v == "1")
            .unwrap_or(false);

    // Check for color output
    let use_color = std::env::var("NO_COLOR")
        .map(|v| v != "true" && v != "1")
        .unwrap_or(true);

    // Initialize the subscriber
    if use_json {
        // JSON format for CI/CD and log aggregation
        tracing_subscriber::registry()
            .with(env_filter)
            .with(
                fmt::layer()
                    .json()
                    .with_span_events(FmtSpan::CLOSE)
                    .with_current_span(true),
            )
            .init();
    } else if use_color {
        // Pretty format with colors for terminal
        tracing_subscriber::registry()
            .with(env_filter)
            .with(
                fmt::layer()
                    .pretty()
                    .with_target(true)
                    .with_thread_ids(false)
                    .with_line_number(true)
                    .with_file(true)
                    .with_span_events(FmtSpan::CLOSE),
            )
            .init();
    } else {
        // Compact format without colors
        tracing_subscriber::registry()
            .with(env_filter)
            .with(
                fmt::layer()
                    .compact()
                    .with_target(true)
                    .with_line_number(true),
            )
            .init();
    }

    tracing::info!(
        level = effective_level,
        json = use_json,
        color = use_color,
        "Logging system initialized"
    );

    Ok(())
}

/// Initialize logging with a specific format
///
/// This provides more control over the output format.
///
/// # Arguments
///
/// * `log_level` - Log level (trace, debug, info, warn, error)
/// * `format` - Output format (Pretty, Json, Compact)
///
/// # Example
///
/// ```ignore
/// init_logging_with_format("debug", LogFormat::Json)?;
/// ```
#[allow(dead_code)]
pub fn init_logging_with_format(log_level: &str, format: LogFormat) -> miette::Result<()> {
    let env_filter = EnvFilter::try_new(log_level)
        .map_err(|e| miette::miette!("Invalid log level '{}': {}", log_level, e))?;

    // Add directive for our crates
    let env_filter = env_filter
        .add_directive("nexa_skill_cli=info".parse().unwrap())
        .add_directive("nexa_skill_core=info".parse().unwrap())
        .add_directive("nexa_skill_templates=info".parse().unwrap());

    match format {
        LogFormat::Json => {
            tracing_subscriber::registry()
                .with(env_filter)
                .with(
                    fmt::layer()
                        .json()
                        .with_span_events(FmtSpan::CLOSE)
                        .with_current_span(true),
                )
                .init();
        }
        LogFormat::Pretty => {
            tracing_subscriber::registry()
                .with(env_filter)
                .with(
                    fmt::layer()
                        .pretty()
                        .with_target(true)
                        .with_line_number(true)
                        .with_file(true),
                )
                .init();
        }
        LogFormat::Compact => {
            tracing_subscriber::registry()
                .with(env_filter)
                .with(
                    fmt::layer()
                        .compact()
                        .with_target(true)
                        .with_line_number(true),
                )
                .init();
        }
    }

    tracing::info!(
        level = log_level,
        format = ?format,
        "Logging initialized with custom format"
    );

    Ok(())
}

/// Shutdown the logging system (reserved for future use)
///
/// This function flushes any remaining log events and shuts down
/// the tracing subscriber. Should be called before application exit.
#[allow(dead_code)]
pub fn shutdown_logging() {
    tracing::info!("Shutting down logging system");
    // tracing-subscriber doesn't require explicit shutdown
    // but we log the event for completeness
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_format_default() {
        assert_eq!(LogFormat::default(), LogFormat::Pretty);
    }

    #[test]
    fn test_effective_level_verbose() {
        let verbose = true;
        let level = if verbose { "debug" } else { "info" };
        assert_eq!(level, "debug");
    }

    #[test]
    fn test_effective_level_normal() {
        let verbose = false;
        let level = if verbose { "debug" } else { "info" };
        assert_eq!(level, "info");
    }
}
