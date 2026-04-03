# SKILL.md 规范定义

> **Nexa Skill Compiler 源文件的完整语法规范与元数据定义**

---

## 1. 规范概述

`SKILL.md` 是 NSC 编译器的源文件格式，它基于标准 Markdown 语法，通过 YAML Frontmatter 定义元数据，通过 Markdown Body 定义执行逻辑。本规范遵循 [Agent Skills 官方规范](https://agentskills.io/) 并进行了扩展以支持 NSC 的编译特性。

### 1.1 设计原则

| 原则 | 描述 |
|------|------|
| **人类优先** | 文件首先服务于人类阅读和编写，其次服务于机器解析 |
| **渐进式披露** | 元数据精简，详细内容按需加载 |
| **强类型约束** | 关键字段有严格的格式校验，编译期报错 |
| **平台无关** | 源文件不包含任何平台特定语法，由编译器适配 |

### 1.2 文件位置约定

```text
skill-name/
├── SKILL.md          # 必选：元数据 + 指令
├── scripts/          # 可选：可执行脚本
├── references/       # 可选：参考文档
├── assets/           # 可选：静态资源
└── templates/        # 可选：模板文件
```

---

## 2. 文件结构

一个完整的 `SKILL.md` 文件由两部分组成：

```markdown
---
# YAML Frontmatter 区域
name: skill-name
description: 技能描述
---

# Markdown Body 区域
## Description
详细描述...

## Procedures
1. 步骤一
2. 步骤二
```

### 2.1 Frontmatter 与 Body 的分隔

- Frontmatter 必须以 `---` 开始和结束
- Body 紧接在 Frontmatter 结束标记之后
- Frontmatter 与 Body 之间允许有空行

---

## 3. YAML Frontmatter 规范

### 3.1 必选字段

| 字段 | 类型 | 约束 | 描述 |
|------|------|------|------|
| `name` | `string` | 1-64字符，kebab-case | 技能唯一标识符 |
| `description` | `string` | 1-1024字符 | 功能描述与触发条件 |

#### 3.1.1 `name` 字段规范

**格式约束**：
- 仅允许小写字母 (`a-z`)、数字 (`0-9`) 和连字符 (`-`)
- 不能以连字符开头或结尾
- 不能包含连续连字符 (`--`)
- 必须与父目录名称匹配

**有效示例**：
```yaml
name: database-migration
name: web-scraper
name: pdf-processing
name: code-review-v2
```

**无效示例**：
```yaml
name: Database-Migration  # 包含大写字母
name: -database           # 以连字符开头
name: database--migration # 包含连续连字符
name: database_migration  # 包含下划线
name: 数据库迁移           # 包含非 ASCII 字符
```

#### 3.1.2 `description` 字段规范

**内容要求**：
- 必须描述"能做什么"和"何时触发"
- 应包含有助于 Agent 路由的关键词
- 建议明确"不该何时触发"的边界条件

**良好示例**：
```yaml
description: >-
  执行数据库表结构修改、数据迁移或复杂 SQL DDL 操作。
  当用户要求修改数据库架构、添加/删除列、创建索引时触发。
  不要在仅需要查询数据（SELECT）时触发此技能。
```

**不良示例**：
```yaml
description: 帮助处理数据库。  # 过于模糊，缺乏触发条件
```

### 3.2 可选字段

| 字段 | 类型 | 约束 | 描述 |
|------|------|------|------|
| `version` | `string` | 语义化版本 | 技能版本号 |
| `license` | `string` | 许可证名称或文件引用 | 许可证声明 |
| `compatibility` | `string` | 1-500字符 | 环境兼容性说明 |
| `metadata` | `object` | 键值对映射 | 扩展元数据 |
| `allowed-tools` | `string` | 空格分隔的工具列表 | 预批准工具（实验性） |

#### 3.2.1 `version` 字段

采用语义化版本格式 (`MAJOR.MINOR.PATCH`)：

```yaml
version: "1.0.0"
version: "2.1.3"
```

#### 3.2.2 `compatibility` 字段

声明技能运行所需的环境条件：

```yaml
compatibility: 需要 Python 3.10+ 和 PostgreSQL 客户端
compatibility: 需要 git, docker, jq 和网络访问
compatibility: 专为 Claude Code 设计
```

#### 3.2.3 `metadata` 字段

用于存储扩展属性，键名建议具有唯一性以避免冲突：

```yaml
metadata:
  author: nexa-dev-team
  created_at: "2026-04-01"
  category: database
  tags: [migration, sql, ddl]
```

#### 3.2.4 `allowed-tools` 字段（实验性）

声明预批准的工具，减少运行时确认：

```yaml
allowed-tools: Bash(git:*) Bash(jq:*) Read Write
```

### 3.3 NSC 扩展字段

以下字段是 NSC 编译器的扩展，用于支持高级编译特性：

| 字段 | 类型 | 描述 |
|------|------|------|
| `mcp_servers` | `array<string>` | MCP 服务器依赖声明 |
| `input_schema` | `object` | 输入参数 JSON Schema |
| `output_schema` | `object` | 输出参数 JSON Schema |
| `hitl_required` | `boolean` | 是否需要人工审批 |
| `pre_conditions` | `array<string>` | 执行前必须满足的条件 |
| `post_conditions` | `array<string>` | 执行后必须验证的条件 |
| `fallbacks` | `array<string>` | 错误恢复策略 |
| `permissions` | `array<object>` | 权限声明列表 |
| `security_level` | `string` | 安全等级 (low/medium/high/critical) |

#### 3.3.1 `mcp_servers` 字段

声明技能运行所需的 MCP 服务器：

```yaml
mcp_servers:
  - neon-postgres-admin
  - github-pr-creator
  - filesystem-server
```

#### 3.3.2 `input_schema` 字段

基于 JSON Schema 定义输入参数：

```yaml
input_schema:
  type: object
  properties:
    target_table:
      type: string
      description: 目标表名
    migration_type:
      type: string
      enum: [add_column, drop_column, alter_type, rename]
    columns:
      type: array
      items:
        type: object
        properties:
          name: { type: string }
          data_type: { type: string }
  required: [target_table, migration_type]
```

#### 3.3.3 `hitl_required` 字段

对于涉及敏感操作的技能，强制要求人工确认：

```yaml
hitl_required: true  # 执行前必须等待人类审批
```

#### 3.3.4 `pre_conditions` / `post_conditions` 字段

定义执行前后必须满足的断言：

```yaml
pre_conditions:
  - 检查当前环境是否为非生产环境 (staging/dev)
  - 确认目标表存在于数据库中
  - 验证用户具有 ALTER TABLE 权限

post_conditions:
  - 执行 ORM 模型同步脚本
  - 运行数据库完整性检查
  - 更新 schema 版本记录
```

#### 3.3.5 `fallbacks` 字段

定义错误恢复策略：

```yaml
fallbacks:
  - 如果遇到外键约束冲突，停止执行并列出受影响的关联表
  - 如果 SQL 执行超时，尝试分批处理
  - 如果回滚失败，记录错误并通知管理员
```

#### 3.3.6 `permissions` 字段

声明技能所需的权限：

```yaml
permissions:
  - kind: network
    scope: "https://api.example.com/*"
  - kind: fs
    scope: "/tmp/migration-*"
  - kind: db
    scope: "postgres:staging:ALTER"
```

**权限类型枚举**：

| Kind | Scope 格式 | 示例 |
|------|-----------|------|
| `network` | URL pattern | `https://api.github.com/*` |
| `fs` | 文件路径 pattern | `/tmp/skill-*` |
| `db` | `db_type:db_name:operation` | `postgres:staging:SELECT` |
| `exec` | 命令 pattern | `git:*` |
| `mcp` | MCP 服务器名称 | `filesystem-server` |

#### 3.3.7 `security_level` 字段

声明技能的安全等级，影响编译期的审计强度：

```yaml
security_level: high  # low | medium | high | critical
```

| 等级 | 审计行为 |
|------|----------|
| `low` | 仅基础格式校验 |
| `medium` | 权限声明检查 |
| `high` | 强制 HITL，高危词汇扫描 |
| `critical` | 禁止自动执行，必须人工审批 |

---

## 4. Markdown Body 规范

### 4.1 推荐章节结构

Body 内容应遵循以下章节结构（顺序建议）：

```markdown
# [技能名称]

简短的功能概述（可选，与 description 互补）

## Description

详细的功能描述、适用场景、边界条件

## Triggers / When to Use

触发条件的详细说明，包含关键词和场景示例

## Context Gathering

执行前需要收集的上下文信息

## Procedures / Execution Steps

标准作业程序（SOP），带编号的执行步骤

## Examples / Few-Shot Examples

输入输出示例，帮助 Agent 理解预期行为

## Edge Cases / Common Pitfalls

常见边界情况和处理方式

## Fallbacks / Error Handling

错误恢复策略（如果未在 Frontmatter 中定义）

## References

相关文档链接（可选）
```

### 4.2 Procedures 章节规范

Procedures 是 Body 的核心，NSC 会将其解析为 `ProcedureStep` 结构。

**格式要求**：
- 使用有序列表（`1.`, `2.`, `3.`）
- 每个步骤应简洁明确
- 关键步骤可标记为 `[CRITICAL]`

**示例**：

```markdown
## Procedures

1. 验证 URL 格式是否合法。
2. [CRITICAL] 发送 HTTP GET 请求，设置超时为 10 秒。
3. 使用 BeautifulSoup 解析 HTML。
4. 定位 `<table>` 标签并提取数据。
5. 将数据转换为 JSON 格式输出。
6. 如果遇到 JavaScript 渲染的内容，切换到 Selenium 方案。
```

**解析结果**：

```rust
ProcedureStep {
    order: 1,
    instruction: "验证 URL 格式是否合法。",
    is_critical: false,
}
ProcedureStep {
    order: 2,
    instruction: "发送 HTTP GET 请求，设置超时为 10 秒。",
    is_critical: true,  // 从 [CRITICAL] 标记解析
}
```

### 4.3 Examples 章节规范

Examples 提供 Few-shot 示例，帮助 Agent 理解预期行为。

**格式建议**：

```markdown
## Examples

### Example 1: 基础用法

> **User**: 把 users 表的 status 字段改成 enum 类型。
> 
> **Agent Action**:
> 1. 读取 users 表当前结构。
> 2. 创建 status_enum 类型：`CREATE TYPE status_enum AS ENUM ('active', 'inactive', 'pending')`
> 3. 修改列类型：`ALTER TABLE users ALTER COLUMN status TYPE status_enum USING status::status_enum`
> 4. 请求用户确认后执行。

### Example 2: 复杂迁移

> **User**: 在 orders 表添加 order_items 关联表。
> 
> **Agent Action**:
> 1. 分析 orders 表结构和现有关联。
> 2. 设计 order_items 表 Schema。
> 3. 创建外键约束。
> 4. 生成迁移脚本并请求审批。
```

### 4.4 代码块规范

代码块应标注语言类型，NSC 会保留代码块内容：

```markdown
## Procedures

1. 执行以下 SQL 检查当前表结构：

```sql
SELECT column_name, data_type, is_nullable 
FROM information_schema.columns 
WHERE table_name = 'users';
```

2. 使用 Python 脚本处理数据：

```python
import json

def transform_data(raw_data):
    return {
        "columns": raw_data.keys(),
        "values": raw_data.values()
    }
```
```

### 4.5 文件引用规范

引用其他文件时，使用相对路径：

```markdown
详细参考文档见 [REFERENCE.md](references/REFERENCE.md)。

执行脚本：
```bash
python scripts/migrate.py --config assets/config.yaml
```
```

**引用层级限制**：
- 建议引用深度不超过 1 层
- 避免深层嵌套的引用链

---

## 5. 完整示例

### 5.1 基础技能示例

```markdown
---
name: web-scraper
version: "1.0.0"
description: >-
  从指定 URL 提取表格数据并格式化为 JSON。
  当用户要求"抓取网页数据"、"提取表格"、"爬取网站"时触发。
license: MIT
compatibility: 需要 Python 3.10+ 和 BeautifulSoup4
metadata:
  author: nexa-dev
  category: data-extraction
---

# Web Scraper

从网页中提取结构化表格数据。

## Triggers

- 用户提到"抓取"、"爬取"、"提取表格"
- 用户提供 URL 并要求获取数据
- 用户需要从网页导出数据

## Procedures

1. 验证 URL 格式，确保是有效的 HTTP/HTTPS 地址。
2. [CRITICAL] 发送 HTTP GET 请求，超时设置为 10 秒。
3. 检查响应状态码，非 200 则报错。
4. 使用 BeautifulSoup 解析 HTML 内容。
5. 定位所有 `<table>` 标签。
6. 提取表头和行数据。
7. 转换为 JSON 格式输出。

## Examples

> **User**: 抓取 https://example.com/data 的表格数据。
> 
> **Agent**: 
> 1. 验证 URL...
> 2. 发送请求...
> 3. 解析 HTML...
> 4. 输出 JSON：
```json
{
  "headers": ["Name", "Value"],
  "rows": [["Item1", "100"], ["Item2", "200"]]
}
```

## Edge Cases

- 如果网页需要 JavaScript 渲染，提示用户使用 Selenium 方案
- 如果表格嵌套在复杂结构中，尝试多种定位策略
- 如果遇到反爬机制，建议降低频率或使用代理
```

### 5.2 高级技能示例（完整 NSC 扩展）

```markdown
---
name: postgres-schema-migration
version: "2.1.0"
description: >-
  执行 PostgreSQL 数据库表结构修改、数据迁移或复杂 SQL DDL 操作。
  当用户要求修改数据库架构、添加/删除列、创建索引时触发。
  不要在仅需要查询数据（SELECT）时触发此技能。

mcp_servers:
  - neon-postgres-admin
  - github-pr-creator

input_schema:
  type: object
  properties:
    target_table:
      type: string
      description: 目标表名
    migration_type:
      type: string
      enum: [add_column, drop_column, alter_type, rename_column, create_index]
    column_definition:
      type: object
      properties:
        name: { type: string }
        data_type: { type: string }
        nullable: { type: boolean, default: true }
  required: [target_table, migration_type]

hitl_required: true
security_level: high

pre_conditions:
  - 检查当前环境是否为非生产环境 (staging/dev)
  - 确认目标表存在于数据库中
  - 验证用户具有 ALTER TABLE 权限

post_conditions:
  - 执行 ORM 模型同步脚本
  - 运行数据库完整性检查
  - 更新 schema_version 表

fallbacks:
  - 如果遇到外键约束冲突，停止执行并列出受影响的关联表
  - 如果 SQL 执行超时 (>30s)，尝试分批处理
  - 如果回滚失败，记录错误并通知 DBA

permissions:
  - kind: db
    scope: "postgres:staging:ALTER"
  - kind: network
    scope: "https://api.github.com/*"
  - kind: exec
    scope: "git:*"

metadata:
  author: nexa-db-team
  category: database
  tags: [postgresql, migration, ddl]
---

# PostgreSQL Schema Migration

负责数据库架构演进的资深 DBA Agent 技能。

## Triggers

- 用户要求"修改表结构"、"添加列"、"删除列"
- 用户要求"修改字段类型"、"创建索引"
- 用户要求"执行数据库迁移"

## Context Gathering

1. 使用 `neon-postgres-admin` MCP 提取目标表的当前 Schema。
2. 检索代码库中与该表相关的 ORM 模型文件。
3. 检查是否存在外键约束或触发器依赖。

## Procedures

### 1. 现状评估

- 提取 `{{target_table}}` 的当前列定义。
- 分析现有索引和约束。
- 检查 ORM 模型是否需要同步更新。

### 2. 方案生成

- [CRITICAL] 编写 SQL 迁移脚本，必须包含 `UP` 和 `DOWN` 逻辑。
- 生成迁移文件命名：`{timestamp}_{migration_type}_{target_table}.sql`
- 在本地沙盒环境试运行 SQL。

### 3. 审批与执行

- 展示迁移方案给用户。
- [CRITICAL] 等待用户明确批准 (HITL)。
- 执行迁移脚本。
- 验证执行结果。

### 4. 后续处理

- 更新 ORM 模型文件。
- 提交 Git PR 包含迁移脚本。
- 更新 schema_version 记录。

## Examples

### Example 1: 添加列

> **User**: 在 users 表添加 last_login_at 时间戳字段。
> 
> **Agent Action**:
> 1. 读取 users 表结构：发现已有 id, name, email, created_at 列。
> 2. 生成迁移脚本：
```sql
-- UP
ALTER TABLE users ADD COLUMN last_login_at TIMESTAMP DEFAULT NULL;

-- DOWN
ALTER TABLE users DROP COLUMN last_login_at;
```
> 3. 请求用户审批...
> 4. 执行并验证。

### Example 2: 修改列类型

> **User**: 把 orders 表的 amount 字段从 INTEGER 改成 DECIMAL(10,2)。
> 
> **Agent Action**:
> 1. 检查现有数据范围...
> 2. 生成迁移脚本：
```sql
-- UP
ALTER TABLE orders ALTER COLUMN amount TYPE DECIMAL(10,2) USING amount::DECIMAL(10,2);

-- DOWN
ALTER TABLE orders ALTER COLUMN amount TYPE INTEGER USING amount::INTEGER;
```
> 3. [CRITICAL] 检查是否有计算依赖...
> 4. 请求审批并执行。

## Edge Cases

- **外键约束冲突**：停止执行，列出受影响表，请求用户决策
- **数据类型转换风险**：提示可能的数据丢失，建议先备份
- **生产环境警告**：如果检测到生产环境，拒绝自动执行

## Fallbacks

- 如果迁移失败，自动执行 DOWN 脚本回滚
- 如果回滚也失败，记录完整错误日志并通知管理员
- 如果 ORM 同步失败，提示手动更新

## References

- [PostgreSQL ALTER TABLE 文档](references/pg_alter_table.md)
- [迁移脚本模板](templates/migration_template.sql)
```

---

## 6. 校验规则汇总

### 6.1 Frontmatter 校验

| 字段 | 校验规则 | 错误级别 |
|------|----------|----------|
| `name` | kebab-case, 1-64字符, 与目录名匹配 | Error |
| `description` | 1-1024字符, 非空 | Error |
| `version` | 语义化版本格式 | Warning |
| `input_schema` | 有效 JSON Schema | Error |
| `mcp_servers` | 服务器在 Allowlist 中 | Warning |
| `permissions` | 格式正确, 权限在安全基线内 | Error |
| `security_level` | 有效枚举值 | Warning |

### 6.2 Body 校验

| 检查项 | 校验规则 | 错误级别 |
|--------|----------|----------|
| Procedures 存在性 | 必须包含 `## Procedures` 章节 | Error |
| Procedures 格式 | 有序列表格式 | Warning |
| Examples 参数 | 参数在 input_schema 范围内 | Warning |
| 高危词汇 | `rm -rf`, `DROP`, `DELETE` 等需权限声明 | Error |
| 文件引用 | 相对路径, 深度 ≤ 1 | Warning |

---

## 7. 版本兼容性

### 7.1 规范版本

NSC 支持的 SKILL.md 规范版本：

| 版本 | 状态 | 主要特性 |
|------|------|----------|
| `v1.0` | 稳定 | 基础字段 (name, description) |
| `v1.1` | 稳定 | MCP 支持, JSON Schema |
| `v2.0` | 当前 | NSC 扩展字段, 权限系统, Anti-Skill |

### 7.2 向后兼容策略

- 新增可选字段不影响旧文件编译
- 必选字段变更需要规范版本升级
- 编译器根据 `version` 字段选择校验规则集

---

## 8. 相关文档

- [COMPILER_PIPELINE.md](COMPILER_PIPELINE.md) - Frontend 解析实现
- [IR_DESIGN.md](IR_DESIGN.md) - SkillIR 数据结构映射
- [SECURITY_MODEL.md](SECURITY_MODEL.md) - 权限审计与安全等级
- [API_REFERENCE.md](API_REFERENCE.md) - 校验 API 定义