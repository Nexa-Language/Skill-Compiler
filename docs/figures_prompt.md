# 论文配图详细描述与提示词

> **Nexa Skill Compiler 论文配图设计文档**
>
> 本文档为每张论文配图提供详细的视觉描述、布局规范、元素清单和风格要求，
> 作为 AI 画图的输入提示词。每张图独立描述，可分别交给不同工具生成。

---

## 通用风格规范

以下规范适用于所有配图：

- **风格**: 学术论文标准系统架构图风格，参考 LLVM、TVM、PyTorch 等顶会系统论文的配图风格
- **配色**: 使用专业、克制的配色方案。主色调为深蓝 (#2B3A67) 和浅灰 (#F0F2F5)，强调色为橙色 (#E8734A) 用于标注关键创新点，绿色 (#4CAF50) 用于标注安全相关组件
- **字体**: 英文使用 Helvetica/Arial 无衬线字体，中文使用思源黑体（Noto Sans CJK）。标题 14pt，正文 11pt，标注 9pt
- **线条**: 组件边框使用 1.5pt 实线，数据流箭头使用 2pt 实线带箭头，虚线表示可选/异步路径
- **阴影**: 组件框使用极轻微阴影（offset 2px, blur 4px, opacity 10%），不使用重阴影
- **圆角**: 所有矩形组件框使用 4px 圆角，phase 大框使用 8px 圆角
- **图标**: 不使用emoji或装饰性图标，使用简洁的几何符号（如小圆点、小三角）标注数据流方向
- **布局**: 横向从左到右的数据流方向，纵向分层排列，保持视觉平衡
- **尺寸**: 每张图最终输出为 300dpi PNG，宽度适配单栏（3.5英寸）或双栏（7英寸）论文排版

---

## Fig 1: NSC 系统架构总览图

### 图表定位

这是论文最核心的架构图，放置在 Section 3 (System Design / Architecture) 的开头。
参考 LLVM 论文的经典架构图风格：横向四阶段管线，每阶段内部展示子组件，阶段间用粗箭头连接并标注数据流类型。

### 整体布局

**横向四阶段管线布局**，从左到右依次为：

```
[Input] → [Phase 1: Frontend] → [Phase 2: IR Construction] → [Phase 3: Analyzer] → [Phase 4: Backend] → [Output]
```

每个 Phase 用一个大的浅色矩形框包裹，内部放置子组件。Phase 之间用粗箭头连接，箭头上方标注传递的数据结构名称。

### 详细元素清单

#### Input 区域（最左侧，窄列）

- 一个文档图标，标注 `SKILL.md`
- 下方两个小标签：`YAML Frontmatter` 和 `Markdown Body`
- 用细线从 SKILL.md 分叉到这两个标签

#### Phase 1: Frontend（浅蓝色大框 #E8F0FE）

框标题：**Phase 1: Frontend**（左上角，粗体）

内部三个子组件（纵向排列，各用白色小矩形框）：

1. **FrontmatterParser** — 标注 `serde_yaml + regex`，下方小字 "extract YAML metadata"
2. **MarkdownParser** — 标注 `pulldown-cmark`，下方小字 "event stream parsing"
3. **ASTBuilder** — 标注 `state machine`，下方小字 "assemble syntax tree"

三个子组件之间用细箭头连接：FrontmatterParser → ASTBuilder，MarkdownParser → ASTBuilder

#### Phase 1 → Phase 2 粗箭头

箭头上方标注：**RawAST**

#### Phase 2: IR Construction（浅绿色大框 #E8F8E8）

框标题：**Phase 2: IR Construction**（左上角，粗体）

内部三个子组件：

1. **SkillIR Builder** — 标注 "type mapping & struct conversion"
2. **NestedDataDetector** — 用橙色强调框标注 ⚡ "AST Optimization"，下方小字 "compute nesting depth ≥ 3 → YAML flag"
3. **Type Mapper** — 标注 "FrontmatterMeta → SkillIR fields"

子组件连接：SkillIR Builder ← Type Mapper，SkillIR Builder ← NestedDataDetector

#### Phase 2 → Phase 3 粗箭头

箭头上方标注：**SkillIR**

#### Phase 3: Analyzer（浅橙色大框 #FFF3E0）

框标题：**Phase 3: Analyzer**（左上角，粗体）

内部五个子组件（纵向链式排列，每个用白色小矩形框）：

1. **SchemaValidator** — 标注 "field type & constraint check"，右侧小标签 `Warning`
2. **MCPDependencyChecker** — 标注 "server allowlist verification"，右侧小标签 `Error`
3. **PermissionAuditor** — 用绿色强调框标注 🔒 "Security"，下方小字 "baseline audit & scope validation"
4. **AntiSkillInjector** — 用橙色强调框标注 ⚡ "Key Innovation"，下方小字 "auto-inject safety constraints by pattern detection"
5. **NestedDataDetector** — 标注 "set requires_yaml_optimization flag"

五个子组件用细箭头链式连接：1→2→3→4→5

右侧有一个虚线框标注 **Diagnostic**，用虚线从 SchemaValidator、MCPDependencyChecker、PermissionAuditor 连接到 Diagnostic

#### Phase 3 → Phase 4 粗箭头

箭头上方标注：**ValidatedSkillIR**

#### Phase 4: Backend（浅紫色大框 #F3E8F8）

框标题：**Phase 4: Backend**（左上角，粗体）

内部结构为上下两部分：

**上半部分：EmitterRegistry**（白色大矩形框）

- 中间一个矩形标注 **EmitterRegistry**
- 从 Registry 分叉出四条线到四个 Emitter

**下半部分：四个 Emitter**（横向排列，各用不同颜色小矩形框）

1. **ClaudeEmitter** — 深蓝色框，标注 "XML Tags"，下方小字 "semantic layering +23% accuracy"
2. **CodexEmitter** — 绿色框，标注 "Pure Markdown"，下方小字 "format tax elimination"
3. **GeminiEmitter** — 红色框，标注 "MD + YAML"，下方小字 "AST optimization: YAML 51.9% > JSON 43.1%"
4. **KimiEmitter** — 灰色框，标注 "Full Markdown"，下方小字 "ultra-long context preservation"

**右侧附加组件：RoutingManifestGenerator**（橙色强调框）
- 标注 "Progressive Routing"
- 下方小字 "only name + description → ~50 tokens/skill"
- 用虚线从 ValidatedSkillIR 连接到此组件

#### Output 区域（最右侧，窄列）

纵向排列多个产物图标：

1. `SKILL.md` (Claude格式) — 深蓝色标签
2. `skill-name.md` (Codex格式) — 绿色标签
3. `skill-name.md` (Gemini格式) — 红色标签
4. `SKILL.md` (Kimi格式) — 灴色标签
5. `routing_manifest.yaml` — 橙色标签
6. `manifest.json` — 灰色标签
7. `signature.sha256` — 小灰色标签

四个 Emitter 各用箭头连到对应的产物标签。RoutingManifestGenerator 用箭头连到 routing_manifest.yaml。

### 关键创新点标注方式

图中三个关键创新点用橙色 (#E8734A) 虚线框 + 小闪电符号 ⚡ 标注：

1. **Anti-Skill Injection** (Phase 3) — 编译期自动注入安全约束
2. **AST Optimization / NestedDataDetector** (Phase 2+3) — 嵌套数据深度检测驱动格式选择
3. **Progressive Routing Manifest** (Phase 4) — 渐进式路由清单解决上下文膨胀

安全相关组件用绿色 (#4CAF50) 虚线框 + 小锁符号 🔒 标注：

1. **PermissionAuditor** (Phase 3)
2. **SecurityLevel** (SkillIR 内)

### 尺寸与排版

- 整图宽度：双栏 7 英寸 (180mm)
- 整图高度：约 3.5 英寸 (90mm)
- Phase 框间距：8mm
- 子组件框间距：4mm

---

## Fig 2: Compiler 定位与 m×n → m+n 示意图

### 图表定位

放置在 Section 1 (Introduction / Motivation) 中，用于阐述核心动机。
左半边展示 NSC 在 Agent 调用链中的位置，右半边展示经典编译器论点 m×n → m+n。

### 整体布局

**左右对称双栏布局**，中间用竖线分隔：

```
[左半边: Agent Invocation Flow with NSC] | [右半边: m×n → m+n Compiler Argument]
```

### 左半边详细描述

**标题**: Agent Skill Invocation Flow

**纵向五层流程图**，从上到下：

1. **Skill Authoring Layer**（最上方，浅灰色大框）
   - 内部一个人物图标 + 文档图标，标注 "Skill Developer"
   - 产物标注 `SKILL.md (unified source)`
   - 用粗箭头向下

2. **Compilation Layer**（浅蓝色大框，橙色边框强调）
   - 内部标注 **NSC Compiler**
   - 下方四个小标签横向排列：`Claude` `Codex` `Gemini` `Kimi`
   - 用粗箭头向下

3. **Agent Initialization Layer**（浅绿色大框）
   - 内部标注 "Agent Startup"
   - 右侧小框标注 `routing_manifest.yaml`，小字 "only ~50 tokens/skill"
   - 用箭头向下到路由匹配

4. **Routing & Matching Layer**（浅黄色大框）
   - 内部标注 "Semantic Routing"
   - 两个小标签：`Implicit: description match` 和 `Explicit: @skill-name`
   - 用箭头向下到动态加载

5. **Execution Layer**（最下方，浅灰色大框）
   - 内部标注 "Dynamic Skill Loading + Execution"
   - 小字 "full SKILL.md injected only when matched"
   - 右侧四个 Agent 图标：Claude Code, Codex CLI, Gemini CLI, Kimi CLI

**关键标注**：
- 在 Compilation Layer 旁边用橙色标注框写 "Write Once, Run Anywhere"
- 在 Routing & Matching Layer 旁边用绿色标注框写 "Progressive Disclosure: 99.5% token savings"

### 右半边详细描述

**标题**: The Compiler Argument: m×n → m+n

**上下对比布局**：

**上半部分：Without Compiler（红色/灰色调，表示问题）**

- 标题：❌ "Without Compiler: m × n Manual Adaptations"
- 一个矩阵/网格图：
  - 左侧纵向列出 m 个技能：`skill-1`, `skill-2`, `skill-3`, ..., `skill-m`
  - 顶部横向列出 n 个平台：`Claude`, `Codex`, `Gemini`, `Kimi`, ..., `Platform-n`
  - 矩阵内部每个交叉点用小红色方块填充，表示需要手动适配
  - 右下角标注 "m × n = 12 adaptations"（以 m=3, n=4 为例）
  - 每个小方块标注 "manual rewrite"

**下半部分：With Compiler（蓝色/绿色调，表示解决方案）**

- 标题：✅ "With NSC Compiler: m + n Components"
- 一个简化的线性图：
  - 左侧纵向列出 m 个技能：`skill-1`, `skill-2`, `skill-3`, ..., `skill-m`（用蓝色标签）
  - 中间一个大框标注 **NSC Compiler**（橙色强调）
  - 右侧纵向列出 n 个 Emitter：`ClaudeEmitter`, `CodexEmitter`, `GeminiEmitter`, `KimiEmitter`, ..., `Emitter-n`（用绿色标签）
  - 所有技能用箭头指向 Compiler，Compiler 用箭头指向所有 Emitter
  - 右下角标注 "m + n = 7 components"（以 m=3, n=4 为例）

**中间过渡箭头**：
- 从上半部分到下半部分，用一个大粗箭头标注 "Compiler reduces O(m×n) → O(m+n)"

### 尺寸与排版

- 整图宽度：双栏 7 英寸 (180mm)
- 整图高度：约 4 英寸 (100mm)
- 左右各占 50% 宽度
- 中间竖线分隔

---

## Fig 3: 编译管线数据流详图

### 图表定位

放置在 Section 3.1 (Compilation Pipeline) 中，展示每个阶段的具体数据转换过程。
这张图比 Fig 1 更详细，聚焦于数据结构和转换逻辑。

### 整体布局

**横向流水线布局**，每个阶段展示输入数据结构 → 转换操作 → 输出数据结构：

```
[SKILL.md] → [RawAST] → [SkillIR] → [ValidatedSkillIR] → [Platform Outputs]
```

在每个数据结构位置，展示其核心字段的简化类图。

### 详细元素清单

#### SKILL.md（最左侧）

展示一个简化的源文件示例框：

```
---
name: database-migration
description: Execute PostgreSQL DDL operations
security_level: critical
permissions: [{kind: database, scope: "postgres:*:ALL"}]
hitl_required: true
---

# PostgreSQL Schema Migration
## Procedures
1. Analyze current schema state
2. Generate migration SQL with rollback
3. Execute in transaction block
```

用箭头指向 RawAST

#### RawAST

展示简化类图框：

```
RawAST {
  source_path: String
  frontmatter: FrontmatterMeta
  body: MarkdownBody
  source_hash: SHA-256
}
```

FrontmatterMeta 子框（虚线连接）：

```
FrontmatterMeta {
  name: String
  description: String
  permissions: Vec<PermissionDecl>
  security_level: Option<String>
  mcp_servers: Option<Vec<String>>
  ...
}
```

MarkdownBody 子框（虚线连接）：

```
MarkdownBody {
  sections: Vec<Section>
  procedures: Vec<RawProcedureStep>
  examples: Vec<RawExample>
}
```

用箭头指向 SkillIR，箭头标注 "build_ir()"

#### SkillIR

展示简化类图框（核心字段）：

```
SkillIR {
  name: Arc<str>          // zero-copy
  version: Arc<str>
  description: String
  security_level: SecurityLevel
  permissions: Vec<Permission>
  procedures: Vec<ProcedureStep>
  anti_skill_constraints: Vec<Constraint>
  requires_yaml_optimization: bool  // ⚡ AST flag
  nested_data_depth: Option<usize>  // ⚡ AST flag
  ...
}
```

SecurityLevel 子框（绿色强调）：

```
SecurityLevel = Low | Medium | High | Critical
```

用箭头指向 ValidatedSkillIR，箭头标注 "Analyzer.analyze()"

#### ValidatedSkillIR

展示简化类图框：

```
ValidatedSkillIR {
  inner: SkillIR           // validated + enhanced
  warnings: Vec<Diagnostic>  // non-blocking
}
```

右侧虚线框标注 Analyzer Chain：

```
SchemaValidator → MCPDependencyChecker
  → PermissionAuditor → AntiSkillInjector
  → NestedDataDetector
```

用箭头指向 Platform Outputs

#### Platform Outputs（最右侧，四个并列产物框）

1. **Claude Output** — 展示简化 XML 片段：
   ```
   <skill name="database-migration">
     <procedures>
       <step order="1">Analyze schema</step>
     </procedures>
     <anti-skill-constraint>
       Never execute DROP without confirmation
     </anti-skill-constraint>
   </skill>
   ```

2. **Codex Output** — 展示简化 Markdown 片段：
   ```
   # database-migration
   ## Procedures
   1. Analyze current schema state
   ## Constraints
   > Never execute DROP without confirmation
   ```

3. **Gemini Output** — 展示简化 MD+YAML 片段：
   ```
   # database-migration
   ## Input Schema
   ```yaml
   data:
     nested:
       deep: string
   ```
   ```

4. **Kimi Output** — 展示简化完整 Markdown 片段：
   ```
   # database-migration (v1.0.0)
   **Security Level**: critical | **HITL**: required
   ## Full Procedures with all details...
   ```

### 尺寸与排版

- 整图宽度：双栏 7 英寸 (180mm)
- 整图高度：约 3 英寸 (75mm)
- 数据结构框用浅灰背景 + 深色边框
- 代码片段框用浅黄背景 (#FFFDE7) + 细边框

---

## Fig 4: 渐进式路由机制对比图

### 图表定位

放置在 Section 3.4 (Progressive Routing) 或 Section 4 (Evaluation) 中，
用于直观展示渐进式披露机制对上下文膨胀的解决效果。

### 整体布局

**左右对比布局**：

```
[左: Traditional Approach - Context Bloat] | [右: NSC Progressive Routing - Token Savings]
```

### 左半边详细描述

**标题**: ❌ Traditional: Full Skill Loading

**流程图**（纵向）：

1. Agent Startup → 加载全量 SKILL.md（15个技能）
2. 展示一个大的"膨胀上下文"框：
   - 内部堆叠 15 个完整的 SKILL.md 文档图标
   - 标注 "≈ 150,000 tokens"
   - 用红色标注 "Context Bloat"
3. 下方展示三个问题标签（红色）：
   - "Attention Dilution" — 模型注意力被无关内容分散
   - "High API Cost" — 每次对话都加载全量
   - "Hallucination Risk" — 信息过载导致幻觉

### 右半边详细描述

**标题**: ✅ NSC: Progressive Disclosure

**流程图**（纵向）：

1. Agent Startup → 仅加载 routing_manifest.yaml
2. 展示一个小的"精简路由表"框：
   - 内部展示简化的 YAML 片段：
     ```
     skills:
       - name: db-migration
         description: Execute DDL
       - name: api-client  
         description: Call REST APIs
       - name: file-organizer
         description: Organize files
     ```
   - 标注 "≈ 750 tokens (50 × 15)"
   - 用绿色标注 "99.5% Savings"
3. 用户请求 → Semantic Routing Match
4. 匹配成功 → Dynamic Load 1 个完整 SKILL.md
5. 展示一个中等大小的框：
   - 内部只有 1 个 SKILL.md 文档图标
   - 标注 "≈ 10,000 tokens (only matched skill)"
   - 用绿色标注 "Focused Attention"

### 底部对比条

一个横向对比条形图：

```
Traditional:  ████████████████████████████████████████ 150K tokens
NSC Routing:  ██ 750 tokens (initial) + 10K (on demand)
```

标注 "Token reduction: 99.5% at initialization, 93.3% per invocation"

### 尺寸与排版

- 整图宽度：双栏 7 英寸 (180mm)
- 整图高度：约 3.5 英寸 (90mm)

---

## Fig 5: 格式敏感性与 AST 优化决策图

### 图表定位

放置在 Section 3.3 (Backend Design / Format Sensitivity) 中，
展示 NSC 如何基于实证研究为不同平台选择最优输出格式。

### 整体布局

**三部分组合布局**：

```
[上部: Format Sensitivity Research Data] → [中部: AST Optimization Decision Flow] → [下部: Platform-Specific Output Examples]
```

### 上部：格式敏感性研究数据

**小型柱状图**，展示嵌套数据格式准确率对比：

```
YAML:    ████████████████████████████████████ 51.9%
Markdown: ██████████████████████████████████ 48.2%
JSON:    ███████████████████████████████ 43.1%
XML:     █████████████████████████ 33.8%
```

标注来源："Empirical study on nested data parsing accuracy across formats"

每个柱状条使用不同颜色：
- YAML: 绿色 (#4CAF50)
- Markdown: 蓝色 (#2196F3)
- JSON: 橙色 (#FF9800)
- XML: 红色 (#F44336)

### 中部：AST 优化决策流程

**决策树流程图**：

```
ValidatedSkillIR
  │
  ├─→ NestedDataDetector: compute depth
  │     │
  │     ├─ depth < 3 → requires_yaml_optimization = false
  │     │     │
  │     │     └─→ GeminiEmitter: use Markdown format
  │     │
  │     └─ depth ≥ 3 → requires_yaml_optimization = true
  │           │
  │           └─→ GeminiEmitter: use YAML format for nested data
  │
  ├─→ Platform = Claude → ClaudeEmitter: XML tags
  │
  ├─→ Platform = Codex → CodexEmitter: Pure Markdown (no JSON)
  │     │
  │     └─→ "Decoupled Reasoning & Formatting"
  │         "API layer handles structured output"
  │
  └─→ Platform = Kimi → KimiEmitter: Full Markdown
```

决策节点用菱形框，处理节点用矩形框，创新标注用橙色虚线框。

### 下部：平台特定输出示例

四个并列小框，每个展示对应平台的一段输出片段（与 Fig 3 类似但更简洁）：

1. **Claude** — XML 标签嵌套结构
2. **Codex** — 纯 Markdown 结构化指令
3. **Gemini** — Markdown + YAML 嵌套数据块
4. **Kimi** — 完整 Markdown 保留所有细节

每个框下方标注核心策略关键词：
- Claude: "Semantic Layering"
- Codex: "Format Tax Elimination"
- Gemini: "AST Optimization"
- Kimi: "Context Preservation"

### 尺寸与排版

- 整图宽度：双栏 7 英寸 (180mm)
- 整图高度：约 4 英寸 (100mm)
- 上部柱状图占 25% 高度
- 中部决策树占 50% 高度
- 下部示例占 25% 高度

---

## Fig 6: Anti-Skill 注入机制示意图

### 图表定位

放置在 Section 3.2 (Analyzer / Security Model) 中，
展示 Anti-Skill 注入如何基于模式检测自动增强技能安全性。

### 整体布局

**横向流程图**：

```
[Procedure Text] → [Pattern Detection] → [Constraint Injection] → [Enhanced SkillIR]
```

### 详细元素清单

#### 左侧：原始 Procedure 文本

展示一个简化的步骤列表框：

```
Procedures:
  1. "Execute HTTP GET request to fetch data"
  2. "Parse HTML response with BeautifulSoup"
  3. "DROP temporary tables after migration"
  4. "Loop through results until complete"
```

用箭头指向 Pattern Detection

#### 中间：Pattern Detection

展示四个 Anti-Pattern 规则框（纵向排列）：

1. 🔴 **HTTP Pattern** — trigger: `["HTTP", "GET", "POST", "fetch"]`
   - constraint: "Never execute HTTP request without timeout (default 10s). No retry > 3 on 403."

2. 🔴 **HTML Parse Pattern** — trigger: `["BeautifulSoup", "HTML parse", "scrape"]`
   - constraint: "Do not parse raw JS variables with HTML parsers. Fallback to Regex for <script>."

3. 🔴 **Destructive DB Pattern** — trigger: `["DROP", "DELETE", "TRUNCATE"]`
   - constraint: "Never execute destructive DB ops without user confirmation. Show affected rows first."

4. 🔴 **Loop Pattern** — trigger: `["while", "loop", "repeat"]`
   - constraint: "All loops must have max iteration limit (default 1000). Implement counter + break."

每个规则框左侧用红色小圆点标注触发关键词匹配状态（matched ✓ 或 unmatched ✗）。

在示例中，步骤1匹配 HTTP Pattern ✓，步骤2匹配 HTML Parse Pattern ✓，步骤3匹配 Destructive DB Pattern ✓，步骤4匹配 Loop Pattern ✓。

用箭头指向 Constraint Injection

#### 右侧：Constraint Injection 结果

展示增强后的 SkillIR 框：

```
SkillIR {
  procedures: [4 steps unchanged]
  anti_skill_constraints: [
    Constraint { source: "anti-skill-injector",
                 content: "HTTP timeout rule...",
                 level: Warning, scope: Global },
    Constraint { source: "anti-skill-injector",
                 content: "HTML parser fallback rule...",
                 level: Warning, scope: Global },
    Constraint { source: "anti-skill-injector",
                 content: "Destructive DB confirmation rule...",
                 level: Warning, scope: Global },
    Constraint { source: "anti-skill-injector",
                 content: "Loop iteration limit rule...",
                 level: Warning, scope: Global },
  ]
}
```

用橙色标注框标注 "4 safety constraints auto-injected at compile time"

### 尺寸与排版

- 整图宽度：双栏 7 英寸 (180mm)
- 整图高度：约 2.5 英寸 (65mm)

---

## 配图优先级与论文排版建议

| Figure | 优先级 | 论文位置 | 尺寸建议 | 说明 |
|--------|--------|----------|----------|------|
| Fig 1 | ⭐⭐⭐ 必须 | Section 3 开头 | 双栏 7英寸 | 最核心的架构图 |
| Fig 2 | ⭐⭐⭐ 必须 | Section 1 Introduction | 双栏 7英寸 | 动机论述的核心视觉论证 |
| Fig 3 | ⭐⭐ 推荐 | Section 3.1 Pipeline | 双栏 7英寸 | 数据流详图，补充 Fig 1 |
| Fig 4 | ⭐⭐ 推荐 | Section 3.4 或 Section 4 | 单栏 3.5英寸 | 渐进式路由对比 |
| Fig 5 | ⭐ 可选 | Section 3.3 Backend | 双栏 7英寸 | 格式敏感性+AST优化 |
| Fig 6 | ⭐ 可选 | Section 3.2 Analyzer | 单栏 3.5英寸 | Anti-Skill 注入机制 |

**建议**：Fig 1 和 Fig 2 是必须画的，Fig 3-6 根据论文篇幅和页数限制决定是否纳入。如果页数紧张，Fig 3 可以与 Fig 1 合并（在 Fig 1 中增加更多数据结构细节），Fig 4 可以简化为文字描述中的小图。

---

## 各图生成注意事项

1. **一致性**：所有图中相同组件使用相同的颜色和命名。例如 ClaudeEmitter 在所有图中都使用深蓝色，Anti-Skill 在所有图中都使用橙色标注。
2. **简洁性**：每张图只展示核心信息，避免过度细节。代码片段最多 5-6 行。
3. **可读性**：确保所有文字在 300dpi 下清晰可读，最小字体不低于 8pt。
4. **矢量格式**：优先生成 SVG 或 PDF 矢量格式，再转换为 PNG。这样在论文排版时可以缩放而不失真。
5. **黑白兼容**：确保图表在黑白打印时仍然可区分（使用不同灰度或线条样式区分组件）。
6. **无水印**：所有生成的图不包含任何工具水印或版权标记。