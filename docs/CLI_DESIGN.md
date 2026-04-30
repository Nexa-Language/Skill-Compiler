# CLI 交互设计

> **命令行界面设计、命令规范、用户体验与错误展示**

---

## 1. CLI 设计概述

NSC 的 CLI 设计遵循 Rust 生态的极简与高效原则，基于 `clap` 构建，集成 `miette` 提供精美的错误报告。

### 1.1 设计原则

| 原则 | 描述 |
|------|------|
| **极简入口** | 单一主命令 `nexa-skill`，子命令区分功能 |
| **一致性** | 命令参数风格与 Rust 工具链保持一致 |
| **可发现性** | 内置帮助文档和示例，`--help` 即可了解全部功能 |
| **精美报错** | 使用 `miette` 提供带行号、代码片段的错误提示 |

### 1.2 命令层级结构

```text
nexa-skill
├── build       # 编译 SKILL.md
├── check       # 验证格式但不生成产物
├── validate    # 详细验证并输出诊断报告
├── init        # 初始化新技能模板
├── list        # 列出已编译的技能
└── clean       # 清理编译产物
└── version     # 显示版本信息
└── help        # 显示帮助信息
```

---

## 2. 命令详细设计

### 2.1 `build` 命令

**用途**：编译 SKILL.md 文件或目录，生成平台特定产物。

**语法**：
```bash
nexa-skill build [OPTIONS] <FILE_OR_DIR>
```

**参数**：

| 参数 | 类型 | 描述 | 默认值 |
|------|------|------|--------|
| `<FILE_OR_DIR>` | 必选 | SKILL.md 文件路径或技能目录 | - |
| `--claude` | Flag | 生成 Claude XML 产物 | - |
| `--codex` | Flag | 生成 Codex JSON Schema 产物 | - |
| `--gemini` | Flag | 生成 Gemini Markdown 产物 | - |
| `--kimi` | Flag | 生成 Kimi Markdown 产物 | - |
| `--all` | Flag | 生成所有支持平台的产物 | - |
| `--out-dir <DIR>` | Option | 输出目录路径 | `./build/` |
| `--config <FILE>` | Option | 配置文件路径 | `./nsc.toml` |
| `--verbose` | Flag | 显示详细输出 | - |
| `--quiet` | Flag | 静默模式，仅显示错误 | - |
| `--force` | Flag | 强制覆盖已存在的产物 | - |

**示例**：
```bash
# 单目标编译
nexa-skill build --claude database-migration.md

# 多目标编译
nexa-skill build --claude --codex ./skills/web-scraper/

# 全平台编译，输出到指定目录
nexa-skill build --all --out-dir ./dist ./skills/

# 使用配置文件
nexa-skill build --config ./nsc.toml ./skills/
```

### 2.2 `check` 命令

**用途**：验证 SKILL.md 格式正确性，但不生成产物文件。

**语法**：
```bash
nexa-skill check [OPTIONS] <FILE_OR_DIR>
```

**参数**：

| 参数 | 类型 | 描述 | 默认值 |
|------|------|------|--------|
| `<FILE_OR_DIR>` | 必选 | SKILL.md 文件路径或技能目录 | - |
| `--strict` | Flag | 启用严格模式（警告也视为错误） | - |
| `--format` | Option | 输出格式 (`text`/`json`) | `text` |
| `--verbose` | Flag | 显示详细诊断信息 | - |

**示例**：
```bash
# 基础检查
nexa-skill check database-migration.md

# 严格模式检查
nexa-skill check --strict ./skills/

# JSON 格式输出（用于 CI/CD）
nexa-skill check --format json ./skills/
```

### 2.3 `validate` 命令

**用途**：详细验证并输出完整的诊断报告，包括所有警告和建议。

**语法**：
```bash
nexa-skill validate [OPTIONS] <FILE_OR_DIR>
```

**参数**：

| 参数 | 类型 | 描述 | 默认值 |
|------|------|------|--------|
| `<FILE_OR_DIR>` | 必选 | SKILL.md 文件路径或技能目录 | - |
| `--report <FILE>` | Option | 输出诊断报告文件 | - |
| `--format` | Option | 报告格式 (`text`/`json`/`html`) | `text` |
| `--suggest` | Flag | 显示修复建议 | - |

**示例**：
```bash
# 详细验证
nexa-skill validate database-migration.md

# 生成 HTML 报告
nexa-skill validate --report report.html --format html ./skills/

# 显示修复建议
nexa-skill validate --suggest database-migration.md
```

### 2.4 `init` 命令

**用途**：初始化新的技能模板目录结构。

**语法**：
```bash
nexa-skill init [OPTIONS] <SKILL_NAME>
```

**参数**：

| 参数 | 类型 | 描述 | 默认值 |
|------|------|------|--------|
| `<SKILL_NAME>` | 必选 | 技能名称（kebab-case） | - |
| `--dir <DIR>` | Option | 创建目录路径 | 当前目录 |
| `--template <TYPE>` | Option | 模板类型 (`basic`/`advanced`/`enterprise`) | `basic` |
| `--author <NAME>` | Option | 作者名称 | - |
| `--version <VER>` | Option | 初始版本号 | `1.0.0` |

**示例**：
```bash
# 创建基础技能模板
nexa-skill init web-scraper

# 创建高级模板
nexa-skill init --template advanced database-migration

# 指定目录和作者
nexa-skill init --dir ./skills --author "nexa-dev" web-scraper
```

### 2.5 `list` 命令

**用途**：列出已编译的技能及其产物信息。

**语法**：
```bash
nexa-skill list [OPTIONS]
```

**参数**：

| 参数 | 类型 | 描述 | 默认值 |
|------|------|------|--------|
| `--dir <DIR>` | Option | 编译产物目录 | `./build/` |
| `--format` | Option | 输出格式 (`table`/`json`/`simple`) | `table` |
| `--filter <PATTERN>` | Option | 过滤技能名称 | - |

**示例**：
```bash
# 列出所有已编译技能
nexa-skill list

# JSON 格式输出
nexa-skill list --format json

# 过滤特定技能
nexa-skill list --filter "database-*"
```

### 2.6 `clean` 命令

**用途**：清理编译产物目录。

**语法**：
```bash
nexa-skill clean [OPTIONS]
```

**参数**：

| 参数 | 类型 | 描述 | 默认值 |
|------|------|------|--------|
| `--dir <DIR>` | Option | 清理目录路径 | `./build/` |
| `--skill <NAME>` | Option | 仅清理指定技能 | - |
| `--dry-run` | Flag | 显示将删除的文件但不实际删除 | - |
| `--force` | Flag | 强制删除，不提示确认 | - |

**示例**：
```bash
# 清理所有产物
nexa-skill clean

# 仅清理特定技能
nexa-skill clean --skill database-migration

# 预览删除（不实际执行）
nexa-skill clean --dry-run
```

---

## 3. CLI 实现架构

### 3.1 Clap 命令定义

```rust
// nexa-skill-cli/src/main.rs

use clap::{Parser, Subcommand};

/// Nexa Skill Compiler - 将 SKILL.md 编译为 AI Agent 可执行产物
#[derive(Parser)]
#[command(name = "nexa-skill")]
#[command(author = "Nexa Dev Team")]
#[command(version = "1.0.0")]
#[command(about = "AI Agent Skill Compiler", long_about = None)]
struct Cli {
    /// 启用详细输出
    #[arg(short, long, global = true)]
    verbose: bool,
    
    /// 静默模式
    #[arg(short, long, global = true, conflicts_with = "verbose")]
    quiet: bool,
    
    /// 子命令
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// 编译 SKILL.md 文件或目录
    Build(BuildArgs),
    
    /// 验证格式但不生成产物
    Check(CheckArgs),
    
    /// 详细验证并输出诊断报告
    Validate(ValidateArgs),
    
    /// 初始化新技能模板
    Init(InitArgs),
    
    /// 列出已编译的技能
    List(ListArgs),
    
    /// 清理编译产物
    Clean(CleanArgs),
}

/// build 命令参数
#[derive(Parser)]
struct BuildArgs {
    /// SKILL.md 文件路径或技能目录
    #[arg(required = true)]
    input: String,
    
    /// 生成 Claude XML 产物
    #[arg(long)]
    claude: bool,
    
    /// 生成 Codex JSON Schema 产物
    #[arg(long)]
    codex: bool,
    
    /// 生成 Gemini Markdown 产物
    #[arg(long)]
    gemini: bool,
    
    /// 生成 Kimi Markdown 产物
    #[arg(long)]
    kimi: bool,
    
    /// 生成所有支持平台的产物
    #[arg(long, conflicts_with_all = ["claude", "codex", "gemini", "kimi"])]
    all: bool,
    
    /// 输出目录路径
    #[arg(short, long, default_value = "./build/")]
    out_dir: String,
    
    /// 配置文件路径
    #[arg(short, long)]
    config: Option<String>,
    
    /// 强制覆盖已存在的产物
    #[arg(short, long)]
    force: bool,
}

/// check 命令参数
#[derive(Parser)]
struct CheckArgs {
    /// SKILL.md 文件路径或技能目录
    #[arg(required = true)]
    input: String,
    
    /// 启用严格模式
    #[arg(long)]
    strict: bool,
    
    /// 输出格式
    #[arg(long, default_value = "text")]
    format: String,
}

/// validate 命令参数
#[derive(Parser)]
struct ValidateArgs {
    /// SKILL.md 文件路径或技能目录
    #[arg(required = true)]
    input: String,
    
    /// 输出诊断报告文件
    #[arg(short, long)]
    report: Option<String>,
    
    /// 报告格式
    #[arg(long, default_value = "text")]
    format: String,
    
    /// 显示修复建议
    #[arg(long)]
    suggest: bool,
}

/// init 命令参数
#[derive(Parser)]
struct InitArgs {
    /// 技能名称（kebab-case）
    #[arg(required = true)]
    name: String,
    
    /// 创建目录路径
    #[arg(short, long, default_value = ".")]
    dir: String,
    
    /// 模板类型
    #[arg(short, long, default_value = "basic")]
    template: String,
    
    /// 作者名称
    #[arg(short, long)]
    author: Option<String>,
    
    /// 初始版本号
    #[arg(short, long, default_value = "1.0.0")]
    version: String,
}

/// list 命令参数
#[derive(Parser)]
struct ListArgs {
    /// 编译产物目录
    #[arg(short, long, default_value = "./build/")]
    dir: String,
    
    /// 输出格式
    #[arg(long, default_value = "table")]
    format: String,
    
    /// 过滤技能名称
    #[arg(short, long)]
    filter: Option<String>,
}

/// clean 命令参数
#[derive(Parser)]
struct CleanArgs {
    /// 清理目录路径
    #[arg(short, long, default_value = "./build/")]
    dir: String,
    
    /// 仅清理指定技能
    #[arg(short, long)]
    skill: Option<String>,
    
    /// 预览删除
    #[arg(long)]
    dry_run: bool,
    
    /// 强制删除
    #[arg(short, long)]
    force: bool,
}

fn main() -> miette::Result<()> {
    let cli = Cli::parse();
    
    // 设置日志级别
    if cli.verbose {
        std::env::set_var("RUST_LOG", "debug");
    } else if cli.quiet {
        std::env::set_var("RUST_LOG", "error");
    }
    
    // 执行子命令
    match cli.command {
        Commands::Build(args) => cmd_build::execute(args),
        Commands::Check(args) => cmd_check::execute(args),
        Commands::Validate(args) => cmd_validate::execute(args),
        Commands::Init(args) => cmd_init::execute(args),
        Commands::List(args) => cmd_list::execute(args),
        Commands::Clean(args) => cmd_clean::execute(args),
    }
}
```

### 3.2 命令执行模块

```rust
// nexa-skill-cli/src/commands/build.rs

use crate::CliArgs;
use nexa_skill_core::{Compiler, TargetPlatform};
use nexa_skill_core::error::CompileError;

pub fn execute(args: BuildArgs) -> miette::Result<()> {
    // 解析目标平台
    let targets = resolve_targets(&args);
    
    if targets.is_empty() {
        return Err(miette::miette!("No target platform specified. Use --claude, --codex, --gemini, --kimi, or --all"));
    }
    
    // 创建编译器
    let compiler = Compiler::new();
    
    // 执行编译
    let result = if std::path::Path::new(&args.input).is_dir() {
        compiler.compile_dir(&args.input, &targets, &args.out_dir)
    } else {
        compiler.compile_file(&args.input, &targets, &args.out_dir)
            .map(|output| vec![output])
    };
    
    match result {
        Ok(outputs) => {
            // 显示成功信息
            for output in outputs {
                println!("✅ Compiled '{}' to:", output.skill_name);
                for target in &output.targets {
                    println!("   - {} ({})", target.display_name(), target.extension());
                }
                println!("   Output: {}", output.output_dir);
            }
            Ok(())
        }
        Err(e) => {
            // miette 自动渲染精美错误
            Err(e.into())
        }
    }
}

fn resolve_targets(args: &BuildArgs) -> Vec<TargetPlatform> {
    if args.all {
        return vec![
            TargetPlatform::Claude,
            TargetPlatform::Codex,
            TargetPlatform::Gemini,
            TargetPlatform::Kimi,
        ];
    }
    
    let mut targets = Vec::new();
    if args.claude { targets.push(TargetPlatform::Claude); }
    if args.codex { targets.push(TargetPlatform::Codex); }
    if args.gemini { targets.push(TargetPlatform::Gemini); }
    if args.kimi { targets.push(TargetPlatform::Kimi); }
    targets
}
```

---

## 4. 错误展示设计

### 4.1 Miette 错误报告

NSC 使用 `miette` 库提供精美的终端错误报告，包含精确行号和代码片段。

**错误报告示例**：

```text
Error: nexa_skill::parse::missing_required_section

  × The skill definition is missing a critical SOP section.
   ╭─[database-migration.md:15:1]
 14 │ 
 15 │ ## Examples
   · ╰──── We expected `## Procedures` before examples.
 16 │ > User: Do something...
   ╰────
  help: Agent SOPs require a numbered procedure list. Add a `## Procedures` heading.

Error: nexa_skill::ir::invalid_name

  × Invalid skill name format.
   ╭─[database-migration.md:2:7]
  1 │ ---
  2 │ name: Database-Migration
   ·       ──────────────────── Must be lowercase kebab-case
  3 │ version: "1.0.0"
   ╰────
  help: Use lowercase letters, numbers, and hyphens only. Example: "database-migration"
```

### 4.2 错误类型定义

```rust
// nexa-skill-core/src/error/diagnostic.rs

use miette::{Diagnostic, SourceSpan};
use thiserror::Error;

/// 解析错误
#[derive(Debug, Error, Diagnostic)]
pub enum ParseError {
    #[error("Missing YAML frontmatter")]
    #[diagnostic(
        code(nsc::parse::missing_frontmatter),
        help("Add YAML frontmatter at the beginning of the file, enclosed by ---")
    )]
    MissingFrontmatter,
    
    #[error("YAML parse error: {message}")]
    #[diagnostic(
        code(nsc::parse::yaml_error),
        help("Check YAML syntax: indentation, quotes, and special characters")
    )]
    YamlParseError {
        message: String,
        #[source_code]
        src: String,
        #[label("error location")]
        span: SourceSpan,
    },
    
    #[error("Missing required section: {section}")]
    #[diagnostic(
        code(nsc::parse::missing_section),
        help("Add a '{section}' section to your SKILL.md")
    )]
    MissingSection {
        section: String,
        #[source_code]
        src: String,
        #[label("expected here")]
        span: SourceSpan,
    },
}

/// IR 构建错误
#[derive(Debug, Error, Diagnostic)]
pub enum IRError {
    #[error("Invalid name format: '{name}'")]
    #[diagnostic(
        code(nsc::ir::invalid_name),
        help("Name must be kebab-case: lowercase letters, numbers, hyphens. 1-64 chars.")
    )]
    InvalidNameFormat {
        name: String,
        #[source_code]
        src: String,
        #[label("invalid name")]
        span: SourceSpan,
    },
    
    #[error("Description too long: {length} characters (max 1024)")]
    #[diagnostic(
        code(nsc::ir::description_length),
        help("Shorten the description to 1024 characters or less")
    )]
    DescriptionTooLong {
        length: usize,
        #[source_code]
        src: String,
        #[label("{length} chars")]
        span: SourceSpan,
    },
}

/// 分析错误
#[derive(Debug, Error, Diagnostic)]
pub enum AnalyzeError {
    #[error("MCP server '{server}' is not in the allowlist")]
    #[diagnostic(
        code(nsc::analyze::mcp_not_allowed),
        help("Add '{server}' to the MCP allowlist in nsc.toml, or remove it from the skill")
    )]
    MCPNotAllowed {
        server: String,
        #[source_code]
        src: String,
        #[label("declared here")]
        span: SourceSpan,
    },
    
    #[error("Dangerous keyword '{keyword}' without permission declaration")]
    #[diagnostic(
        code(nsc::analyze::missing_permission),
        help("Add a permission declaration for '{keyword}' operations")
    )]
    MissingPermission {
        keyword: String,
        #[source_code]
        src: String,
        #[label("dangerous keyword")]
        span: SourceSpan,
    },
}
```

### 4.3 错误级别与颜色

```rust
// nexa-skill-core/src/error/level.rs

use miette::Severity;

/// 错误级别
#[derive(Debug, Clone, Copy)]
pub enum ErrorLevel {
    /// 错误：阻断编译
    Error,
    /// 警告：不阻断编译
    Warning,
    /// 建议：可选修复
    Suggestion,
}

impl ErrorLevel {
    /// 转换为 miette Severity
    pub fn to_severity(&self) -> Severity {
        match self {
            ErrorLevel::Error => Severity::Error,
            ErrorLevel::Warning => Severity::Warning,
            ErrorLevel::Suggestion => Severity::Advice,
        }
    }
    
    /// ANSI 颜色代码
    pub fn color(&self) -> &'static str {
        match self {
            ErrorLevel::Error => "\x1b[31m", // 红色
            ErrorLevel::Warning => "\x1b[33m", // 黄色
            ErrorLevel::Suggestion => "\x1b[36m", // 青色
        }
    }
}
```

---

## 5. 输出格式设计

### 5.1 成功输出

**单文件编译**：
```text
✅ Compiled 'database-migration' to:
   - Claude Code (.xml)
   - OpenAI Codex (_schema.json)
   Output: ./build/database-migration/
```

**目录批量编译**：
```text
✅ Compiled 3 skills:

  database-migration
   - Claude Code (.xml)
   - OpenAI Codex (_schema.json)
   Output: ./build/database-migration/

  web-scraper
   - Claude Code (.xml)
   Output: ./build/web-scraper/

  pdf-processing
   - Claude Code (.xml)
   - Gemini CLI (.md)
   Output: ./build/pdf-processing/

Total: 7 target files generated in 1.2s
```

### 5.2 验证输出

**check 命令输出**：
```text
Checking database-migration.md...

✅ Format: Valid
✅ Frontmatter: Valid
✅ Procedures: 4 steps found
⚠️  Warning: input_schema references parameter 'columns' not used in examples
✅ Security: High level with HITL enabled

Result: PASSED (1 warning)
```

**validate 命令输出（详细）**：
```text
Validating database-migration.md...

=== Frontmatter Validation ===
✅ name: database-migration (valid kebab-case)
✅ version: 2.1.0 (valid semver)
✅ description: 156 chars (within limit)
✅ mcp_servers: 2 servers declared
   - neon-postgres-admin ✅ (in allowlist)
   - github-pr-creator ✅ (in allowlist)

=== Schema Validation ===
✅ input_schema: Valid JSON Schema
   Properties: target_table, migration_type, columns
⚠️  Warning: Parameter 'columns' not used in examples
   Suggestion: Add an example using 'columns' parameter

=== Procedures Validation ===
✅ Procedures section found
✅ 4 steps parsed
   Step 1: 提取目标表的当前 Schema
   Step 2: [CRITICAL] 编写 SQL 迁移脚本
   Step 3: 在本地沙盒环境试运行 SQL
   Step 4: [CRITICAL] 等待用户明确批准后执行

=== Security Validation ===
✅ security_level: high
✅ hitl_required: true (required for high level)
✅ permissions: 3 declared
   - database:postgres:staging:ALTER ✅
   - network:https://api.github.com/* ✅
   - exec:git:* ✅

=== Anti-Skill Injection ===
✅ 2 constraints injected
   - db-cascade (block): Never use CASCADE without approval
   - sql-injection (error): Never execute raw SQL without validation

=== Summary ===
Total: 15 checks
Passed: 14
Warnings: 1
Errors: 0

Result: PASSED
```

### 5.3 JSON 输出格式

```json
{
  "status": "passed",
  "skill": "database-migration",
  "checks": {
    "total": 15,
    "passed": 14,
    "warnings": 1,
    "errors": 0
  },
  "diagnostics": [
    {
      "level": "warning",
      "code": "nsc::schema::unused_parameter",
      "message": "Parameter 'columns' not used in examples",
      "location": {
        "file": "database-migration.md",
        "line": 8,
        "column": 5
      },
      "suggestion": "Add an example using 'columns' parameter"
    }
  ],
  "output": {
    "targets": ["claude", "codex"],
    "output_dir": "./build/database-migration/"
  }
}
```

---

## 6. 配置文件设计

### 6.1 nsc.toml 配置格式

```toml
# Nexa Skill Compiler 配置文件

[compiler]
# 默认输出目录
output_dir = "./build/"

# 默认目标平台（未指定时使用）
default_targets = ["claude", "codex"]

# 是否启用严格模式
strict_mode = false

# 是否生成签名文件
generate_signature = true

[mcp]
# MCP 服务器白名单
allowlist = [
    "filesystem-server",
    "github-server",
    "postgres-server",
    "neon-postgres-admin",
    "github-pr-creator",
]

# 是否允许未声明的 MCP
allow_undeclared = false

[security]
# 默认安全等级
default_security_level = "medium"

# 高危关键词列表
dangerous_keywords = [
    "rm -rf",
    "DROP",
    "DELETE",
    "TRUNCATE",
    "UPDATE",
    "ALTER",
    "GRANT",
    "shutdown",
    "reboot",
    "format",
]

# 是否强制 HITL for critical level
force_hitl_for_critical = true

[templates]
# 自定义模板目录
custom_templates_dir = "./templates/"

# 默认模板类型
default_template = "basic"

[output]
# 产物目录结构
manifest_filename = "manifest.json"
signature_filename = "signature.sha256"
target_dir = "target/"
assets_dir = "assets/"
meta_dir = "meta/"
```

### 6.2 配置加载

```rust
// nexa-skill-cli/src/config.rs

use std::path::Path;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub compiler: CompilerConfig,
    pub mcp: MCPConfig,
    pub security: SecurityConfig,
    pub templates: TemplatesConfig,
    pub output: OutputConfig,
}

impl Config {
    /// 从文件加载配置
    pub fn from_file(path: &Path) -> Result<Self, ConfigError> {
        let content = std::fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }
    
    /// 查找配置文件（按优先级）
    pub fn discover() -> Option<Self> {
        // 优先级：CLI 指定 > 当前目录 > 用户目录 > 默认
        let paths = [
            Path::new("./nsc.toml"),
            Path::new("~/.config/nsc/nsc.toml"),
            Path::new("/etc/nsc/nsc.toml"),
        ];
        
        for path in paths {
            if path.exists() {
                return Self::from_file(path).ok();
            }
        }
        
        None
    }
    
    /// 默认配置
    pub fn default() -> Self {
        Self {
            compiler: CompilerConfig::default(),
            mcp: MCPConfig::default(),
            security: SecurityConfig::default(),
            templates: TemplatesConfig::default(),
            output: OutputConfig::default(),
        }
    }
}
```

---

## 7. 进度与状态展示

### 7.1 编译进度条

```rust
// nexa-skill-cli/src/progress.rs

use indicatif::{ProgressBar, ProgressStyle};

/// 创建编译进度条
pub fn create_compile_progress(total: usize) -> ProgressBar {
    let pb = ProgressBar::new(total);
    pb.set_style(ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}")
        .progress_chars("=>-"));
    pb
}

/// 编译步骤进度
pub fn show_step_progress(pb: &ProgressBar, step: &str, current: usize, total: usize) {
    pb.set_position(current);
    pb.set_message(step);
}
```

**进度条示例**：
```text
⠋ [00:00:01] [=====>------------] 2/5 Parsing Procedures...
```

### 7.2 状态图标

| 状态 | 图标 | 颜色 |
|------|------|------|
| 成功 | ✅ | 绿色 |
| 警告 | ⚠️ | 黄色 |
| 错误 | ❌ | 红色 |
| 进行中 | ⠋ | 青色（旋转） |
| 跳过 | ⊘ | 灰色 |

---

## 8. 交互式确认

### 8.1 HITL 确认提示

```rust
// nexa-skill-cli/src/interactive.rs

use dialoguer::Confirm;

/// HITL 确认提示
pub fn request_hitl_confirmation(skill_name: &str, reason: &str) -> bool {
    println!("⚠️  Human-In-The-Loop Confirmation Required");
    println!("   Skill: {}", skill_name);
    println!("   Reason: {}", reason);
    
    Confirm::new()
        .with_prompt("Do you want to proceed with this skill execution?")
        .default(false)
        .interact()
        .unwrap_or(false)
}
```

**交互示例**：
```text
⚠️  Human-In-The-Loop Confirmation Required
   Skill: database-migration
   Reason: Security level is 'high' and involves database ALTER operations

Do you want to proceed with this skill execution? [y/N]
```

### 8.2 清理确认

```rust
/// 清理确认提示
pub fn request_clean_confirmation(files: &[String]) -> bool {
    println!("🗑️  The following files will be deleted:");
    for file in files {
        println!("   - {}", file);
    }
    
    Confirm::new()
        .with_prompt("Are you sure you want to delete these files?")
        .default(false)
        .interact()
        .unwrap_or(false)
}
```

---

## 9. 相关文档

- [ERROR_HANDLING.md](ERROR_HANDLING.md) - 错误处理与诊断系统
- [DEVELOPMENT_GUIDE.md](DEVELOPMENT_GUIDE.md) - CLI 开发指南
- [API_REFERENCE.md](API_REFERENCE.md) - CLI API 定义