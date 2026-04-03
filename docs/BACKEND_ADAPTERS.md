# 后端适配器设计

> **Emitter Trait 定义、各平台适配策略与产物生成细节**

---

## 1. 后端设计概述

Backend 阶段负责将 `ValidatedSkillIR` 序列化为特定平台的原生表示。NSC 采用**多态发射器 (Polymorphic Emitter)** 架构，通过统一的 `Emitter` Trait 支持不同目标平台。

### 1.1 设计原则

| 原则 | 描述 |
|------|------|
| **平台原生** | 每个平台生成最适合其底层模型的格式 |
| **编译期检查** | 使用 Askama 模板引擎，模板错误在编译期发现 |
| **可扩展** | 新平台只需实现 `Emitter` Trait |
| **并行发射** | 多目标编译时使用 Rayon 并行生成 |

### 1.2 平台适配策略总览

| 平台 | 底层模型 | 首选格式 | 核心策略 |
|------|----------|----------|----------|
| **Claude** | Claude 4.6 Opus | XML | XML 原教旨主义，强标签嵌套 |
| **Codex** | GPT-5.4 | JSON Schema | Function Calling 接口化 |
| **Gemini** | Gemini 3.1 Pro | Markdown | 结构化 SOP，多模态钩子 |
| **Kimi** | K2.5 | 纯文本 | 海量上下文，文档吞噬 |

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
}

impl TargetPlatform {
    /// 获取平台标识符（用于 CLI flag）
    pub fn slug(&self) -> &'static str {
        match self {
            TargetPlatform::Claude => "claude",
            TargetPlatform::Codex => "codex",
            TargetPlatform::Gemini => "gemini",
            TargetPlatform::Kimi => "kimi",
            TargetPlatform::Copilot => "copilot",
            TargetPlatform::VSCode => "vscode",
        }
    }
    
    /// 获取产物文件扩展名
    pub fn extension(&self) -> &'static str {
        match self {
            TargetPlatform::Claude => ".xml",
            TargetPlatform::Codex => "_schema.json",
            TargetPlatform::Gemini => ".md",
            TargetPlatform::Kimi => ".md",
            TargetPlatform::Copilot => ".json",
            TargetPlatform::VSCode => ".md",
        }
    }
    
    /// 获取平台显示名称
    pub fn display_name(&self) -> &'static str {
        match self {
            TargetPlatform::Claude => "Claude Code",
            TargetPlatform::Codex => "OpenAI Codex",
            TargetPlatform::Gemini => "Gemini CLI",
            TargetPlatform::Kimi => "Kimi CLI",
            TargetPlatform::Copilot => "GitHub Copilot",
            TargetPlatform::VSCode => "VS Code Agent",
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

### 3.1 适配策略

Claude 模型在对齐训练时深度绑定了 XML 标签，对嵌套清晰的 XML 结构有极高的指令遵循度。

**核心策略**：
- **XML 原教旨主义**：使用严格嵌套的 XML 树结构
- **边界清晰**：使用 `<system>`, `<skill_definition>`, `<execution_steps>` 等标签划分信息块
- **防御性强**：将约束包装在 `<strict_constraints>` 中，防止提示词注入

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

## 4. Codex Emitter

### 4.1 适配策略

OpenAI 的演进路线围绕 Function Calling 和严格的 JSON 输出。Codex Agent 更像传统软件工程师，偏好"面向对象"的技能定义。

**核心策略**：
- **JSON Schema 优先**：将 SkillIR 转化为 OpenAI Function Calling Payload
- **接口化**：关注 `input_schema` 和 `returns`，而非华丽辞藻
- **描述扩写**：将 Procedures 整合到 `description` 字段中

### 4.2 技术实现

```rust
// nexa-skill-core/src/backend/codex.rs

use crate::backend::{Emitter, TargetPlatform};
use crate::ir::ValidatedSkillIR;
use crate::error::EmitError;
use serde_json::{json, Value};

/// Codex 发射器
pub struct CodexEmitter;

impl CodexEmitter {
    pub fn new() -> Self {
        Self
    }
    
    /// 构建 OpenAI Function Calling Schema
    fn build_function_schema(ir: &ValidatedSkillIR) -> Value {
        let inner = ir.as_ref();
        
        // 扩写 description，包含 Procedures
        let expanded_description = Self::expand_description(inner);
        
        json!({
            "name": inner.name,
            "description": expanded_description,
            "parameters": inner.input_schema.clone().unwrap_or_else(|| json!({
                "type": "object",
                "properties": {},
                "required": []
            })),
            "returns": inner.output_schema.clone(),
            "metadata": {
                "version": inner.version,
                "hitl_required": inner.hitl_required,
                "security_level": inner.security_level.to_string(),
                "mcp_servers": inner.mcp_servers.iter().map(|s| s.to_string()).collect::<Vec<_>>(),
            },
            "constraints": Self::build_constraints(inner),
        })
    }
    
    /// 扩写 description
    fn expand_description(ir: &SkillIR) -> String {
        let mut desc = ir.description.clone();
        
        // 添加 Procedures 概要
        if !ir.procedures.is_empty() {
            desc.push_str("\n\nExecution Steps:\n");
            for step in &ir.procedures {
                desc.push_str(&format!("{}. {}\n", step.order, step.instruction));
            }
        }
        
        // 添加关键约束
        if !ir.anti_skill_constraints.is_empty() {
            desc.push_str("\n\nStrict Constraints:\n");
            for constraint in &ir.anti_skill_constraints {
                desc.push_str(&format!("- {}\n", constraint.content));
            }
        }
        
        desc
    }
    
    /// 构建约束列表
    fn build_constraints(ir: &SkillIR) -> Value {
        ir.anti_skill_constraints.iter().map(|c| json!({
            "source": c.source,
            "content": c.content,
            "level": c.level.to_string(),
        })).collect()
    }
}

impl Emitter for CodexEmitter {
    fn target(&self) -> TargetPlatform {
        TargetPlatform::Codex
    }
    
    fn emit(&self, ir: &ValidatedSkillIR) -> Result<String, EmitError> {
        let schema = Self::build_function_schema(ir);
        
        // 序列化为 JSON
        let content = serde_json::to_string_pretty(&schema)?;
        
        Ok(content)
    }
    
    fn file_extension(&self) -> &'static str {
        "_schema.json"
    }
}
```

### 4.3 产物示例

```json
{
  "name": "database-migration",
  "description": "执行 PostgreSQL 数据库表结构修改、数据迁移或复杂 SQL DDL 操作。\n\nExecution Steps:\n1. 提取目标表的当前 Schema\n2. 编写 SQL 迁移脚本，必须包含 UP 和 DOWN 逻辑\n3. 在本地沙盒环境试运行 SQL\n4. 等待用户明确批准后执行\n\nStrict Constraints:\n- Never use CASCADE without explicit user approval\n- Never execute raw SQL without parameter validation",
  "parameters": {
    "type": "object",
    "properties": {
      "target_table": {
        "type": "string",
        "description": "目标表名"
      },
      "migration_type": {
        "type": "string",
        "enum": ["add_column", "drop_column", "alter_type"]
      }
    },
    "required": ["target_table", "migration_type"]
  },
  "returns": {
    "type": "object",
    "properties": {
      "success": { "type": "boolean" },
      "migration_file": { "type": "string" }
    }
  },
  "metadata": {
    "version": "2.1.0",
    "hitl_required": true,
    "security_level": "high",
    "mcp_servers": ["neon-postgres-admin", "github-pr-creator"]
  },
  "constraints": [
    {
      "source": "db-cascade",
      "content": "Never use CASCADE without explicit user approval",
      "level": "block"
    }
  ]
}
```

---

## 5. Gemini Emitter

### 5.1 适配策略

Gemini 对原生 Markdown 列表的解析极佳，支持多模态钩子，偏好结构化的 SOP 流程。

**核心策略**：
- **结构化 Markdown**：重新组装为排版严密的"机器级 Markdown"
- **SOP 流程化**：使用带编号的执行步骤
- **多模态钩子**：保留视觉/上下文感知描述

### 5.2 技术实现

```rust
// nexa-skill-core/src/backend/gemini.rs

use crate::backend::{Emitter, TargetPlatform};
use crate::ir::ValidatedSkillIR;
use crate::error::EmitError;
use askama::Template;

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
}

/// Gemini 发射器
pub struct GeminiEmitter;

impl GeminiEmitter {
    pub fn new() -> Self {
        Self
    }
}

impl Emitter for GeminiEmitter {
    fn target(&self) -> TargetPlatform {
        TargetPlatform::Gemini
    }
    
    fn emit(&self, ir: &ValidatedSkillIR) -> Result<String, EmitError> {
        let inner = ir.as_ref();
        
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
        };
        
        let content = template.render()?;
        
        Ok(content)
    }
    
    fn file_extension(&self) -> &'static str {
        ".md"
    }
}
```

### 5.3 Askama 模板定义

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

### 5.4 产物示例

```markdown
# database-migration

> Version: 2.1.0 | Security Level: high

## Description

执行 PostgreSQL 数据库表结构修改、数据迁移或复杂 SQL DDL 操作。

> ⚠️ **HITL Required**: This skill requires human approval before execution.

## Pre-Conditions

- 检查当前环境是否为非生产环境 (staging/dev)
- 确认目标表存在于数据库中

## Context Gathering

- 使用 neon-postgres-admin 提取目标表的当前 Schema
- 检索代码库中与该表相关的 ORM 模型文件

## Execution Steps

**Step 1**

提取目标表的当前 Schema

**Step 2** [CRITICAL]

编写 SQL 迁移脚本，必须包含 UP 和 DOWN 逻辑

**Step 3**

在本地沙盒环境试运行 SQL

**Step 4** [CRITICAL]

等待用户明确批准后执行

## Strict Constraints

> **block**: Never use CASCADE without explicit user approval. Always list affected tables before executing.

> **error**: Never execute raw SQL without parameter validation.

## Fallbacks

- 如果遇到外键约束冲突，停止执行并列出受影响的关联表
- 如果 SQL 执行超时，尝试分批处理

## Post-Conditions

- 执行 ORM 模型同步脚本
- 运行数据库完整性检查

## Examples

### 添加列

**User**: 在 users 表添加 last_login_at 时间戳字段

**Agent**:
1. 读取 users 表结构
2. 生成迁移脚本
3. 请求用户审批
4. 执行并验证
```

---

## 6. Kimi Emitter

### 6.1 适配策略

Kimi 延续了超长上下文领域的统治力，偏好海量上下文而非结构化约束。

**核心策略**：
- **纯文本优先**：生成无结构的纯文本或完整 Markdown
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
    ) -> Result<Vec<(TargetPlatform, String)>, EmitError> {
        targets
            .par_iter()
            .map(|target| {
                let emitter = self.registry.get(target)?;
                let content = emitter.emit(ir)?;
                Ok((*target, content))
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
├── target/                  # 平台特定产物
│   ├── claude.xml           # Claude XML
│   ├── codex_schema.json    # Codex JSON Schema
│   ├── gemini.md            # Gemini Markdown
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

### 9.2 示例：自定义 VS Code Emitter

```rust
// 自定义 VS Code Agent Emitter

use crate::backend::{Emitter, TargetPlatform};
use crate::ir::ValidatedSkillIR;
use crate::error::EmitError;

pub struct VSCodeEmitter;

impl Emitter for VSCodeEmitter {
    fn target(&self) -> TargetPlatform {
        TargetPlatform::VSCode
    }
    
    fn emit(&self, ir: &ValidatedSkillIR) -> Result<String, EmitError> {
        // VS Code 使用标准 Markdown 格式
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
registry.register(Box::new(VSCodeEmitter));
```

---

## 10. 相关文档

- [COMPILER_PIPELINE.md](COMPILER_PIPELINE.md) - Backend 阶段在管线中的位置
- [IR_DESIGN.md](IR_DESIGN.md) - ValidatedSkillIR 数据结构
- [API_REFERENCE.md](API_REFERENCE.md) - Emitter Trait API 定义
- [CLI_DESIGN.md](CLI_DESIGN.md) - Target Flag 命令行设计