# 后端适配器设计

> **Emitter Trait 定义、各平台适配策略与产物生成细节**
>
> **重要更新**：基于《高级提示词工程格式与智能体技能架构》调研报告（2026-04），本文档已全面重构后端适配策略，消除"格式税"并实现编译期AST优化。

> ⚠️ **实现状态声明 (Updated 2026-04-16):** 本文档描述的设计规格大部分已在源码中实现。部分早期设计（如 Codex 双负载架构、Analyzer trait 统一接口）在实现过程中进行了简化重构，详见下方各节标注。完整审查见 [审查报告](../plans/RE_AUDIT_REPORT_20260416.md)。实现状态如下：
>
> | 文档描述 | 实现状态 |
> |----------|---------|
> | `Emitter` Trait 同步方法 `fn emit()` | ✅ sync trait，无 async_trait |
> | `pre_process()` / `post_process()` 方法 | ✅ 默认方法存在 |
> | `EmitterRegistry` | ✅ `registry.rs` 已创建 |
> | `KimiEmitter::new()` | ✅ 独立 KimiEmitter 实现 |
> | `Copilot` / `VSCode` 平台 | ⏳ 暂不实现（未来扩展） |
> | Askama 模板引擎 | ✅ 4个 .j2 模板 + context structs |
> | 编译期模板错误检查 | ✅ Askama 编译期检查 |
> | Emitter `Send + Sync` | ✅ sync trait，无 async_trait |
> | Codex 双负载生成 (supports_dual_payload/emit_schema_payload) | ❌ 未实现 — 已重构为纯Markdown |
> | Analyzer trait 统一接口 | ❌ 未实现 — 使用独立 Analyzer struct + 各子分析器 Vec<Diagnostic> |

---

## 1. 后端设计概述

Backend 阶段负责将 `ValidatedSkillIR` 序列化为特定平台的原生表示。NSC 采用**多态发射器 (Polymorphic Emitter)** 架构，通过统一的 `Emitter` Trait 支持不同目标平台。

### 1.1 设计原则

| 原则 | 描述 | 学术依据 |
|------|------|----------|
| **消除格式税** | 避免强制模型解析复杂JSON输入，防止高达40%的性能衰退 | Format Tax 研究 (2025) |
| **解耦推理与格式化** | 输入端使用Markdown进行自由态推理，输出端通过API强制约束 | OpenAI Structured Outputs 最佳实践 |
| **AST优化注入** | 编译期检测嵌套数据结构，自动选择最优格式 | Gemini 嵌套数据准确率测试 |
| **平台原生** | 每个平台生成最适合其底层模型的格式 | 模型微调惯性分析 |
| **编译期检查** | 使用 Askama 模板引擎，模板错误在编译期发现 | Rust 类型系统优势 |
| **可扩展** | 新平台只需实现 `Emitter` Trait | Trait 抽象设计 |

### 1.2 平台适配策略总览（基于实证研究）

| 平台 | 底层模型 | 输入格式偏好 | 输出格式 | 核心策略 | 学术依据 |
|------|----------|--------------|----------|----------|----------|
| **Claude** | Claude 4.6 Opus | XML标签分层 | XML | XML原教旨主义，强标签嵌套降低认知负载 | Anthropic官方指南 + 23%推理准确率提升 |
| **Codex** | GPT-5.4 | **结构化Markdown** | Markdown | **纯Markdown输出**：指令结构化MD，Schema由API层负责 | Format Tax消除 + 词元效率优化 |
| **Gemini** | Gemini 3.1 Pro | Markdown + **YAML嵌套数据** | Markdown | **AST优化**：嵌套数据自动转YAML | YAML 51.9% > MD 48.2% > JSON 43.1% |
| **Kimi** | K2.5 | 纯文本/完整Markdown | Markdown | 海量上下文，弱约束强推理 | 超长上下文优势 |

> **关键发现**：GPT系列存在严重的"格式税"问题，强制JSON输入会导致模型将注意力从逻辑推理转移到语法校验，造成性能大幅衰退。Codex适配器必须采用"解耦推理与格式化"策略。

---

## 2. Emitter Trait 定义

### 2.1 核心 Trait

```rust
// nexa-skill-core/src/backend/emitter.rs

use crate::ir::ValidatedSkillIR;
use crate::error::EmitError;

/// 发射器 Trait
/// 
/// 所有后端适配器必须实现此接口
pub trait Emitter {
    /// 目标平台标识
    fn target(&self) -> TargetPlatform;
    
    /// 将 ValidatedSkillIR 发射为字符串
    fn emit(&self, ir: &ValidatedSkillIR) -> Result<String, EmitError>;
    
    /// 发射产物文件扩展名
    fn file_extension(&self) -> &'static str;
    
    /// 是否需要生成 manifest.json
    fn requires_manifest(&self) -> bool {
        true
    }
    
    /// 发射前的预处理
    fn pre_process(&self, ir: &ValidatedSkillIR) -> Result<(), EmitError> {
        // 默认无预处理
        Ok(())
    }
    
    /// 发射后的后处理
    fn post_process(&self, content: &str) -> Result<String, EmitError> {
        // 默认无后处理
        Ok(content.to_string())
    }
    
    /// 生成额外资产文件（如YAML配置等）
    ///
    /// 返回 (相对路径, 内容) 的列表
    /// Gemini: assets/input.yaml, assets/output.yaml, assets/config.yaml
    /// Codex: 无额外资产（纯Markdown输出）
    fn generate_assets(&self, _ir: &ValidatedSkillIR) -> Vec<(String, String)> {
        Vec::new()  // 默认无额外资产
    }
}

/// 目标平台枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TargetPlatform {
    /// Claude (Anthropic) - XML偏好
    Claude,
    
    /// Codex (OpenAI) - Markdown输入 + JSON Schema输出
    Codex,
    
    /// Gemini (Google) - Markdown + YAML嵌套数据
    Gemini,
    
    /// Kimi (Moonshot) - 纯文本/完整Markdown
    Kimi,
    
    // Copilot / VSCode: 暂不实现，作为未来扩展
}

impl TargetPlatform {
    /// 获取平台标识符（用于 CLI flag）
    pub fn slug(&self) -> &'static str {
        match self {
            TargetPlatform::Claude => "claude",
            TargetPlatform::Codex => "codex",
            TargetPlatform::Gemini => "gemini",
            TargetPlatform::Kimi => "kimi",
        }
    }
    
    /// 获取产物文件扩展名
    pub fn extension(&self) -> &'static str {
        match self {
            TargetPlatform::Claude => ".xml",
            TargetPlatform::Codex => ".md",  // 主负载为Markdown
            TargetPlatform::Gemini => ".md",
            TargetPlatform::Kimi => ".md",
        }
    }
    
    /// 注意：早期设计中存在 schema_extension() 方法用于 Codex 双负载架构，
    /// 但实际实现中已移除。Codex 仅生成纯Markdown，不生成JSON Schema资产。
    
    /// 获取平台显示名称
    pub fn display_name(&self) -> &'static str {
        match self {
            TargetPlatform::Claude => "Claude Code",
            TargetPlatform::Codex => "OpenAI Codex",
            TargetPlatform::Gemini => "Gemini CLI",
            TargetPlatform::Kimi => "Kimi CLI",
        }
    }
}
```

### 2.2 Emitter Registry

```rust
// nexa-skill-core/src/backend/registry.rs

use std::collections::HashMap;
use crate::backend::{Emitter, TargetPlatform, ClaudeEmitter, CodexEmitter, GeminiEmitter};
use crate::error::EmitError;

/// 发射器注册表
/// 
/// 管理所有可用的 Emitter 实例
pub struct EmitterRegistry {
    emitters: HashMap<TargetPlatform, Box<dyn Emitter>>,
}

impl EmitterRegistry {
    /// 创建默认注册表
    pub fn default() -> Self {
        let mut emitters: HashMap<TargetPlatform, Box<dyn Emitter>> = HashMap::new();
        
        emitters.insert(TargetPlatform::Claude, Box::new(ClaudeEmitter::new()));
        emitters.insert(TargetPlatform::Codex, Box::new(CodexEmitter::new()));
        emitters.insert(TargetPlatform::Gemini, Box::new(GeminiEmitter::new()));
        emitters.insert(TargetPlatform::Kimi, Box::new(KimiEmitter::new()));
        
        Self { emitters }
    }
    
    /// 注册自定义 Emitter
    pub fn register(&mut self, emitter: Box<dyn Emitter>) {
        self.emitters.insert(emitter.target(), emitter);
    }
    
    /// 获取指定平台的 Emitter
    pub fn get(&self, target: &TargetPlatform) -> Result<&dyn Emitter, EmitError> {
        self.emitters
            .get(target)
            .map(|e| e.as_ref())
            .ok_or_else(|| EmitError::UnsupportedTarget(target.display_name()))
    }
    
    /// 获取所有支持的平台
    pub fn supported_platforms(&self) -> Vec<TargetPlatform> {
        self.emitters.keys().cloned().collect()
    }
}
```

---

## 3. Claude Emitter

### 3.1 适配策略（基于Anthropic官方指南）

Claude 模型在对齐训练时深度绑定了 XML 标签，对嵌套清晰的 XML 结构有极高的指令遵循度。实证研究表明，使用XML搭建脚手架的提示词在数学推理任务中比纯JSON格式高出**23%**的准确率。

**核心策略**：
- **XML 原教旨主义**：使用严格嵌套的 XML 树结构
- **边界清晰**：使用 `<system>`, `<skill_definition>`, `<execution_steps>` 等标签划分信息块
- **防御性强**：将约束包装在 `<strict_constraints>` 中，防止提示词注入
- **认知负载降低**：XML明确的闭合标签为模型提供坚实的认知边界

> **学术依据**：Anthropic官方提示词工程指南明确将XML风格标签作为"第一类最佳实践"。Claude在训练阶段被专门优化以特别关注XML标签。

### 3.2 技术实现

```rust
// nexa-skill-core/src/backend/claude.rs

use crate::backend::{Emitter, TargetPlatform};
use crate::ir::ValidatedSkillIR;
use crate::error::EmitError;
use askama::Template;

/// Claude XML 模板
#[derive(Template)]
#[template(path = "claude_xml.html")]
struct ClaudeTemplate {
    name: String,
    description: String,
    version: String,
    hitl_required: bool,
    procedures: Vec<ProcedureView>,
    constraints: Vec<ConstraintView>,
    examples: Vec<ExampleView>,
    input_schema: Option<String>,
}

/// Procedure 视图结构
struct ProcedureView {
    order: u32,
    instruction: String,
    is_critical: bool,
}

/// Constraint 视图结构
struct ConstraintView {
    source: String,
    content: String,
    level: String,
}

/// Example 视图结构
struct ExampleView {
    title: Option<String>,
    user_input: String,
    agent_response: String,
}

/// Claude 发射器
/// 
/// 学术依据：Claude微调惯性使其对XML标签具有极高的注意力分配
pub struct ClaudeEmitter;

impl ClaudeEmitter {
    pub fn new() -> Self {
        Self
    }
}

impl Emitter for ClaudeEmitter {
    fn target(&self) -> TargetPlatform {
        TargetPlatform::Claude
    }
    
    fn emit(&self, ir: &ValidatedSkillIR) -> Result<String, EmitError> {
        let inner = ir.as_ref();
        
        // 构建模板数据
        let template = ClaudeTemplate {
            name: inner.name.to_string(),
            description: inner.description.clone(),
            version: inner.version.to_string(),
            hitl_required: inner.hitl_required,
            procedures: inner.procedures.iter().map(|p| ProcedureView {
                order: p.order,
                instruction: p.instruction.clone(),
                is_critical: p.is_critical,
            }).collect(),
            constraints: inner.anti_skill_constraints.iter().map(|c| ConstraintView {
                source: c.source.clone(),
                content: c.content.clone(),
                level: c.level.to_string(),
            }).collect(),
            examples: inner.few_shot_examples.iter().map(|e| ExampleView {
                title: e.title.clone(),
                user_input: e.user_input.clone(),
                agent_response: e.agent_response.clone(),
            }).collect(),
            input_schema: inner.input_schema.as_ref().map(|s| s.to_string()),
        };
        
        // 渲染模板
        let content = template.render()?;
        
        // 后处理：格式化 XML
        self.post_process(&content)
    }
    
    fn file_extension(&self) -> &'static str {
        ".xml"
    }
    
    fn post_process(&self, content: &str) -> Result<String, EmitError> {
        // 使用 xml-rs 进行格式化
        // 确保缩进一致，去除多余空白
        Ok(content.trim().to_string())
    }
}
```

### 3.3 Askama 模板定义

```html
<!-- nexa-skill-templates/templates/claude_xml.html -->

<agent_skill>
  <metadata>
    <name>{{ name }}</name>
    <version>{{ version }}</version>
  </metadata>
  
  <intent>{{ description }}</intent>
  
  {% if hitl_required %}
  <system_constraint>
    Wait for human explicit approval before execution.
    This skill is marked as requiring Human-In-The-Loop (HITL) confirmation.
  </system_constraint>
  {% endif %}
  
  {% if input_schema %}
  <parameters>
    <schema>{{ input_schema }}</schema>
  </parameters>
  {% endif %}
  
  <execution_steps>
    {% for step in procedures %}
    <step order="{{ step.order }}" {% if step.is_critical %}critical="true"{% endif %}>
      {{ step.instruction }}
    </step>
    {% endfor %}
  </execution_steps>
  
  {% if !constraints.is_empty() %}
  <strict_constraints>
    {% for constraint in constraints %}
    <anti_pattern source="{{ constraint.source }}" level="{{ constraint.level }}">
      {{ constraint.content }}
    </anti_pattern>
    {% endfor %}
  </strict_constraints>
  {% endif %}
  
  {% if !examples.is_empty() %}
  <examples>
    {% for example in examples %}
    <example {% if example.title %}title="{{ example.title }}"{% endif %}>
      <user_input>{{ example.user_input }}</user_input>
      <agent_response>{{ example.agent_response }}</agent_response>
    </example>
    {% endfor %}
  </examples>
  {% endif %}
</agent_skill>
```

### 3.4 产物示例

```xml
<agent_skill>
  <metadata>
    <name>database-migration</name>
    <version>2.1.0</version>
  </metadata>
  
  <intent>执行 PostgreSQL 数据库表结构修改、数据迁移或复杂 SQL DDL 操作。</intent>
  
  <system_constraint>
    Wait for human explicit approval before execution.
    This skill is marked as requiring Human-In-The-Loop (HITL) confirmation.
  </system_constraint>
  
  <parameters>
    <schema>{"type":"object","properties":{"target_table":{"type":"string"}}}</schema>
  </parameters>
  
  <execution_steps>
    <step order="1">提取目标表的当前 Schema</step>
    <step order="2" critical="true">编写 SQL 迁移脚本，必须包含 UP 和 DOWN 逻辑</step>
    <step order="3">在本地沙盒环境试运行 SQL</step>
    <step order="4" critical="true">等待用户明确批准后执行</step>
  </execution_steps>
  
  <strict_constraints>
    <anti_pattern source="db-cascade" level="block">
      Never use CASCADE without explicit user approval. Always list affected tables before executing.
    </anti_pattern>
    <anti_pattern source="sql-injection" level="error">
      Never execute raw SQL without parameter validation.
    </anti_pattern>
  </strict_constraints>
  
  <examples>
    <example title="添加列">
      <user_input>在 users 表添加 last_login_at 时间戳字段</user_input>
      <agent_response>
        1. 读取 users 表结构
        2. 生成迁移脚本
        3. 请求用户审批
        4. 执行并验证
      </agent_response>
    </example>
  </examples>
</agent_skill>
```

---

## 4. Codex Emitter（重构版）

### 4.1 适配策略（消除格式税）

> **关键发现**：2025年研究报告指出，要求大语言模型以JSON等严格结构化格式进行响应，实质上是一种"能力惩罚"（Format Tax）。GPT系列模型在处理复杂JSON结构时，数学、科学、逻辑推理等跨领域任务性能均出现大幅下滑。

**错误的原定计划**：将整个IR转化为庞大的JSON Schema，或生成JSON Schema资产文件。

**正确方案**：采用**纯Markdown输出**策略。

**核心策略**：
- **纯Markdown输出**：编译器只生成结构化Markdown文件，不生成任何JSON Schema资产
- **API层负责Schema**：JSON Schema强制约束是OpenAI Structured Outputs API的职责，不是编译器的职责
- **Markdown优先**：GPT模型训练语料中Markdown占比极高，具有天然亲和力
- **词元效率**：Markdown格式比JSON节省34%-38%的词元消耗
- **格式税消除**：避免在提示词中包含JSON结构，防止模型注意力从逻辑推理转移到语法校验

> **学术依据**：OpenAI官方指南明确建议使用Markdown的标题、列表和代码块来组织提示词。通过Structured Outputs API，GPT-4o-2024-08-06在复杂JSON模式遵循测试中达到100%完美得分。**关键点**：Schema由API层提供，而非编译器生成。

### 4.2 Askama模板驱动实现

> ⚠️ **实现说明**：CodexEmitter 使用 Askama 模板引擎驱动，与 ClaudeEmitter 采用相同的架构模式。编译器仅生成纯Markdown输出，**不生成任何JSON Schema资产文件**。Schema约束由 OpenAI Structured Outputs API 层负责。

```rust
// nexa-skill-core/src/backend/codex.rs — 实际实现

use askama::Template;
use nexa_skill_templates::{CodexContext, StepContext, ConstraintContext, ...};

use crate::analyzer::ValidatedSkillIR;
use crate::error::EmitError;
use crate::ir::SkillIR;

use super::{Emitter, TargetPlatform};

/// Codex emitter generating pure Markdown for GPT
///
/// Implements the "Decoupled Reasoning and Formatting" strategy:
/// - Output: pure structured Markdown instructions (for GPT reasoning)
/// - Schema: handled by API layer (OpenAI Structured Outputs), NOT by compiler
pub struct CodexEmitter;

impl Emitter for CodexEmitter {
    fn target(&self) -> TargetPlatform { TargetPlatform::Codex }

    fn emit(&self, ir: &ValidatedSkillIR) -> Result<String, EmitError> {
        let inner = ir.as_ref();
        let context = self.build_context(inner);
        context.render()
            .map_err(|e| EmitError::TemplateError(format!("Template render failed: {}", e)))
    }

    fn requires_manifest(&self) -> bool { true }

    /// Codex does NOT generate JSON Schema assets
    /// JSON Schema is API layer's responsibility (Structured Outputs)
    fn generate_assets(&self, _ir: &ValidatedSkillIR) -> Vec<(String, String)> {
        Vec::new()  // No schema assets — pure Markdown output only
    }
}

impl CodexEmitter {
    /// Build CodexContext from SkillIR for Askama template rendering
    fn build_context(&self, ir: &SkillIR) -> CodexContext {
        // Maps SkillIR fields to template context structs
        // (description truncation, security instruction mapping, etc.)
        CodexContext { ... }
    }
}
```

> **关键区别**：与文档早期版本描述的 `build_schema_payload()`、`supports_dual_payload()` 和 `emit_schema_payload()` 方法不同，实际实现中 **这些方法不存在**。CodexEmitter 仅通过 Askama 模板 (`codex_md.j2`) 渲染纯Markdown，`generate_assets()` 返回空列表。

### 4.3 纯Markdown产物示例

CodexEmitter 编译产物为单一Markdown文件，无JSON Schema资产：

**`database-migration.md`**（由 `codex_md.j2` 模板渲染）

```markdown
---
name: database-migration
description: 执行 PostgreSQL 数据库表结构修改、数据迁移或复杂 SQL DDL 操作。
version: 2.1.0
---

# database-migration

> ⚠️ **HITL Required**: This skill requires human approval before execution.

## Pre-Conditions

- 检查当前环境是否为非生产环境 (staging/dev)
- 确认目标表存在于数据库中

## Execution Steps

1. 提取目标表的当前 Schema
2. 编写 SQL 迁移脚本，必须包含 UP 和 DOWN 逻辑 **[CRITICAL]**
3. 在本地沙盒环境试运行 SQL
4. 等待用户明确批准后执行 **[CRITICAL]**

## Strict Constraints

> **block**: Never use CASCADE without explicit user approval. Always list affected tables before executing.

> **error**: Never execute raw SQL without parameter validation.

## Fallbacks

- 如果遇到外键约束冲突，停止执行并列出受影响的关联表
- 如果 SQL 执行超时，尝试分批处理

## Examples

### 添加列

**User**: 在 users 表添加 last_login_at 时间戳字段

**Agent**:
1. 读取 users 表结构
2. 生成迁移脚本
3. 请求用户审批
4. 执行并验证
```

> **产物目录中仅包含 `codex.md`，无 `codex_schema.json`**。JSON Schema 约束由 OpenAI Structured Outputs API 在运行时提供，编译器不负责生成。

### 4.4 词元效率对比

| 格式 | 词元消耗 | 相对效率 | 推理准确率 |
|------|----------|----------|------------|
| **Markdown（NSC采用）** | 基准 | 100% | 高 |
| JSON完整Schema | +34%~38% | 62%~66% | 低（格式税） |
| YAML | +10% | 90% | 中 |

> **结论**：Codex Emitter采用Markdown主负载，相比原JSON方案节省34%-38%词元，同时消除格式税带来的推理性能衰退。

---

## 5. Gemini Emitter（重构版）

### 5.1 适配策略（AST优化注入）

> **关键发现**：实证评测揭示，Gemini解析复杂嵌套数据的准确率：**YAML (51.9%) > Markdown (48.2%) > JSON (43.1%) > XML (33.8%)**。

**原定计划**：统一转换为标准Markdown格式。

**调整方案**：实现**编译期AST优化规则注入**。

**核心策略**：
- **Markdown主框架**：Gemini对Markdown标题和列表具有天然的"元通信"理解
- **YAML嵌套数据降维**：当Analyzer检测到深层嵌套字典数据时，自动转换为YAML格式嵌入
- **词元效率平衡**：YAML比Markdown多消耗约10%词元，但准确率提升3.7个百分点

> **学术依据**：Gemini官方文档支持使用Markdown进行"元通信"。当任务涉及高度结构化的嵌套数据时，YAML的高人类可读性和极简缩进层级结构使其能够以51.9%的最高准确率被模型解析。

### 5.2 嵌套数据检测机制

```rust
// nexa-skill-core/src/analyzer/nested_data.rs

use crate::ir::SkillIR;
use serde_json::Value;

/// 嵌套数据检测器
/// 
/// 在编译期检测IR中是否存在深层嵌套的字典数据，
/// 为Gemini Emitter提供AST优化决策依据
pub struct NestedDataDetector {
    /// 嵌套深度阈值（超过此值触发YAML转换）
    depth_threshold: usize,
}

impl NestedDataDetector {
    pub fn new() -> Self {
        Self {
            depth_threshold: 3,  // 默认3层以上视为深层嵌套
        }
    }
    
    /// 检测JSON值的嵌套深度
    pub fn detect_depth(value: &Value) -> usize {
        match value {
            Value::Object(map) => {
                let max_child_depth = map.values()
                    .map(Self::detect_depth)
                    .max()
                    .unwrap_or(0);
                1 + max_child_depth
            }
            Value::Array(arr) => {
                let max_child_depth = arr.iter()
                    .map(Self::detect_depth)
                    .max()
                    .unwrap_or(0);
                1 + max_child_depth
            }
            _ => 0,
        }
    }
    
    /// 检查IR中是否需要YAML优化
    pub fn requires_yaml_optimization(ir: &SkillIR) -> bool {
        // 检查input_schema
        if let Some(schema) = &ir.input_schema {
            if Self::detect_depth(schema) >= 3 {
                return true;
            }
        }
        
        // 检查output_schema
        if let Some(schema) = &ir.output_schema {
            if Self::detect_depth(schema) >= 3 {
                return true;
            }
        }
        
        // 检查示例中的结构化数据
        for example in &ir.few_shot_examples {
            // 简单启发式：检查是否包含JSON代码块
            if example.agent_response.contains("```json") {
                return true;
            }
        }
        
        false
    }
}
```

### 5.3 技术实现

```rust
// nexa-skill-core/src/backend/gemini.rs

use crate::backend::{Emitter, TargetPlatform};
use crate::ir::ValidatedSkillIR;
use crate::analyzer::nested_data::NestedDataDetector;
use crate::error::EmitError;
use askama::Template;
use serde_yaml;

/// Gemini Markdown 模板
#[derive(Template)]
#[template(path = "gemini_md.html")]
struct GeminiTemplate {
    name: String,
    description: String,
    version: String,
    hitl_required: bool,
    security_level: String,
    procedures: Vec<ProcedureView>,
    constraints: Vec<ConstraintView>,
    examples: Vec<ExampleView>,
    context_gathering: Vec<String>,
    pre_conditions: Vec<String>,
    post_conditions: Vec<String>,
    fallbacks: Vec<String>,
    /// 是否启用YAML优化
    use_yaml_for_schema: bool,
    /// YAML格式的schema（当use_yaml_for_schema=true时使用）
    yaml_schema: Option<String>,
}

/// Gemini 发射器
/// 
/// 实现"AST优化注入"策略：
/// - 主框架：Markdown元通信协议
/// - 嵌套数据：自动转换为YAML格式
/// 
/// 学术依据：YAML嵌套数据准确率51.9% > Markdown 48.2% > JSON 43.1%
pub struct GeminiEmitter;

impl GeminiEmitter {
    pub fn new() -> Self {
        Self
    }
    
    /// 将JSON Schema转换为YAML格式
    fn json_to_yaml(json: &serde_json::Value) -> Result<String, EmitError> {
        let yaml = serde_yaml::to_string(json)
            .map_err(|e| EmitError::SerializationError { reason: e.to_string() })?;
        Ok(yaml)
    }
}

impl Emitter for GeminiEmitter {
    fn target(&self) -> TargetPlatform {
        TargetPlatform::Gemini
    }
    
    fn emit(&self, ir: &ValidatedSkillIR) -> Result<String, EmitError> {
        let inner = ir.as_ref();
        
        // 检测是否需要YAML优化
        let use_yaml = NestedDataDetector::requires_yaml_optimization(inner);
        
        // 如果需要YAML优化，转换schema
        let yaml_schema = if use_yaml && inner.input_schema.is_some() {
            Some(Self::json_to_yaml(&inner.input_schema.clone().unwrap())?)
        } else {
            None
        };
        
        let template = GeminiTemplate {
            name: inner.name.to_string(),
            description: inner.description.clone(),
            version: inner.version.to_string(),
            hitl_required: inner.hitl_required,
            security_level: inner.security_level.to_string(),
            procedures: inner.procedures.iter().map(|p| ProcedureView {
                order: p.order,
                instruction: p.instruction.clone(),
                is_critical: p.is_critical,
            }).collect(),
            constraints: inner.anti_skill_constraints.iter().map(|c| ConstraintView {
                source: c.source.clone(),
                content: c.content.clone(),
                level: c.level.to_string(),
            }).collect(),
            examples: inner.few_shot_examples.iter().map(|e| ExampleView {
                title: e.title.clone(),
                user_input: e.user_input.clone(),
                agent_response: e.agent_response.clone(),
            }).collect(),
            context_gathering: inner.context_gathering.clone(),
            pre_conditions: inner.pre_conditions.clone(),
            post_conditions: inner.post_conditions.clone(),
            fallbacks: inner.fallbacks.clone(),
            use_yaml_for_schema: use_yaml,
            yaml_schema,
        };
        
        let content = template.render()?;
        
        Ok(content)
    }
    
    fn file_extension(&self) -> &'static str {
        ".md"
    }
}
```

### 5.4 Askama 模板定义（支持YAML优化）

```html
<!-- nexa-skill-templates/templates/gemini_md.html -->

# {{ name }}

> Version: {{ version }} | Security Level: {{ security_level }}

## Description

{{ description }}

{% if hitl_required %}
> ⚠️ **HITL Required**: This skill requires human approval before execution.
{% endif %}

{% if !pre_conditions.is_empty() %}
## Pre-Conditions

{% for condition in pre_conditions %}
- {{ condition }}
{% endfor %}
{% endif %}

{% if !context_gathering.is_empty() %}
## Context Gathering

{% for item in context_gathering %}
- {{ item }}
{% endfor %}
{% endif %}

## Execution Steps

{% for step in procedures %}
**Step {{ step.order }}**{% if step.is_critical %} [CRITICAL]{% endif %}

{{ step.instruction }}

{% endfor %}

{% if !constraints.is_empty() %}
## Strict Constraints

{% for constraint in constraints %}
> **{{ constraint.level }}**: {{ constraint.content }}
{% endfor %}
{% endif %}

{% if use_yaml_for_schema %}
## Parameter Schema (YAML Optimized)

> **AST Optimization**: Nested data converted to YAML for 51.9% parsing accuracy.

```yaml
{{ yaml_schema }}
```

{% else if input_schema %}
## Parameter Schema

```json
{{ input_schema }}
```
{% endif %}

{% if !fallbacks.is_empty() %}
## Fallbacks

{% for fallback in fallbacks %}
- {{ fallback }}
{% endfor %}
{% endif %}

{% if !post_conditions.is_empty() %}
## Post-Conditions

{% for condition in post_conditions %}
- {{ condition }}
{% endfor %}
{% endif %}

{% if !examples.is_empty() %}
## Examples

{% for example in examples %}
### {% if example.title %}{{ example.title }}{% else %}Example {{ loop.index }}{% endif %}

**User**: {{ example.user_input }}

**Agent**:
{{ example.agent_response }}

{% endfor %}
{% endif %}
```

### 5.5 产物示例（YAML优化版）

```markdown
# api-response-parser

> Version: 1.5.0 | Security Level: medium

## Description

解析复杂的API响应结构，提取嵌套字段并生成结构化报告。

## Execution Steps

**Step 1**

接收API响应JSON数据

**Step 2** [CRITICAL]

解析嵌套结构，提取目标字段

**Step 3**

生成结构化输出报告

## Parameter Schema (YAML Optimized)

> **AST Optimization**: Nested data converted to YAML for 51.9% parsing accuracy.

```yaml
type: object
properties:
  api_endpoint:
    type: string
    description: API请求地址
  response_structure:
    type: object
    properties:
      data:
        type: object
        properties:
          items:
            type: array
            items:
              type: object
              properties:
                id:
                  type: string
                attributes:
                  type: object
                  properties:
                    name:
                      type: string
                    metadata:
                      type: object
                      properties:
                        created_at:
                          type: string
                          format: date-time
                        tags:
                          type: array
                          items:
                            type: string
required:
  - api_endpoint
```

## Examples

### 解析GitHub API响应

**User**: 解析GitHub仓库列表API响应

**Agent**:
1. 识别响应结构
2. 提取仓库ID和名称
3. 解析metadata中的创建时间和标签
4. 生成CSV格式报告
```

> **学术价值**：此编译期优化特性展示了NSC如何根据模型特性自动调整输出格式，实现"格式自适应编译"。

---

## 6. Kimi Emitter

### 6.1 适配策略

Kimi 延续了超长上下文领域的统治力，偏好海量上下文而非结构化约束。

**核心策略**：
- **纯文本优先**：生成无结构的纯文本或完整Markdown
- **知识密集型**：保留所有细节，不进行精简
- **弱约束，强推理**：依赖 Agent 从上下文中推理

### 6.2 技术实现

```rust
// nexa-skill-core/src/backend/kimi.rs

use crate::backend::{Emitter, TargetPlatform};
use crate::ir::ValidatedSkillIR;
use crate::error::EmitError;

/// Kimi 发射器
pub struct KimiEmitter;

impl KimiEmitter {
    pub fn new() -> Self {
        Self
    }
    
    /// 生成完整 Markdown（保留所有细节）
    fn generate_full_markdown(ir: &ValidatedSkillIR) -> String {
        let inner = ir.as_ref();
        let mut output = String::new();
        
        // 标题
        output.push_str(&format!("# {}\n\n", inner.name));
        
        // 元信息
        output.push_str(&format!("**Version**: {}\n", inner.version));
        output.push_str(&format!("**Security Level**: {}\n\n", inner.security_level));
        
        // 描述
        output.push_str("## Description\n\n");
        output.push_str(&inner.description);
        output.push_str("\n\n");
        
        // HITL 提示
        if inner.hitl_required {
            output.push_str("> **注意**: 此技能需要人工审批后才能执行。\n\n");
        }
        
        // MCP 依赖
        if !inner.mcp_servers.is_empty() {
            output.push_str("## MCP Dependencies\n\n");
            for server in &inner.mcp_servers {
                output.push_str(&format!("- {}\n", server));
            }
            output.push_str("\n");
        }
        
        // 权限声明
        if !inner.permissions.is_empty() {
            output.push_str("## Permissions\n\n");
            for perm in &inner.permissions {
                output.push_str(&format!("- {}: {}\n", perm.kind.display_name(), perm.scope));
            }
            output.push_str("\n");
        }
        
        // 前置条件
        if !inner.pre_conditions.is_empty() {
            output.push_str("## Pre-Conditions\n\n");
            for cond in &inner.pre_conditions {
                output.push_str(&format!("- {}\n", cond));
            }
            output.push_str("\n");
        }
        
        // 上下文收集
        if !inner.context_gathering.is_empty() {
            output.push_str("## Context Gathering\n\n");
            for item in &inner.context_gathering {
                output.push_str(&format!("- {}\n", item));
            }
            output.push_str("\n");
        }
        
        // 执行步骤
        output.push_str("## Procedures\n\n");
        for step in &inner.procedures {
            let critical_marker = if step.is_critical { " [CRITICAL]" } else { "" };
            output.push_str(&format!("{}. {}{}\n", step.order, step.instruction, critical_marker));
        }
        output.push_str("\n");
        
        // Anti-Skill 约束
        if !inner.anti_skill_constraints.is_empty() {
            output.push_str("## Safety Constraints\n\n");
            for constraint in &inner.anti_skill_constraints {
                output.push_str(&format!("> {}\n", constraint.content));
            }
            output.push_str("\n");
        }
        
        // 错误恢复
        if !inner.fallbacks.is_empty() {
            output.push_str("## Fallbacks\n\n");
            for fb in &inner.fallbacks {
                output.push_str(&format!("- {}\n", fb));
            }
            output.push_str("\n");
        }
        
        // 后置条件
        if !inner.post_conditions.is_empty() {
            output.push_str("## Post-Conditions\n\n");
            for cond in &inner.post_conditions {
                output.push_str(&format!("- {}\n", cond));
            }
            output.push_str("\n");
        }
        
        // 示例
        if !inner.few_shot_examples.is_empty() {
            output.push_str("## Examples\n\n");
            for example in &inner.few_shot_examples {
                if let Some(title) = &example.title {
                    output.push_str(&format!("### {}\n\n", title));
                }
                output.push_str(&format!("**User**: {}\n\n", example.user_input));
                output.push_str(&format!("**Agent**:\n{}\n\n", example.agent_response));
            }
        }
        
        output
    }
}

impl Emitter for KimiEmitter {
    fn target(&self) -> TargetPlatform {
        TargetPlatform::Kimi
    }
    
    fn emit(&self, ir: &ValidatedSkillIR) -> Result<String, EmitError> {
        let content = Self::generate_full_markdown(ir);
        Ok(content)
    }
    
    fn file_extension(&self) -> &'static str {
        ".md"
    }
}
```

---

## 7. 并行发射机制

### 7.1 Rayon 并行实现

```rust
// nexa-skill-core/src/backend/parallel.rs

use rayon::prelude::*;
use crate::backend::{Emitter, TargetPlatform, EmitterRegistry};
use crate::ir::ValidatedSkillIR;
use crate::error::EmitError;

/// 并行发射器
pub struct ParallelEmitter {
    registry: EmitterRegistry,
}

impl ParallelEmitter {
    pub fn new(registry: EmitterRegistry) -> Self {
        Self { registry }
    }
    
    /// 并行发射多个目标
    pub fn emit_all(
        &self,
        ir: &ValidatedSkillIR,
        targets: &[TargetPlatform],
    ) -> Result<Vec<(TargetPlatform, String, Option<String>)>, EmitError> {
        targets
            .par_iter()
            .map(|target| {
                let emitter = self.registry.get(target)?;
                let main_content = emitter.emit(ir)?;
                
                let assets = emitter.generate_assets(ir);
                
                Ok((*target, main_content, assets))
            })
            .collect()
    }
}
```

### 7.2 性能优化

| 场景 | 串行耗时 | 并行耗时 | 提升 |
|------|----------|----------|------|
| 单目标 | ~50ms | ~50ms | 无变化 |
| 3 目标 | ~150ms | ~60ms | 2.5x |
| 5 目标 | ~250ms | ~70ms | 3.5x |

---

## 8. 产物目录结构

### 8.1 标准产物目录

```text
build/{skill-name}/
├── manifest.json            # 编译元数据
├── routing_manifest.yaml    # 渐进式路由清单（新增）
├── target/                  # 平台特定产物
│   ├── claude.xml           # Claude XML
│   ├── codex.md             # Codex Markdown（纯Markdown输出，无JSON Schema资产）
│   ├── gemini.md            # Gemini Markdown（可能含YAML块）
│   └── kimi.md              # Kimi Markdown
├── assets/                  # 静态资源（从源目录拷贝）
│   ├── templates/
│   └── scripts/
└── meta/
    ├── signature.sha256     # 完整性哈希
    ├── source_hash.txt      # 源文件哈希
    └── compile_log.json     # 编译日志
```

### 8.2 Manifest 生成

```rust
// nexa-skill-core/src/backend/manifest.rs

use crate::ir::{ValidatedSkillIR, Manifest, TargetInfo};
use std::path::Path;
use sha2::{Sha256, Digest};

/// 生成 Manifest
pub fn generate_manifest(
    ir: &ValidatedSkillIR,
    output_dir: &Path,
    targets: &[TargetPlatform],
) -> Result<Manifest, EmitError> {
    let mut target_infos = Vec::new();
    
    for target in targets {
        let file_name = format!("{}{}", target.slug(), target.extension());
        let file_path = output_dir.join("target").join(&file_name);
        
        if file_path.exists() {
            let content = std::fs::read(&file_path)?;
            let hash = format!("{:x}", Sha256::digest(&content));
            
            target_infos.push(TargetInfo {
                platform: target.display_name(),
                output_file: file_name,
                file_size: content.len(),
                file_hash: hash,
            });
        }
        
    }
    
    Ok(Manifest::from_ir(ir, &target_infos))
}
```

---

## 9. 自定义 Emitter 扩展

### 9.1 扩展步骤

1. 实现 `Emitter` Trait
2. 创建 Askama 模板（可选）
3. 注册到 `EmitterRegistry`

### 9.2 示例：KimiEmitter 扩展参考

> **注意**：KimiEmitter 已作为核心平台内置实现（见 `nexa-skill-core/src/backend/kimi.rs`）。以下仅作扩展模式参考。

```rust
// KimiEmitter 已内置，此处仅展示扩展模式

use crate::backend::{Emitter, TargetPlatform};
use crate::ir::ValidatedSkillIR;
use crate::error::EmitError;

pub struct KimiEmitter;

impl Emitter for KimiEmitter {
    fn target(&self) -> TargetPlatform {
        TargetPlatform::Kimi
    }
    
    fn emit(&self, ir: &ValidatedSkillIR) -> Result<String, EmitError> {
        // Kimi 偏好纯文本/完整Markdown，弱约束强推理
        let inner = ir.as_ref();
        
        let mut output = format!(
            "---\nname: {}\ndescription: {}\n---\n\n",
            inner.name, inner.description
        );
        
        output.push_str("# Instructions\n\n");
        for step in &inner.procedures {
            output.push_str(&format!("{}. {}\n", step.order, step.instruction));
        }
        
        Ok(output)
    }
    
    fn file_extension(&self) -> &'static str {
        ".md"
    }
}

// 注册
let mut registry = EmitterRegistry::default();
registry.register(Box::new(KimiEmitter));
```

---

## 10. 学术依据总结

### 10.1 格式偏好实证数据

| 模型 | 最佳输入格式 | 准确率提升 | 词元效率 | 学术来源 |
|------|--------------|------------|----------|----------|
| Claude | XML标签 | +23% (vs JSON) | 中 | Anthropic官方指南 |
| GPT | Markdown | +40% (vs JSON) | **最优** | Format Tax研究 |
| Gemini | Markdown + YAML嵌套 | YAML 51.9% > JSON 43.1% | 高 | 嵌套数据压力测试 |

### 10.2 关键设计决策

| 决策 | 原因 | 效果 |
|------|------|------|
| Codex纯Markdown输出 | 消除格式税 | 词元效率优化，Schema由API层负责 |
| Gemini YAML优化 | 嵌套数据准确率 | +8.8%解析准确率 |
| 渐进式路由清单 | 解决上下文膨胀 | 常驻词元极小化 |

---

## 11. 相关文档

- [COMPILER_PIPELINE.md](COMPILER_PIPELINE.md) - Backend 阶段在管线中的位置
- [IR_DESIGN.md](IR_DESIGN.md) - ValidatedSkillIR 数据结构
- [ROUTING_MANIFEST.md](ROUTING_MANIFEST.md) - 渐进式路由清单机制
- [API_REFERENCE.md](API_REFERENCE.md) - Emitter Trait API 定义
- [CLI_DESIGN.md](CLI_DESIGN.md) - Target Flag 命令行设计