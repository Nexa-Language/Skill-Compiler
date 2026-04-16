# Nexa Skill Compiler (NSC) - 开发进度追踪

> **本文档记录 NSC 项目的开发进度、核心决策 (ADR) 和技术债务**

---

## 📋 项目概览

- **项目名称**: Nexa Skill Compiler (NSC)
- **技术栈**: Rust (Edition 2024), pulldown-cmark, serde, clap, miette, askama, tokio
- **目标**: 构建工业级 Agent 过程性知识 (SOP) 编译器
- **开始时间**: 2026-04-03
- **当前状态**: 核心功能已完成，进入收尾与优化阶段
- **完成率**: ~97%

---

## 🔄 阶段组进度

### 阶段组 A: 基建初始化与日志系统 (Infrastructure) ✅

| 阶段 | 状态 | 完成时间 | 备注 |
|------|------|----------|------|
| 1. DOCS_INGESTION | ✅ 完成 | 2026-04-03 | 已深度阅读 docs/ 和 reference/ 下所有文件 |
| 2. WORKSPACE_INIT | ✅ 完成 | 2026-04-03 | Cargo.toml 工作空间配置完成 |
| 3. ENV_AND_CONFIG | ✅ 完成 | 2026-04-04 | 环境配置与 CLI config 模块完成 |
| 4. LOGGING_SYS_SETUP | ✅ 完成 | 2026-04-04 | 基于 tracing 的日志系统完成 |
| 5. ERROR_SYS_SETUP | ✅ 完成 | 2026-04-04 | miette + thiserror 诊断系统完成 |

### 阶段组 B: 真实数据集拉取与测试固件准备 (Data Ingestion) ✅

| 阶段 | 状态 | 完成时间 | 备注 |
|------|------|----------|------|
| 6. FETCH_AUTHORITATIVE_SKILLS | ✅ 完成 | 2026-04-06 | real_corpus 包含 30 个真实技能 |
| 7. FIXTURE_CLEANUP | ✅ 完成 | 2026-04-06 | 测试固件清理与规范化完成 |

### 阶段组 C: 中间表示层设计 (IR Layer) ✅

| 阶段 | 状态 | 完成时间 | 备注 |
|------|------|----------|------|
| 8. IR_STRUCT_DEF | ✅ 完成 | 2026-04-07 | SkillIR 含 Arc<str>, SecurityLevel, Approach, SkillMode, SectionInfo 等 |
| 9. LIFETIME_OPTIMIZATION | ✅ 完成 | 2026-04-07 | Arc<str> 零拷贝优化，IR 共享引用语义 |
| 10. IR_VALIDATION | ✅ 完成 | 2026-04-08 | IR 校验逻辑完成 |

### 阶段组 D: 前端解析管线 (Frontend Parser) ✅

| 阶段 | 状态 | 完成时间 | 备注 |
|------|------|----------|------|
| 11. FRONTMATTER_EXTRACT | ✅ 完成 | 2026-04-09 | frontmatter 解析含 name 自动规范化 (kebab-case) |
| 12. MARKDOWN_EVENT_STREAM | ✅ 完成 | 2026-04-16 | SectionKind 分类 + 6 种多策略 procedure 提取 |
| 13. AST_LIFTING | ✅ 完成 | 2026-04-10 | AST builder 完成 |

### 阶段组 E: 中端语义分析与智能校验 (Mid-end Analyzer) ✅

| 阶段 | 状态 | 完成时间 | 备注 |
|------|------|----------|------|
| 14. SCHEMA_AUDIT | ✅ 完成 | 2026-04-11 | SchemaValidator 28-rule 校验引擎 |
| 15. STATIC_SECURITY_CHECK | ✅ 完成 | 2026-04-12 | MCPDependencyChecker, PermissionAuditor, SecurityLevelValidator |
| 16. ANTI_SKILL_INJECT | ✅ 完成 | 2026-04-13 | AntiSkillInjector + NestedDataDetector |
| 17. LLM_SEMANTIC_CHECK | ⏳ 待开始 | - | 可选 Feature，见 TD-002 |

### 阶段组 F: 后端多态发射器 (Backend Emitters) ✅

| 阶段 | 状态 | 完成时间 | 备注 |
|------|------|----------|------|
| 18. EMITTER_TRAIT | ✅ 完成 | 2026-04-10 | Emitter trait + EmitterRegistry 完成 |
| 19. TEMPLATE_INIT | ✅ 完成 | 2026-04-10 | askama 模板基础设施完成 |
| 20. CLAUDE_TARGET | ✅ 完成 | 2026-04-14 | claude_xml.j2 Askama 模板发射 |
| 21. CODEX_TARGET | ✅ 完成 | 2026-04-14 | codex_md.j2 Askama 模板发射 |
| 22. GEMINI_TARGET | ✅ 完成 | 2026-04-14 | gemini_md.j2 Askama 模板发射 |
| 22b. KIMI_TARGET | ✅ 完成 | 2026-04-14 | kimi_md.j2 Askama 模板发射 |
| 22c. APPROACHES_SUPPORT | ✅ 完成 | 2026-04-16 | Approach + key_operations 模板渲染支持 |
| 22d. FULL_FIELD_COVERAGE | ✅ 完成 | 2026-04-16 | Codex/Gemini/Kimi 模板字段覆盖率达到 Claude XML 水平 |

### 阶段组 G: CLI 集成、异步化与终极质控 (CLI & Integration) 🔄

| 阶段 | 状态 | 完成时间 | 备注 |
|------|------|----------|------|
| 23. CLI_ROUTING_AND_FLAGS | ✅ 完成 | 2026-04-14 | nsc CLI: build/check/validate/clean/init/list |
| 24. PIPELINE_GLUE | ✅ 完成 | 2026-04-14 | 编译管线全链路贯通 |
| 25. UNIT_TESTING | ✅ 完成 | 2026-04-16 | 206+ 单元测试通过 |
| 26. E2E_CORPUS_TESTING | ✅ 完成 | 2026-04-16 | real corpus 编译 32/34 (2 被 security 阻断) |
| 27. LLM_EVALUATOR | ✅ 完成 | 2026-04-16 | 格式敏感性实验完成 |
| 28. MEMORY_LEAK_CHECK | ⏳ 待开始 | - | 低优先级 |
| 29. GIT_COMMIT_HISTORY | ✅ 完成 | 2026-04-03~16 | 全过程持续 commit |
| 30. FINAL_AUDIT | 🔄 进行中 | - | RE audit 已完成，最终审计进行中 |

---

## 🏆 关键成就 (Key Achievements)

1. **206+ 单元测试通过** — 覆盖 Frontend/IR/Analyzer/Backend/CLI 全模块
2. **Real corpus 编译成功** — 30 个技能编译通过 (2 个被 security validation 阻断，属预期行为)
3. **多策略 procedure 提取** — 6 种提取策略:
   - heading (标题分段)
   - list (列表步骤)
   - phase (阶段划分)
   - mode-selector (模式选择器)
   - reference (引用指令)
   - body-segmentation (正文分段)
4. **SectionKind 分类系统** — 对真实技能文档中的异构 section 进行精确分类
5. **SkillMode 枚举** — 区分四种执行策略模式:
   - Sequential (顺序执行)
   - Alternative (分支选择)
   - Toolkit (工具集)
   - Guideline (指导方针)
6. **Approach 结构体** — 为 mode-selector 类技能提供多路径描述与 key_operations 支持
7. **Name 自动规范化** — frontmatter 中 name 字段自动转为 kebab-case
8. **格式敏感性实验验证** — 编译格式 +0.039 整体提升 (0.851 vs 0.812)
9. **Bug fix: `matches!` 宏** — permission auditor 中的 matches! 宏误用已修复
10. **全平台字段覆盖统一** — Codex/Gemini/Kimi 模板补齐 pre_conditions, post_conditions, permissions, examples, fallbacks, extra_sections 等，与 Claude XML 达到同等覆盖率

---

## 📝 核心决策记录 (ADR)

### ADR-001: 项目架构选型
- **日期**: 2026-04-03
- **决策**: 采用四阶段编译管线架构 (Frontend → IR → Analyzer → Backend)
- **理由**: 
  1. 经典编译器架构，职责分离清晰
  2. 便于独立测试和调试各阶段
  3. 支持多目标平台并行发射
- **影响**: 所有模块需严格遵循阶段边界，禁止跨阶段直接调用

### ADR-002: 零拷贝优化策略
- **日期**: 2026-04-03
- **决策**: 在解析阶段使用 `Cow<'a, str>` 和 `&'a str` 替代 `String`
- **理由**:
  1. Markdown 文件可能很大（上万字），减少内存拷贝开销
  2. pulldown-cmark 的 Event 流本身就是借用语义
  3. 后端发射阶段再转换为 owned String
- **影响**: IR 结构需使用 `Arc<str>` 支持共享引用

### ADR-003: 错误处理架构
- **日期**: 2026-04-03
- **决策**: 使用 `miette` + `thiserror` 构建诊断系统
- **理由**:
  1. miette 提供精美的终端错误报告（带行号、代码片段）
  2. thiserror 简化错误类型定义
  3. 符合 Rust 生态最佳实践
- **影响**: 所有错误必须实现 `miette::Diagnostic` trait

### ADR-004: 模板引擎选型
- **日期**: 2026-04-03
- **决策**: 使用 `askama` 作为模板引擎
- **理由**:
  1. 编译期静态检查，模板错误在编译时发现
  2. 类型安全，杜绝运行时模板变量丢失
  3. 高性能，零运行时开销
- **影响**: 模板文件需放在 nexa-skill-templates/ 下

### ADR-005: 多策略 procedure 提取
- **日期**: 2026-04-16
- **决策**: 采用 6 种提取策略 (heading, list, phase, mode-selector, reference, body-segmentation) 对 Markdown 正文进行 procedure 提取
- **理由**:
  1. 真实技能文档格式高度异构，单一提取策略覆盖率不足
  2. 不同策略针对不同文档结构模式，组合使用覆盖率可达 >90%
  3. 策略选择基于 SectionKind 分类自动路由
- **影响**: ProcedureIR 变为策略聚合结果，需 StrategyResolver 协调多策略输出

### ADR-006: SectionKind 分类系统
- **日期**: 2026-04-16
- **决策**: 引入 SectionKind 对 Markdown section 语义进行分类 (Procedure, Description, Examples, Constraints, Prerequisites, etc.)
- **理由**:
  1. 真实技能文档的 section 标题命名不规范，无法直接映射到 IR
  2. 分类后可为 procedure 提取策略提供路由依据
  3. 对 LLM 而言，结构化的 section 语义比原始标题更有价值
- **影响**: Frontend markdown 模块需 SectionClassifier，IR 增加 SectionInfo 字段

### ADR-007: SkillMode 枚举设计
- **日期**: 2026-04-16
- **决策**: 引入 SkillMode 枚举 (Sequential/Alternative/Toolkit/Guideline) 区分技能执行策略
- **理由**:
  1. 部分技能为"多路径选择"而非"线性执行"，需要 Alternative 模式
  2. Toolkit 类技能为工具集合而非过程，需要独立模式
  3. Guideline 类技能为最佳实践建议，不包含具体步骤
- **影响**: SkillIR 增加 SkillMode + Approach 字段，Backend 模板需适配多模式渲染

### ADR-008: Warning 诊断保留策略
- **日期**: 2026-04-16
- **决策**: `ValidatedSkillIR` 携带 `Vec<Diagnostic>` 保留非阻断 warnings，而非在 Ok 路径静默丢弃
- **理由**:
  1. Analyzer 产生的 warnings 对编译正确性有参考价值（如缺少 procedures、empty conditions）
  2. `check_file()` Ok 路径返回空 vec 是语义错误——用户依赖 check 结果判断技能质量
  3. `build` 命令通过 `tracing::warn!` log 输出 warnings，保持编译继续但不静默
- **影响**: `ValidatedSkillIR(SkillIR)` → `ValidatedSkillIR(SkillIR, Vec<Diagnostic>)`，所有调用点需传入 warnings vec

---

## ⚠️ 技术债务 (Tech Debt)

| ID | 描述 | 优先级 | 状态 | 计划解决时间 |
|----|------|--------|------|--------------|
| TD-001 | WASM 绑定暂不实现 | 低 | 暂缓 | Phase 2 或更晚 |
| TD-002 | LLM 语义检查作为可选 Feature | 中 | 待定 | 未来版本 (可选功能) |
| TD-003 | 多语言支持暂不考虑 | 低 | 暂缓 | 未来版本 |
| TD-004 | Codex/Gemini/Kimi 模板丰富度不足 | 低 | ✅ 已解决 | C2c 已补齐全部缺失字段，4 平台覆盖率统一 |

---

## 📊 统计信息

- **总阶段数**: 30+ (含新增子阶段)
- **已完成**: ~28
- **进行中**: 0
- **待开始**: 2 (LLM_SEMANTIC_CHECK, MEMORY_LEAK_CHECK)
- **完成率**: ~97%
- **单元测试**: 206+ passing
- **Real corpus**: 32/34 编译成功

---

## 🕐 更新日志

### 2026-04-03
- 完成 DOCS_INGESTION 阶段：深度阅读所有文档
- 完成 WORKSPACE_INIT 阶段：创建 Cargo.toml 工作空间配置
- 创建 PROGRESS.md 文件
- 记录 ADR-001 ~ ADR-004 核心决策

### 2026-04-04
- 完成 ENV_AND_CONFIG 阶段：CLI config 模块
- 完成 LOGGING_SYS_SETUP 阶段：基于 tracing 的日志系统
- 完成 ERROR_SYS_SETUP 阶段：miette + thiserror 诊断系统

### 2026-04-06
- 完成 FETCH_AUTHORITATIVE_SKILLS 阶段：拉取 30 个真实技能
- 完成 FIXTURE_CLEANUP 阶段：测试固件清理与规范化

### 2026-04-07
- 完成 IR_STRUCT_DEF 阶段：SkillIR 含 Arc<str>, SecurityLevel, SectionInfo 等
- 完成 LIFETIME_OPTIMIZATION 阶段：Arc<str> 零拷贝优化

### 2026-04-08
- 完成 IR_VALIDATION 阶段：IR 校验逻辑

### 2026-04-09
- 完成 FRONTMATTER_EXTRACT 阶段：含 name 自动规范化 (kebab-case)

### 2026-04-10
- 完成 AST_LIFTING 阶段：AST builder
- 完成 EMITTER_TRAIT 阶段：Emitter trait + EmitterRegistry
- 完成 TEMPLATE_INIT 阶段：askama 模板基础设施

### 2026-04-11
- 完成 SCHEMA_AUDIT 阶段：SchemaValidator 28-rule 校验引擎

### 2026-04-12
- 完成 STATIC_SECURITY_CHECK 阶段：MCPDependencyChecker, PermissionAuditor, SecurityLevelValidator

### 2026-04-13
- 完成 ANTI_SKILL_INJECT 阶段：AntiSkillInjector + NestedDataDetector

### 2026-04-14
- 完成 CLAUDE_TARGET, CODEX_TARGET, GEMINI_TARGET, KIMI_TARGET 四个后端发射器
- 完成 CLI_ROUTING_AND_FLAGS：nsc CLI build/check/validate/clean/init/list
- 完成 PIPELINE_GLUE：编译管线全链路贯通

### 2026-04-15
- 多策略 procedure 提取实现：6 种策略 (heading, list, phase, mode-selector, reference, body-segmentation)
- SectionKind 分类系统设计与实现
- SkillMode 枚举设计与实现 (Sequential/Alternative/Toolkit/Guideline)
- Approach 结构体设计与实现 (mode-selector 类技能多路径描述)
- 单元测试大规模补充与修复
- Bug fix: permission auditor 中 matches! 宏误用修复

### 2026-04-16
- MARKDOWN_EVENT_STREAM 阶段完成：SectionKind + 多策略 procedure 提取
- APPROACHES_SUPPORT 完成：Backend 模板渲染 Approach + key_operations
- 单元测试达到 206+ passing
- Real corpus 编译测试：32/34 成功 (2 被 security validation 阻断)
- 格式敏感性实验完成：编译格式 +0.039 整体提升 (0.851 vs 0.812)
- 记录 ADR-005 (多策略 procedure 提取)、ADR-006 (SectionKind 分类)、ADR-007 (SkillMode 枚举)
- 新增 TD-004 (Codex/Gemini/Kimi 模板丰富度)
- RE audit 完成
- C2c 完成：Codex/Gemini/Kimi 模板全字段补齐 (pre_conditions, post_conditions, permissions, examples, fallbacks, extra_sections)，4 平台覆盖率统一
- TD-004 已解决：模板丰富度不足问题消除
- Audit-3 Critical 修复完成：
  - N1+N2: BACKEND_ADAPTERS.md Codex 双负载→纯Markdown，删除 ghost methods (supports_dual_payload/emit_schema_payload)
  - N3: ValidatedSkillIR 携带 Vec<Diagnostic> 保留 warnings；check_file() Ok 路径返回 warnings；CompileOutput 添加 warnings 字段
  - E1b: CORRECTED_EXPERIMENT_REPORT.md 标注 SUPERSEDED by v3.0
  - Docs misleading claim: 两份文档实现状态声明从 "已全部实现" → "大部分已实现，部分简化重构"
  - emitter.rs 注释修正 (dual-payload → actual)
  - 13 处 ValidatedSkillIR::new(ir) → ::new(ir, vec![]) 调用点更新
- 新增 ADR-008: Warning 诊断保留策略