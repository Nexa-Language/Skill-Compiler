# API 参考文档

> **公开 API、核心 Trait 定义、类型签名与使用示例**

---

## 1. API 概述

NSC 的 API 设计遵循以下原则：

| 原则 | 描述 |
|------|------|
| **最小公开** | 仅公开必要的 API，内部实现细节保持私有 |
| **类型安全** | 所有公开 API 都有完整的类型签名 |
| **错误透明** | 错误类型公开，便于调用者处理 |
| **文档完整** | 所有公开 API 都有文档注释 |

---

## 2. 核心 API

### 2.1 Compiler API

```rust
// nexa-skill-core/src/compiler.rs

use crate::ir::ValidatedSkillIR;
use crate::backend::TargetPlatform;
use crate::error::{CompileError, CompileResult};

/// 编译器主入口
/// 
/// 提供编译 SKILL.md 文件和目录的能力
/// 
/// # Example
/// 
/// ```
/// use nexa_skill_core::{Compiler, TargetPlatform};
/// 
/// let compiler = Compiler::new();
/// let result = compiler.compile_file(
///     "path/to/skill.md",
///     &[TargetPlatform::Claude, TargetPlatform::Codex],
///     "./build/"
/// );
/// ```
pub struct Compiler {
    emitter_registry: EmitterRegistry,
    config: Config,
}

impl Compiler {
    /// 创建默认配置的编译器
    /// 
    /// # Example
    /// 
    /// ```
    /// let compiler = Compiler::new();
    /// ```
    pub fn new() -> Self;
    
    /// 使用自定义配置创建编译器
    /// 
    /// # Arguments
    /// 
    /// * `config` - 编译器配置
    /// 
    /// # Example
    /// 
    /// ```
    /// let config = Config {
    ///     strict_mode: true,
    ///     ..Default::default()
    /// };
    /// let compiler = Compiler::with_config(config);
    /// ```
    pub fn with_config(config: Config) -> Self;
    
    /// 编译单个 SKILL.md 文件
    /// 
    /// # Arguments
    /// 
    /// * `input_path` - SKILL.md 文件路径
    /// * `targets` - 目标平台列表
    /// * `output_dir` - 输出目录路径
    /// 
    /// # Returns
    /// 
    /// 成功返回 `CompileOutput`，失败返回 `CompileError`
    /// 
    /// # Errors
    /// 
    /// - `CompileError::ParseError` - 源文件解析失败
    /// - `CompileError::ValidationError` - 验证失败
    /// - `CompileError::EmitError` - 产物生成失败
    /// - `CompileError::IOError` - 文件操作失败
    /// 
    /// # Example
    /// 
    /// ```
    /// use nexa_skill_core::{Compiler, TargetPlatform};
    /// 
    /// let compiler = Compiler::new();
    /// let output = compiler.compile_file(
    ///     "skills/database-migration.md",
    ///     &[TargetPlatform::Claude],
    ///     "./build/"
    /// )?;
    /// 
    /// println!("Compiled to: {}", output.output_dir);
    /// ```
    pub fn compile_file(
        &self,
        input_path: &str,
        targets: &[TargetPlatform],
        output_dir: &str,
    ) -> CompileResult<CompileOutput>;
    
    /// 编译技能目录
    /// 
    /// # Arguments
    /// 
    /// * `input_dir` - 包含 SKILL.md 的目录路径
    /// * `targets` - 目标平台列表
    /// * `output_dir` - 输出目录路径
    /// 
    /// # Returns
    /// 
    /// 成功返回所有编译结果的列表
    /// 
    /// # Example
    /// 
    /// ```
    /// let outputs = compiler.compile_dir(
    ///     "./skills/",
    ///     &[TargetPlatform::Claude, TargetPlatform::Codex],
    ///     "./build/"
    /// )?;
    /// 
    /// for output in outputs {
    ///     println!("Compiled: {}", output.skill_name);
    /// }
    /// ```
    pub fn compile_dir(
        &self,
        input_dir: &str,
        targets: &[TargetPlatform],
        output_dir: &str,
    ) -> CompileResult<Vec<CompileOutput>>;
    
    /// 验证 SKILL.md 但不生成产物
    /// 
    /// # Arguments
    /// 
    /// * `input_path` - SKILL.md 文件路径
    /// 
    /// # Returns
    /// 
    /// 返回验证结果和诊断信息
    /// 
    /// # Example
    /// 
    /// ```
    /// let result = compiler.check("skills/test.md")?;
    /// 
    /// if result.is_valid() {
    ///     println!("Validation passed");
    /// } else {
    ///     for diagnostic in result.diagnostics() {
    ///         println!("{}", diagnostic);
    ///     }
    /// }
    /// ```
    pub fn check(&self, input_path: &str) -> CompileResult<CheckResult>;
}

/// 编译输出结果
#[derive(Debug, Clone)]
pub struct CompileOutput {
    /// 技能名称
    pub skill_name: String,
    
    /// 输出目录路径
    pub output_dir: String,
    
    /// 生成的目标平台
    pub targets: Vec<TargetPlatform>,
    
    /// 编译耗时（毫秒）
    pub duration_ms: u64,
    
    /// 生成的文件列表
    pub generated_files: Vec<String>,
}

/// 验证结果
#[derive(Debug, Clone)]
pub struct CheckResult {
    /// 是否通过验证
    pub is_valid: bool,
    
    /// 诊断信息列表
    pub diagnostics: Vec<Diagnostic>,
    
    /// 验证的技能信息
    pub skill_info: Option<SkillInfo>,
}

impl CheckResult {
    /// 是否通过验证
    pub fn is_valid(&self) -> bool {
        self.is_valid
    }
    
    /// 获取所有诊断信息
    pub fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }
    
    /// 获取错误数量
    pub fn error_count(&self) -> usize {
        self.diagnostics.iter().filter(|d| d.is_error()).count()
    }
    
    /// 获取警告数量
    pub fn warning_count(&self) -> usize {
        self.diagnostics.iter().filter(|d| d.is_warning()).count()
    }
}
```

### 2.2 Emitter Trait

```rust
// nexa-skill-core/src/backend/emitter.rs

use crate::ir::ValidatedSkillIR;
use crate::error::EmitError;

/// 发射器 Trait
/// 
/// 所有后端适配器必须实现此接口
/// 
/// # Implementation Guide
/// 
/// 实现 `Emitter` Trait 需要完成以下步骤：
/// 
/// 1. 实现 `target()` 返回目标平台标识
/// 2. 实现 `emit()` 生成产物字符串
/// 3. 实现 `file_extension()` 返回产物文件扩展名
/// 
/// # Example
/// 
/// ```
/// use nexa_skill_core::backend::{Emitter, TargetPlatform};
/// use nexa_skill_core::ir::ValidatedSkillIR;
/// use nexa_skill_core::error::EmitError;
/// 
/// pub struct MyEmitter;
/// 
/// impl Emitter for MyEmitter {
///     fn target(&self) -> TargetPlatform {
///         TargetPlatform::Custom("my-platform")
///     }
///     
///     fn emit(&self, ir: &ValidatedSkillIR) -> Result<String, EmitError> {
///         // 生成产物逻辑
///         Ok(format!("Skill: {}", ir.as_ref().name))
///     }
///     
///     fn file_extension(&self) -> &'static str {
///         ".txt"
///     }
/// }
/// ```
pub trait Emitter: Send + Sync {
    /// 目标平台标识
    /// 
    /// # Example
    /// 
    /// ```
    /// fn target(&self) -> TargetPlatform {
    ///     TargetPlatform::Claude
    /// }
    /// ```
    fn target(&self) -> TargetPlatform;
    
    /// 将 ValidatedSkillIR 发射为字符串
    /// 
    /// # Arguments
    /// 
    /// * `ir` - 经过验证的 SkillIR
    /// 
    /// # Returns
    /// 
    /// 成功返回产物字符串，失败返回 `EmitError`
    /// 
    /// # Errors
    /// 
    /// - `EmitError::TemplateError` - 模板渲染失败
    /// - `EmitError::SerializationError` - 序列化失败
    fn emit(&self, ir: &ValidatedSkillIR) -> Result<String, EmitError>;
    
    /// 发射产物文件扩展名
    /// 
    /// # Example
    /// 
    /// ```
    /// fn file_extension(&self) -> &'static str {
    ///     ".xml"
    /// }
    /// ```
    fn file_extension(&self) -> &'static str;
    
    /// 是否需要生成 manifest.json
    /// 
    /// 默认返回 `true`
    fn requires_manifest(&self) -> bool {
        true
    }
    
    /// 发射前的预处理
    /// 
    /// 默认无操作，子类可覆盖
    fn pre_process(&self, _ir: &ValidatedSkillIR) -> Result<(), EmitError> {
        Ok(())
    }
    
    /// 发射后的后处理
    /// 
    /// 默认无操作，子类可覆盖
    fn post_process(&self, content: &str) -> Result<String, EmitError> {
        Ok(content.to_string())
    }
}

/// 目标平台枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TargetPlatform {
    /// Claude (Anthropic)
    Claude,
    
    /// Codex (OpenAI)
    Codex,
    
    /// Gemini (Google)
    Gemini,
    
    /// Kimi (Moonshot)
    Kimi,
    
    /// GitHub Copilot
    Copilot,
    
    /// VS Code Agent
    VSCode,
    
    /// 自定义平台
    Custom(&'static str),
}

impl TargetPlatform {
    /// 获取平台标识符（用于 CLI flag）
    pub fn slug(&self) -> &'static str;
    
    /// 获取产物文件扩展名
    pub fn extension(&self) -> &'static str;
    
    /// 获取平台显示名称
    pub fn display_name(&self) -> &'static str;
}
```

### 2.3 Analyzer Trait

```rust
// nexa-skill-core/src/analyzer/mod.rs

use crate::ir::SkillIR;
use crate::error::{Diagnostic, AnalyzeError};

/// 分析器 Trait
/// 
/// 所有分析器必须实现此接口
/// 
/// # Implementation Guide
/// 
/// 实现 `Analyzer` Trait 需要完成以下步骤：
/// 
/// 1. 实现 `name()` 返回分析器名称
/// 2. 实现 `analyze()` 执行分析逻辑
/// 3. 可选实现 `priority()` 设置执行优先级
/// 
/// # Example
/// 
/// ```
/// use nexa_skill_core::analyzer::Analyzer;
/// use nexa_skill_core::ir::SkillIR;
/// use nexa_skill_core::error::{Diagnostic, AnalyzeError};
/// 
/// pub struct MyAnalyzer;
/// 
/// impl Analyzer for MyAnalyzer {
///     fn name(&self) -> &'static str {
///         "my-analyzer"
///     }
///     
///     fn analyze(&self, ir: &mut SkillIR) -> Result<Vec<Diagnostic>, AnalyzeError> {
///         let mut diagnostics = Vec::new();
///         
///         // 分析逻辑
///         if ir.name.is_empty() {
///             diagnostics.push(Diagnostic::error(
///                 "Name cannot be empty",
///                 "my-analyzer::empty-name"
///             ));
///         }
///         
///         Ok(diagnostics)
///     }
/// }
/// ```
pub trait Analyzer {
    /// 分析器名称
    /// 
    /// 用于日志和错误报告
    fn name(&self) -> &'static str;
    
    /// 执行分析
    /// 
    /// # Arguments
    /// 
    /// * `ir` - 可变引用的 SkillIR，分析器可以修改 IR
    /// 
    /// # Returns
    /// 
    /// 返回诊断列表（警告和错误）
    /// 
    /// # Errors
    /// 
    /// 当分析过程本身出错时返回 `AnalyzeError`
    fn analyze(&self, ir: &mut SkillIR) -> Result<Vec<Diagnostic>, AnalyzeError>;
    
    /// 分析器优先级
    /// 
    /// 数字越小优先级越高，越先执行
    /// 
    /// 默认值：100
    fn priority(&self) -> u8 {
        100
    }
}
```

---

## 3. IR 数据结构 API

### 3.1 SkillIR

```rust
// nexa-skill-core/src/ir/skill_ir.rs

use serde::{Serialize, Deserialize};
use std::sync::Arc;

/// Nexa Skill Compiler 核心中间表示
/// 
/// 这是编译管线中所有阶段的数据交换载体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillIR {
    // ===== 元数据与路由 =====
    pub name: Arc<str>,
    pub version: Arc<str>,
    pub description: String,
    
    // ===== 接口与 MCP =====
    pub mcp_servers: Vec<Arc<str>>,
    pub input_schema: Option<serde_json::Value>,
    pub output_schema: Option<serde_json::Value>,
    
    // ===== 安全与控制 =====
    pub hitl_required: bool,
    pub pre_conditions: Vec<String>,
    pub post_conditions: Vec<String>,
    pub fallbacks: Vec<String>,
    pub permissions: Vec<Permission>,
    pub security_level: SecurityLevel,
    
    // ===== 执行逻辑 =====
    pub context_gathering: Vec<String>,
    pub procedures: Vec<ProcedureStep>,
    pub few_shot_examples: Vec<Example>,
    
    // ===== 编译期注入 =====
    pub anti_skill_constraints: Vec<Constraint>,
}

impl SkillIR {
    /// 创建新的 SkillIR Builder
    /// 
    /// # Example
    /// 
    /// ```
    /// use nexa_skill_core::ir::SkillIR;
    /// 
    /// let ir = SkillIR::builder()
    ///     .name("test-skill")
    ///     .version("1.0.0")
    ///     .description("A test skill")
    ///     .build();
    /// ```
    pub fn builder() -> SkillIRBuilder;
    
    /// 验证 SkillIR 的完整性
    /// 
    /// # Returns
    /// 
    /// 返回验证结果和诊断信息
    pub fn validate(&self) -> ValidationResult;
    
    /// 获取所有声明的权限
    pub fn declared_permissions(&self) -> &[Permission];
    
    /// 检查是否需要 HITL
    pub fn requires_hitl(&self) -> bool;
    
    /// 获取关键步骤
    pub fn critical_steps(&self) -> Vec<&ProcedureStep>;
}

/// SkillIR Builder
pub struct SkillIRBuilder {
    // 内部字段...
}

impl SkillIRBuilder {
    pub fn name(mut self, name: impl Into<String>) -> Self;
    pub fn version(mut self, version: impl Into<String>) -> Self;
    pub fn description(mut self, description: impl Into<String>) -> Self;
    pub fn mcp_servers(mut self, servers: Vec<String>) -> Self;
    pub fn input_schema(mut self, schema: serde_json::Value) -> Self;
    pub fn hitl_required(mut self, required: bool) -> Self;
    pub fn security_level(mut self, level: SecurityLevel) -> Self;
    pub fn procedure(mut self, step: ProcedureStep) -> Self;
    pub fn permission(mut self, permission: Permission) -> Self;
    
    /// 构建 SkillIR
    /// 
    /// # Errors
    /// 
    /// 当必填字段缺失时返回错误
    pub fn build(self) -> Result<SkillIR, IRError>;
}
```

### 3.2 ProcedureStep

```rust
// nexa-skill-core/src/ir/procedure.rs

/// 执行步骤定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcedureStep {
    /// 步骤序号（从 1 开始）
    pub order: u32,
    
    /// 步骤指令文本
    pub instruction: String,
    
    /// 是否为关键步骤
    pub is_critical: bool,
    
    /// 步骤级别约束
    pub constraints: Vec<String>,
}

impl ProcedureStep {
    /// 创建新的步骤
    /// 
    /// # Example
    /// 
    /// ```
    /// use nexa_skill_core::ir::ProcedureStep;
    /// 
    /// let step = ProcedureStep::new(1, "First step");
    /// ```
    pub fn new(order: u32, instruction: impl Into<String>) -> Self;
    
    /// 标记为关键步骤
    /// 
    /// # Example
    /// 
    /// ```
    /// let step = ProcedureStep::new(1, "Critical step").critical();
    /// ```
    pub fn critical(mut self) -> Self;
    
    /// 添加约束
    /// 
    /// # Example
    /// 
    /// ```
    /// let step = ProcedureStep::new(1, "Step with constraint")
    ///     .with_constraint("Must complete within 10 seconds");
    /// ```
    pub fn with_constraint(mut self, constraint: impl Into<String>) -> Self;
}
```

### 3.3 Permission

```rust
// nexa-skill-core/src/ir/permission.rs

/// 权限声明
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Permission {
    /// 权限类型
    pub kind: PermissionKind,
    
    /// 权限范围
    pub scope: String,
    
    /// 权限描述
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    
    /// 是否只读
    #[serde(default)]
    pub read_only: bool,
}

impl Permission {
    /// 创建新的权限声明
    /// 
    /// # Example
    /// 
    /// ```
    /// use nexa_skill_core::ir::{Permission, PermissionKind};
    /// 
    /// let perm = Permission::new(PermissionKind::Network, "https://api.example.com/*");
    /// ```
    pub fn new(kind: PermissionKind, scope: impl Into<String>) -> Self;
    
    /// 添加描述
    pub fn with_description(mut self, description: impl Into<String>) -> Self;
    
    /// 标记为只读
    pub fn read_only(mut self) -> Self;
}

/// 权限类型枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PermissionKind {
    Network,
    #[serde(alias = "fs")]
    FileSystem,
    #[serde(alias = "db")]
    Database,
    #[serde(alias = "exec")]
    Execute,
    MCP,
    Environment,
    Unknown,
}

impl PermissionKind {
    /// 获取显示名称
    pub fn display_name(&self) -> &'static str;
    
    /// 获取 scope 格式说明
    pub fn scope_format(&self) -> &'static str;
}
```

---

## 4. 错误处理 API

### 4.1 Diagnostic

```rust
// nexa-skill-core/src/error/diagnostic.rs

/// NSC 诊断信息
#[derive(Debug, Clone, Error, Diagnostic)]
pub struct Diagnostic {
    // 内部字段...
}

impl Diagnostic {
    /// 创建错误级别诊断
    /// 
    /// # Example
    /// 
    /// ```
    /// use nexa_skill_core::error::Diagnostic;
    /// 
    /// let diag = Diagnostic::error("Something went wrong", "my-module::error-code");
    /// ```
    pub fn error(message: impl Into<String>, code: impl Into<String>) -> Self;
    
    /// 创建警告级别诊断
    /// 
    /// # Example
    /// 
    /// ```
    /// let diag = Diagnostic::warning("This is a warning", "my-module::warning-code");
    /// ```
    pub fn warning(message: impl Into<String>, code: impl Into<String>) -> Self;
    
    /// 创建建议级别诊断
    pub fn advice(message: impl Into<String>, code: impl Into<String>) -> Self;
    
    /// 添加修复建议
    /// 
    /// # Example
    /// 
    /// ```
    /// let diag = Diagnostic::error("Invalid name", "ir::invalid-name")
    ///     .with_help("Use kebab-case format: lowercase letters and hyphens");
    /// ```
    pub fn with_help(mut self, help: impl Into<String>) -> Self;
    
    /// 添加源代码位置
    pub fn with_source(mut self, file_path: impl Into<String>, content: impl Into<String>) -> Self;
    
    /// 添加标签位置
    pub fn with_label(mut self, label: impl Into<String>, line: usize, column: usize) -> Self;
    
    /// 添加文档链接
    pub fn with_url(mut self, url: impl Into<String>) -> Self;
    
    /// 是否为阻断性错误
    pub fn is_blocking(&self) -> bool;
    
    /// 是否为错误级别
    pub fn is_error(&self) -> bool;
    
    /// 是否为警告级别
    pub fn is_warning(&self) -> bool;
    
    /// 获取错误代码
    pub fn code(&self) -> &str;
    
    /// 获取错误消息
    pub fn message(&self) -> &str;
}
```

### 4.2 错误类型

```rust
// nexa-skill-core/src/error/mod.rs

/// 编译错误
#[derive(Debug, Error)]
pub enum CompileError {
    /// 解析错误
    #[error("Parse error: {0}")]
    ParseError(#[from] ParseError),
    
    /// IR 构建错误
    #[error("IR error: {0}")]
    IRError(#[from] IRError),
    
    /// 分析错误
    #[error("Analysis error: {0}")]
    AnalyzeError(#[from] AnalyzeError),
    
    /// 发射错误
    #[error("Emit error: {0}")]
    EmitError(#[from] EmitError),
    
    /// IO 错误
    #[error("IO error: {0}")]
    IOError(#[from] std::io::Error),
}

/// 编译结果类型
pub type CompileResult<T> = Result<T, CompileError>;
```

---

## 5. 安全 API

### 5.1 SecurityLevel

```rust
// nexa-skill-core/src/security/level.rs

/// 安全等级
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SecurityLevel {
    Low,
    Medium,
    High,
    Critical,
}

impl SecurityLevel {
    /// 是否需要强制 HITL
    pub fn requires_hitl(&self) -> bool;
    
    /// 是否禁止自动执行
    pub fn blocks_auto_execution(&self) -> bool;
    
    /// 获取审计检查项
    pub fn audit_checks(&self) -> Vec<AuditCheck>;
}
```

### 5.2 HITLManager

```rust
// nexa-skill-core/src/security/hitl.rs

/// HITL 管理器
pub struct HITLManager {
    interactive: bool,
    timeout: u64,
}

impl HITLManager {
    /// 创建新的 HITL 管理器
    /// 
    /// # Arguments
    /// 
    /// * `interactive` - 是否启用交互模式
    /// * `timeout` - 超时时间（秒）
    pub fn new(interactive: bool, timeout: u64) -> Self;
    
    /// 检查是否需要 HITL
    /// 
    /// # Returns
    /// 
    /// 如果需要 HITL，返回 `Some(HITLRequest)`，否则返回 `None`
    pub fn requires_hitl(&self, ir: &SkillIR) -> Option<HITLRequest>;
    
    /// 请求用户审批
    /// 
    /// # Arguments
    /// 
    /// * `request` - HITL 请求
    /// 
    /// # Returns
    /// 
    /// 返回用户审批结果
    pub fn request_approval(&self, request: &HITLRequest) -> HITLResult;
}

/// HITL 审批请求
pub struct HITLRequest {
    pub skill_name: String,
    pub reason: HITLReason,
    pub risk_description: String,
    pub affected_steps: Vec<u32>,
    pub permissions: Vec<String>,
}

/// HITL 审批结果
pub enum HITLResult {
    Approved,
    Rejected { reason: String },
    Timeout,
}
```

---

## 6. 使用示例

### 6.1 基础编译

```rust
use nexa_skill_core::{Compiler, TargetPlatform};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 创建编译器
    let compiler = Compiler::new();
    
    // 编译单个文件
    let output = compiler.compile_file(
        "skills/database-migration.md",
        &[TargetPlatform::Claude, TargetPlatform::Codex],
        "./build/"
    )?;
    
    println!("Compiled '{}' to:", output.skill_name);
    for target in &output.targets {
        println!("  - {}", target.display_name());
    }
    
    Ok(())
}
```

### 6.2 自定义 Emitter

```rust
use nexa_skill_core::backend::{Emitter, TargetPlatform, EmitterRegistry};
use nexa_skill_core::ir::ValidatedSkillIR;
use nexa_skill_core::error::EmitError;

// 定义自定义 Emitter
pub struct MyCustomEmitter;

impl Emitter for MyCustomEmitter {
    fn target(&self) -> TargetPlatform {
        TargetPlatform::Custom("my-custom-platform")
    }
    
    fn emit(&self, ir: &ValidatedSkillIR) -> Result<String, EmitError> {
        let inner = ir.as_ref();
        Ok(format!(
            "# {}\n\n{}\n",
            inner.name, inner.description
        ))
    }
    
    fn file_extension(&self) -> &'static str {
        ".md"
    }
}

// 注册自定义 Emitter
fn main() {
    let mut registry = EmitterRegistry::default();
    registry.register(Box::new(MyCustomEmitter));
    
    // 使用自定义 Emitter
    let compiler = Compiler::with_registry(registry);
    // ...
}
```

### 6.3 自定义 Analyzer

```rust
use nexa_skill_core::analyzer::Analyzer;
use nexa_skill_core::ir::SkillIR;
use nexa_skill_core::error::{Diagnostic, AnalyzeError};

// 定义自定义 Analyzer
pub struct NamingConventionAnalyzer;

impl Analyzer for NamingConventionAnalyzer {
    fn name(&self) -> &'static str {
        "naming-convention"
    }
    
    fn priority(&self) -> u8 {
        10  // 高优先级
    }
    
    fn analyze(&self, ir: &mut SkillIR) -> Result<Vec<Diagnostic>, AnalyzeError> {
        let mut diagnostics = Vec::new();
        
        // 检查 name 是否符合 kebab-case
        if !is_kebab_case(&ir.name) {
            diagnostics.push(
                Diagnostic::warning(
                    format!("Name '{}' should be in kebab-case", ir.name),
                    "naming::kebab-case"
                ).with_help("Use lowercase letters, numbers, and hyphens")
            );
        }
        
        Ok(diagnostics)
    }
}

fn is_kebab_case(s: &str) -> bool {
    s.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}
```

---

## 7. 相关文档

- [ARCHITECTURE.md](ARCHITECTURE.md) - 系统架构
- [COMPILER_PIPELINE.md](COMPILER_PIPELINE.md) - 编译管线
- [IR_DESIGN.md](IR_DESIGN.md) - IR 数据结构
- [BACKEND_ADAPTERS.md](BACKEND_ADAPTERS.md) - Emitter 实现
- [SECURITY_MODEL.md](SECURITY_MODEL.md) - 安全 API