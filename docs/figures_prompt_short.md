# 论文配图简短描述

> 详细版本备份见 [`figures_prompt.md`](figures_prompt.md)
>
> 本文档为每张图提供简短描述，只说明要有哪些部分和核心内容，
> 不规定具体颜色、尺寸、圆角等视觉细节，保证风格一致即可。

---

## 通用风格

所有图保持统一的学术系统论文风格（参考 LLVM/TVM/PyTorch 论文配图）：
- 统一配色方案，同一组件在不同图中颜色一致（如 ClaudeEmitter 始终同一色系）
- 横向从左到右的数据流方向
- Phase/层用大框包裹，子组件用小框
- 关键创新点用强调色标注，安全组件用另一种强调色标注
- 箭头标注数据流类型（RawAST、SkillIR、ValidatedSkillIR 等）

---

## Fig 1: NSC 系统架构总览图

**位置**: Section 3 开头 | **尺寸**: 双栏宽 | **优先级**: 必须

横向四阶段管线架构图，从左到右：

1. **Input**: SKILL.md 源文件，分叉标注 YAML Frontmatter 和 Markdown Body
2. **Phase 1 Frontend**: 内含 FrontmatterParser（serde_yaml）、MarkdownParser（pulldown-cmark）、ASTBuilder（状态机），三者汇聚输出 RawAST
3. **Phase 2 IR Construction**: 内含 SkillIR Builder（类型映射）、NestedDataDetector（⚡创新点：嵌套深度检测设 YAML flag）、Type Mapper。输出 SkillIR
4. **Phase 3 Analyzer**: 五个子组件链式排列 — SchemaValidator、MCPDependencyChecker、PermissionAuditor（🔒安全）、AntiSkillInjector（⚡创新点：编译期自动注入安全约束）、NestedDataDetector。右侧虚线框 Diagnostic 收集诊断信息。输出 ValidatedSkillIR
5. **Phase 4 Backend**: 上方 EmitterRegistry，下方四个 Emitter 并列 — ClaudeEmitter（XML）、CodexEmitter（纯MD）、GeminiEmitter（MD+YAML）、KimiEmitter（完整MD）。旁边 RoutingManifestGenerator（⚡创新点：渐进式路由）
6. **Output**: 四个平台产物 + routing_manifest.yaml + manifest.json + signature.sha256

三个创新点用强调色标注：Anti-Skill Injection、AST Optimization、Progressive Routing。安全组件用另一种强调色标注。

---

## Fig 2: Compiler 定位与 m×n → m+n

**位置**: Section 1 Introduction | **尺寸**: 双栏宽 | **优先级**: 必须

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

## Fig 3: 编译管线数据流详图

**位置**: Section 3.1 | **尺寸**: 双栏宽 | **优先级**: 推荐

横向流水线，每个阶段展示输入数据结构 → 转换 → 输出数据结构：

- SKILL.md（左侧示例源文件片段）
- → RawAST（简化类图：source_path, frontmatter, body, source_hash）
- → SkillIR（简化类图：name, version, description, security_level, permissions, procedures, anti_skill_constraints, requires_yaml_optimization ⚡, nested_data_depth ⚡）
- → ValidatedSkillIR（简化类图：inner SkillIR + warnings Vec<Diagnostic>），旁边 Analyzer Chain 虚线框
- → 四个平台输出并列，各展示一段简短代码片段（Claude XML、Codex Markdown、Gemini MD+YAML、Kimi 完整MD）

---

## Fig 4: 渐进式路由机制对比图

**位置**: Section 3.4 或 Section 4 | **尺寸**: 单栏宽 | **优先级**: 推荐

左右对比：

- **左：Traditional Full Loading** — Agent 启动加载 15 个完整 SKILL.md，标注 ≈150K tokens，列出三个问题（注意力分散、高 API 成本、幻觉风险）
- **右：NSC Progressive Disclosure** — 启动仅加载 routing_manifest.yaml（≈750 tokens，标注 99.5% savings），用户请求后语义路由匹配，仅动态加载 1 个匹配技能（≈10K tokens）
- 底部对比条形图：Traditional 150K vs NSC 750+10K

---

## Fig 5: 格式敏感性 + AST 优化决策图

**位置**: Section 3.3 | **尺寸**: 双栏宽 | **优先级**: 可选

三部分组合：
- **上部**：嵌套数据格式准确率柱状图（YAML 51.9% > MD 48.2% > JSON 43.1% > XML 33.8%）
- **中部**：AST 优化决策树 — NestedDataDetector 计算 depth，depth<3 用 Markdown，depth≥3 用 YAML；同时展示四平台各自策略（Claude→XML, Codex→纯MD, Gemini→条件YAML, Kimi→完整MD）
- **下部**：四平台输出示例片段，各标注核心策略关键词

---

## Fig 6: Anti-Skill 注入机制示意图

**位置**: Section 3.2 | **尺寸**: 单栏宽 | **优先级**: 可选

横向流程：
- **左侧**：原始 Procedure 步骤文本（含 HTTP、BeautifulSoup、DROP、loop 等关键词）
- **中间**：四类 Anti-Pattern 规则框（HTTP/HTML Parse/Destructive DB/Loop），标注触发关键词和对应约束内容，用匹配状态标记
- **右侧**：增强后的 SkillIR，anti_skill_constraints 字段中列出 4 条自动注入的安全约束，标注 "compile-time auto-injection"

---

## 优先级总结

| Figure | 必须/推荐/可选 | 说明 |
|--------|---------------|------|
| Fig 1 | 必须 | 核心架构图 |
| Fig 2 | 必须 | 动机论述核心视觉 |
| Fig 3 | 推荐 | 数据流补充 Fig 1 |
| Fig 4 | 推荐 | 渐进式路由对比 |
| Fig 5 | 可选 | 格式敏感性+AST优化 |
| Fig 6 | 可选 | Anti-Skill 注入 |