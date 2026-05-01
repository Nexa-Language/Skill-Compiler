# 论文配图简短描述

> 详细版本备份见 [`figures_prompt.md`](figures_prompt.md)
>
> 本文档为每张图提供简短描述，只说明要有哪些部分和核心内容，
> 不规定具体颜色、尺寸、圆角等视觉细节，保证风格一致即可。
>
> **最后更新**: 2026-05-01，与论文 Section 3 最新设计和源码 100% 对齐。

---

## 通用风格

所有图保持统一的学术系统论文风格（参考 LLVM/TVM/PyTorch 论文配图）：
- 统一配色方案，同一组件在不同图中颜色一致（如 ClaudeEmitter 始终同一色系）
- 横向从左到右的数据流方向
- Phase/层用大框包裹，子组件用小框
- 关键创新点用强调色标注，安全组件用另一种强调色标注
- 箭头标注数据流类型（RawAST、SkillIR、ValidatedSkillIR 等）
- **不包含**具体库名（如 serde_yaml、pulldown-cmark）或文件名（如 frontmatter.rs）

---

## Fig 1: NSC 系统架构总览图

**位置**: Section 3.1 开头 | **尺寸**: 双栏宽 | **优先级**: 必须

横向四阶段管线架构图，从左到右：

1. **Input**: SKILL.md 源文件，分叉标注 YAML Frontmatter 和 Markdown Body
2. **Phase 1 Frontend**: 内含 FrontmatterParser（YAML 解析）、MarkdownParser（事件流解析）、ASTBuilder（状态机组装），三者汇聚输出 RawAST
3. **Phase 2 IR Construction**: 内含 SkillIR Builder（类型映射）、NestedDataDetector（⚡创新点：嵌套深度检测，depth≥3 设 YAML flag）、Type Mapper。输出 SkillIR
4. **Phase 3 Analyzer**: 五个子组件链式排列 — SchemaValidator、MCPDependencyChecker、PermissionAuditor（🔒安全）、AntiSkillInjector（⚡创新点：编译期自动注入安全约束）、NestedDataDetector。右侧虚线框 Diagnostic 收集诊断信息。输出 ValidatedSkillIR
5. **Phase 4 Backend**: 上方 EmitterRegistry，下方四个 Emitter 并列 — ClaudeEmitter（XML）、CodexEmitter（XML-Tagged Markdown）、GeminiEmitter（MD+条件YAML）、KimiEmitter（完整MD）。旁边 RoutingManifestGenerator（⚡创新点：渐进式路由）
6. **Output**: Claude `SKILL.md`、Codex `{skill-name}.md`、Gemini `{skill-name}.md`（可能附带YAML assets）、Kimi `SKILL.md`、routing_manifest.yaml / skills_index.json、manifest.json

三个创新点用强调色标注：Anti-Skill Injection、Nested Data Detection、Progressive Routing。安全组件用另一种强调色标注。

**注意**: CodexEmitter 输出是 XML-Tagged Markdown（`<skill>`, `<instructions>`, `<constraints>` 标签），不是纯 Markdown。

---

## Fig 2: Compiler 定位与 m×n → m+n

**位置**: Section 3.1 Architecture Overview | **尺寸**: 双栏宽 | **优先级**: 必须

左右双栏布局：

**左半边 — Agent Skill Invocation Flow**：纵向五层流程：
1. Skill Authoring Layer：开发者编写 SKILL.md（统一源）
2. Compilation Layer：NSC 编译器，分叉到四个平台（标注 "Write Once, Run Anywhere"）
3. Agent Initialization Layer：启动时仅加载 routing_manifest.yaml（标注 "~50 tokens/skill"）
4. Routing & Matching Layer：语义路由匹配（隐式 description match / 显式 @skill-name）
5. Execution Layer：匹配后动态加载完整 SKILL.md 执行

**右半边 — m×n → m+n Compiler Argument**：上下对比：
- 上半部分（问题）：m 个技能 × n 个平台的矩阵网格，每个交叉点是一个手动适配（红色小方块），标注 "m × n adaptations"
- 下半部分（解决方案）：m 个技能 → NSC Compiler 中间大框 → n 个 Emitter，标注 "m + n components"
- 中间大箭头标注 "O(m×n) → O(m+n)"

---

## Fig 3: SkillIR 样例（论文中 Figure 2）

**位置**: Section 3.2 | **尺寸**: 双栏宽 | **优先级**: 必须

展示一个简化版 SkillIR 的 JSON 结构，基于真实源码定义：

```json
{
  "name": "github-api-client",
  "version": "1.0.0",
  "description": "Interact with GitHub REST API for repository management",
  "mcp_servers": ["github-mcp"],
  "input_schema": { "type": "object", "properties": { ... } },
  "security_level": "high",
  "hitl_required": true,
  "permissions": [
    { "kind": "network", "scope": "https://api.github.com/*", "read_only": false }
  ],
  "procedures": [
    { "order": 1, "instruction": "Validate GitHub token from environment", "is_critical": true },
    { "order": 2, "instruction": "Construct REST request from input parameters" },
    { "order": 3, "instruction": "Execute HTTP POST to GitHub API endpoint" }
  ],
  "anti_skill_constraints": [
    {
      "source": "anti-skill-injector",
      "content": "Never execute an HTTP request without a timeout parameter...",
      "level": "warning",
      "scope": "global"
    }
  ],
  "requires_yaml_optimization": false,
  "mode": "sequential"
}
```

重点标注：
- `anti_skill_constraints` 字段（Analyzer 自动注入）
- `requires_yaml_optimization` 和 `nested_data_depth`（AST 优化标志）
- 六个分类区域用不同背景色区分

---

## Fig 4: 四平台 Format 对比（论文中 Figure 3）

**位置**: Section 3.4 | **尺寸**: 双栏宽 | **优先级**: 必须

展示同一 SkillIR 在四个 Emitter 下的输出差异：

**SkillIR 输入**（顶部统一展示）：
- name: "data-migration"
- procedures: [3 steps]
- input_schema: { nested depth = 4 }
- anti_skill_constraints: [1 HTTP safety]

**四个输出并列**：

1. **Claude (XML)**:
   ```xml
   <agent_skill>
     <execution_steps>
       <step order="1" critical="true">...</step>
     </execution_steps>
     <strict_constraints>
       <anti_pattern source="anti-skill-injector">...</anti_pattern>
     </strict_constraints>
   </agent_skill>
   ```

2. **Codex (XML-Tagged Markdown)**:
   ```xml
   <skill name="data-migration">
     <instructions>...</instructions>
     <constraints>
       <forbidden>...</forbidden>
     </constraints>
   </skill>
   ```

3. **Gemini (Markdown + YAML)**:
   ```markdown
   # data-migration
   ## Procedures
   1. ... **[CRITICAL]**
   ## Parameter Schema (YAML Optimized)
   ```yaml
   type: object
   properties:
     migration_config:
       type: object
       properties:
         source_db:
           type: object
           properties:
             host: { type: string }
   ```
   ```

4. **Kimi (Full Markdown)**:
   ```markdown
   # data-migration
   ## Description
   ...
   ## Procedures
   1. ... **[CRITICAL]**
   ## Parameter Schema
   - `migration_config.source_db.host` (string): ...
   ```

重点标注：
- Gemini 的条件 YAML 渲染（depth ≥ 3 触发）
- 所有平台都包含 anti-skill constraints
- Codex 使用 XML 标签（不是纯 Markdown）

---

## Fig 5: 渐进式路由机制对比图

**位置**: Section 3.4 | **尺寸**: 单栏宽 | **优先级**: 推荐

左右对比：

- **左：Traditional Full Loading** — Agent 启动加载 15 个完整 SKILL.md，标注 ≈150K tokens，列出三个问题（注意力分散、高 API 成本、幻觉风险）
- **右：NSC Progressive Disclosure** — 启动仅加载 routing_manifest.yaml（≈750 tokens，标注 99.5% savings），用户请求后语义路由匹配，仅动态加载 1 个匹配技能（≈10K tokens）
- 底部对比条形图：Traditional 150K vs NSC 750+10K

---

## Fig 6: Anti-Skill 注入机制示意图

**位置**: Section 3.3 | **尺寸**: 单栏宽 | **优先级**: 推荐

横向流程：
- **左侧**：原始 Procedure 步骤文本（含 HTTP、BeautifulSoup、DROP、loop 等关键词）
- **中间**：四类 Anti-Pattern 规则框（HTTP/HTML Parse/Destructive DB/Loop），标注触发关键词和对应约束内容，用匹配状态标记
- **右侧**：增强后的 SkillIR，anti_skill_constraints 字段中列出自动注入的安全约束，标注 "compile-time auto-injection"

具体案例展示：
- Input: "Fetch the user profile using an HTTP GET request to the API endpoint."
- Match: HTTP safety rule (keywords: "HTTP", "GET")
- Injected: "Never execute an HTTP request without a timeout parameter (default 10s). Do not retry more than 3 times on 403 Forbidden errors."

---

## Fig 7: 格式敏感性与 AST 优化决策图

**位置**: Section 3.2 + 3.4 | **尺寸**: 双栏宽 | **优先级**: 可选

三部分组合：
- **上部**：嵌套数据格式准确率柱状图（YAML 51.9% > MD 48.2% > JSON 43.1% > XML 33.8%）
- **中部**：AST 优化决策树 — NestedDataDetector 计算 depth，depth<3 用 Markdown，depth≥3 用 YAML；同时展示四平台各自策略（Claude→XML, Codex→XML-Tagged MD, Gemini→条件YAML, Kimi→完整MD）
- **下部**：四平台输出示例片段，各标注核心策略关键词

---

## 优先级总结

| Figure | 必须/推荐/可选 | 说明 |
|--------|---------------|------|
| Fig 1 | 必须 | 核心架构图 |
| Fig 2 | 必须 | 动机论述核心视觉 |
| Fig 3 | 必须 | SkillIR 真实样例（论文 Figure 2） |
| Fig 4 | 必须 | 四平台 Format 对比（论文 Figure 3） |
| Fig 5 | 推荐 | 渐进式路由对比 |
| Fig 6 | 推荐 | Anti-Skill 注入机制 |
| Fig 7 | 可选 | 格式敏感性+AST优化 |
