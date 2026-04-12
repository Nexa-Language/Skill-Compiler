//! Configuration Management
//!
//! This module handles loading configuration from environment variables
//! and configuration files. It provides strongly-typed configuration
//! with validation and sensible defaults.
//!
//! # Configuration Sources
//!
//! 1. Environment variables (highest priority)
//! 2. Configuration file (`.nsc.toml` or specified path)
//! 3. Default values (lowest priority)
//!
//! # Environment Variables
//!
//! | Variable | Description | Default |
//! |----------|-------------|---------|
//! | `NSC_OUT_DIR` | Output directory | `./build/` |
//! | `NSC_SEMANTIC_CHECK` | Enable LLM semantic check | `false` |
//! | `NSC_GENERATE_SIGNATURE` | Generate signature files | `false` |
//! | `NSC_DEFAULT_TARGET` | Default compilation target | `claude` |
//! | `NSC_MCP_WHITELIST` | MCP server whitelist (comma-separated) | `*` |
//! | `NSC_ALLOW_UNDECLARED_MCP` | Allow undeclared MCP servers | `false` |
//! | `NSC_HIGH_RISK_KEYWORDS` | High-risk keywords (comma-separated) | (built-in list) |
//! | `NSC_FORCE_HITL_CRITICAL` | Force HITL for critical level | `true` |
//! | `NSC_LOG_LEVEL` | Log level | `info` |
//! | `OPENAI_API_KEY` | OpenAI API key | - |
//! | `OPENAI_API_BASE` | OpenAI API base URL | - |
//! | `GITHUB_TOKEN` | GitHub token | - |

use serde::Deserialize;
use std::path::PathBuf;

/// Compilation target platform (reserved for future config file usage)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
#[allow(dead_code)]
pub enum TargetPlatform {
    /// Claude XML format
    Claude,
    /// Codex/GPT JSON Schema format
    Codex,
    /// Gemini Markdown format
    Gemini,
}

impl Default for TargetPlatform {
    fn default() -> Self {
        Self::Claude
    }
}

impl std::fmt::Display for TargetPlatform {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Claude => write!(f, "claude"),
            Self::Codex => write!(f, "codex"),
            Self::Gemini => write!(f, "gemini"),
        }
    }
}

/// Application configuration
///
/// This struct holds all configuration options for the Nexa Skill Compiler.
/// It can be loaded from environment variables, configuration files, or defaults.
/// Note: Some fields are reserved for future features and may not be used currently.
#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct Config {
    // === API Configuration ===
    /// OpenAI API key for LLM semantic check
    #[serde(default)]
    pub openai_api_key: Option<String>,

    /// OpenAI API base URL (for custom endpoints)
    #[serde(default)]
    pub openai_api_base: Option<String>,

    /// GitHub token for fetching skill datasets
    #[serde(default)]
    pub github_token: Option<String>,

    // === Output Configuration ===
    /// Default output directory for compiled artifacts
    #[serde(default = "default_out_dir")]
    pub default_out_dir: PathBuf,

    /// Default compilation target
    #[serde(default)]
    pub default_target: TargetPlatform,

    /// Generate signature files for compiled skills
    #[serde(default)]
    pub generate_signature: bool,

    // === Security Configuration ===
    /// MCP server whitelist (comma-separated, `*` means all allowed)
    #[serde(default = "default_mcp_whitelist")]
    pub mcp_whitelist: Vec<String>,

    /// Allow undeclared MCP servers
    #[serde(default)]
    pub allow_undeclared_mcp: bool,

    /// High-risk keywords for permission auditing
    #[serde(default = "default_high_risk_keywords")]
    pub high_risk_keywords: Vec<String>,

    /// Force HITL approval for critical security level
    #[serde(default = "default_force_hitl_critical")]
    pub force_hitl_critical: bool,

    // === Feature Flags ===
    /// Enable LLM semantic check by default
    #[serde(default)]
    pub semantic_check_enabled: bool,

    /// Log level (trace, debug, info, warn, error)
    #[serde(default = "default_log_level")]
    pub log_level: String,

    /// Enable verbose output
    #[serde(default)]
    pub verbose: bool,

    /// Enable color output
    #[serde(default = "default_color_output")]
    pub color_output: bool,
}

fn default_out_dir() -> PathBuf {
    PathBuf::from("./build/")
}

fn default_mcp_whitelist() -> Vec<String> {
    vec!["*".to_string()] // Allow all by default
}

fn default_high_risk_keywords() -> Vec<String> {
    vec![
        "rm".to_string(),
        "sudo".to_string(),
        "chmod".to_string(),
        "chown".to_string(),
        "dd".to_string(),
        "mkfs".to_string(),
        "fdisk".to_string(),
        "kill".to_string(),
        "pkill".to_string(),
        "shutdown".to_string(),
        "reboot".to_string(),
        "exec".to_string(),
        "eval".to_string(),
        "system".to_string(),
        "shell".to_string(),
        "bash".to_string(),
        "sh".to_string(),
    ]
}

fn default_force_hitl_critical() -> bool {
    true
}

fn default_log_level() -> String {
    "info".to_string()
}

fn default_color_output() -> bool {
    true
}

impl Default for Config {
    fn default() -> Self {
        Self {
            openai_api_key: None,
            openai_api_base: None,
            github_token: None,
            default_out_dir: default_out_dir(),
            default_target: TargetPlatform::Claude,
            generate_signature: false,
            mcp_whitelist: default_mcp_whitelist(),
            allow_undeclared_mcp: false,
            high_risk_keywords: default_high_risk_keywords(),
            force_hitl_critical: true,
            semantic_check_enabled: false,
            log_level: default_log_level(),
            verbose: false,
            color_output: true,
        }
    }
}

#[allow(dead_code)]
impl Config {
    /// Load configuration from environment variables
    ///
    /// This method reads configuration from environment variables,
    /// falling back to defaults for missing values.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let config = Config::from_env();
    /// ```
    pub fn from_env() -> Self {
        Self {
            openai_api_key: std::env::var("OPENAI_API_KEY").ok(),
            openai_api_base: std::env::var("OPENAI_API_BASE").ok(),
            github_token: std::env::var("GITHUB_TOKEN").ok(),
            default_out_dir: std::env::var("NSC_OUT_DIR")
                .map(PathBuf::from)
                .unwrap_or_else(|_| default_out_dir()),
            default_target: std::env::var("NSC_DEFAULT_TARGET")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or_default(),
            generate_signature: std::env::var("NSC_GENERATE_SIGNATURE")
                .map(|v| v == "true" || v == "1")
                .unwrap_or(false),
            mcp_whitelist: std::env::var("NSC_MCP_WHITELIST")
                .map(|v| v.split(',').map(|s| s.trim().to_string()).collect())
                .unwrap_or_else(|_| default_mcp_whitelist()),
            allow_undeclared_mcp: std::env::var("NSC_ALLOW_UNDECLARED_MCP")
                .map(|v| v == "true" || v == "1")
                .unwrap_or(false),
            high_risk_keywords: std::env::var("NSC_HIGH_RISK_KEYWORDS")
                .map(|v| v.split(',').map(|s| s.trim().to_string()).collect())
                .unwrap_or_else(|_| default_high_risk_keywords()),
            force_hitl_critical: std::env::var("NSC_FORCE_HITL_CRITICAL")
                .map(|v| v != "false" && v != "0")
                .unwrap_or_else(|_| default_force_hitl_critical()),
            semantic_check_enabled: std::env::var("NSC_SEMANTIC_CHECK")
                .map(|v| v == "true" || v == "1")
                .unwrap_or(false),
            log_level: std::env::var("NSC_LOG_LEVEL")
                .or_else(|_| std::env::var("RUST_LOG"))
                .unwrap_or_else(|_| default_log_level()),
            verbose: std::env::var("NSC_VERBOSE")
                .map(|v| v == "true" || v == "1")
                .unwrap_or(false),
            color_output: std::env::var("NO_COLOR")
                .map(|v| v != "true" && v != "1")
                .unwrap_or(true),
        }
    }

    /// Load configuration from a TOML file
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the configuration file
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The file cannot be read
    /// - The TOML content is malformed
    /// - Required fields are missing
    ///
    /// # Example
    ///
    /// ```ignore
    /// let config = Config::from_file(".nsc.toml")?;
    /// ```
    pub fn from_file(path: &str) -> miette::Result<Self> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| miette::miette!("Failed to read config file '{}': {}", path, e))?;

        let config: Config = toml::from_str(&content)
            .map_err(|e| miette::miette!("Failed to parse config file: {}", e))?;

        Ok(config)
    }

    /// Load configuration with fallback chain
    ///
    /// Priority: env vars > config file > defaults
    ///
    /// # Arguments
    ///
    /// * `config_path` - Optional path to configuration file
    ///
    /// # Example
    ///
    /// ```ignore
    /// // Try .nsc.toml, then fall back to env vars and defaults
    /// let config = Config::load(None)?;
    ///
    /// // Use specific config file
    /// let config = Config::load(Some("./config/production.toml"))?;
    /// ```
    pub fn load(config_path: Option<&str>) -> miette::Result<Self> {
        // Load .env file if present (using dotenvy)
        dotenvy::dotenv().ok();

        // Start with defaults
        let mut config = Self::default();

        // Load from config file if provided or if default config exists
        let config_file = config_path.or_else(|| {
            if std::path::Path::new(".nsc.toml").exists() {
                Some(".nsc.toml")
            } else {
                None
            }
        });

        if let Some(path) = config_file {
            if std::path::Path::new(path).exists() {
                config = Self::from_file(path)?;
            }
        }

        // Merge environment variables (highest priority)
        config = config.merge_env();

        Ok(config)
    }

    /// Merge configuration from environment variables
    ///
    /// Environment variables override file configuration.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let config = Config::from_file(".nsc.toml")?.merge_env();
    /// ```
    pub fn merge_env(mut self) -> Self {
        if let Ok(key) = std::env::var("OPENAI_API_KEY") {
            self.openai_api_key = Some(key);
        }
        if let Ok(base) = std::env::var("OPENAI_API_BASE") {
            self.openai_api_base = Some(base);
        }
        if let Ok(token) = std::env::var("GITHUB_TOKEN") {
            self.github_token = Some(token);
        }
        if let Ok(dir) = std::env::var("NSC_OUT_DIR") {
            self.default_out_dir = PathBuf::from(dir);
        }
        if let Ok(target) = std::env::var("NSC_DEFAULT_TARGET") {
            if let Ok(t) = target.parse() {
                self.default_target = t;
            }
        }
        if let Ok(sig) = std::env::var("NSC_GENERATE_SIGNATURE") {
            self.generate_signature = sig == "true" || sig == "1";
        }
        if let Ok(whitelist) = std::env::var("NSC_MCP_WHITELIST") {
            self.mcp_whitelist = whitelist.split(',').map(|s| s.trim().to_string()).collect();
        }
        if let Ok(allow) = std::env::var("NSC_ALLOW_UNDECLARED_MCP") {
            self.allow_undeclared_mcp = allow == "true" || allow == "1";
        }
        if let Ok(keywords) = std::env::var("NSC_HIGH_RISK_KEYWORDS") {
            self.high_risk_keywords = keywords.split(',').map(|s| s.trim().to_string()).collect();
        }
        if let Ok(force) = std::env::var("NSC_FORCE_HITL_CRITICAL") {
            self.force_hitl_critical = force != "false" && force != "0";
        }
        if let Ok(check) = std::env::var("NSC_SEMANTIC_CHECK") {
            self.semantic_check_enabled = check == "true" || check == "1";
        }
        if let Ok(level) = std::env::var("NSC_LOG_LEVEL").or_else(|_| std::env::var("RUST_LOG")) {
            self.log_level = level;
        }
        if let Ok(verbose) = std::env::var("NSC_VERBOSE") {
            self.verbose = verbose == "true" || verbose == "1";
        }
        if let Ok(no_color) = std::env::var("NO_COLOR") {
            self.color_output = no_color != "true" && no_color != "1";
        }
        self
    }

    /// Check if semantic check is available (has API key)
    pub fn semantic_check_available(&self) -> bool {
        self.semantic_check_enabled && self.openai_api_key.is_some()
    }

    /// Check if a MCP server is in the whitelist
    pub fn is_mcp_allowed(&self, server: &str) -> bool {
        if self.mcp_whitelist.contains(&"*".to_string()) {
            return true;
        }
        self.mcp_whitelist.iter().any(|s| s == server)
    }

    /// Check if a keyword is high-risk
    pub fn is_high_risk(&self, keyword: &str) -> bool {
        self.high_risk_keywords
            .iter()
            .any(|k| k == keyword || keyword.contains(k) || keyword.starts_with(k))
    }
}

impl std::str::FromStr for TargetPlatform {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "claude" => Ok(Self::Claude),
            "codex" | "gpt" => Ok(Self::Codex),
            "gemini" => Ok(Self::Gemini),
            _ => Err(format!("Unknown target platform: {}", s)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.default_target, TargetPlatform::Claude);
        assert!(!config.semantic_check_enabled);
        assert!(config.force_hitl_critical);
        assert!(config.mcp_whitelist.contains(&"*".to_string()));
    }

    #[test]
    fn test_target_platform_parse() {
        assert_eq!(
            "claude".parse::<TargetPlatform>().unwrap(),
            TargetPlatform::Claude
        );
        assert_eq!(
            "codex".parse::<TargetPlatform>().unwrap(),
            TargetPlatform::Codex
        );
        assert_eq!(
            "gpt".parse::<TargetPlatform>().unwrap(),
            TargetPlatform::Codex
        );
        assert_eq!(
            "gemini".parse::<TargetPlatform>().unwrap(),
            TargetPlatform::Gemini
        );
        assert!("unknown".parse::<TargetPlatform>().is_err());
    }

    #[test]
    fn test_is_mcp_allowed() {
        let config = Config::default();
        assert!(config.is_mcp_allowed("any-server"));
        assert!(config.is_mcp_allowed("filesystem"));

        let mut config = Config::default();
        config.mcp_whitelist = vec!["filesystem".to_string(), "github".to_string()];
        assert!(config.is_mcp_allowed("filesystem"));
        assert!(config.is_mcp_allowed("github"));
        assert!(!config.is_mcp_allowed("unknown"));
    }

    #[test]
    fn test_is_high_risk() {
        let config = Config::default();
        assert!(config.is_high_risk("rm"));
        assert!(config.is_high_risk("sudo"));
        assert!(config.is_high_risk("rm -rf /"));
        assert!(!config.is_high_risk("ls"));
        assert!(!config.is_high_risk("cat"));
    }

    #[test]
    fn test_semantic_check_available() {
        let config = Config::default();
        assert!(!config.semantic_check_available());

        let mut config = Config::default();
        config.semantic_check_enabled = true;
        assert!(!config.semantic_check_available());

        config.openai_api_key = Some("test-key".to_string());
        assert!(config.semantic_check_available());
    }
}
