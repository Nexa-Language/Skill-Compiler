# Nexa Skill Compiler (NSC) - 开发进度追踪

> **本文档记录 NSC 项目的开发进度、核心决策 (ADR) 和技术债务**

---

## 📋 项目概览

- **项目名称**: Nexa Skill Compiler (NSC)
- **技术栈**: Rust (Edition 2024), pulldown-cmark, serde, clap, miette, askama, tokio
- **目标**: 构建工业级 Agent 过程性知识 (SOP) 编译器
- **开始时间**: 2026-04-03

---

## 🔄 阶段组进度

### 阶段组 A: 基建初始化与日志系统 (Infrastructure)

| 阶段 | 状态 | 完成时间 | 备注 |
|------|------|----------|------|
| 1. DOCS_INGESTION | ✅ 完成 | 2026-04-03 | 已深度阅读 docs/ 和 reference/ 下所有文件 |
| 2. WORKSPACE_INIT | 🔄 进行中 | - | 创建 Cargo.toml 工作空间配置 |
| 3. ENV_AND_CONFIG | ⏳ 待开始 | - | - |
| 4. LOGGING_SYS_SETUP | ⏳ 待开始 | - | - |
| 5. ERROR_SYS_SETUP | ⏳ 待开始 | - | - |

### 阶段组 B: 真实数据集拉取与测试固件准备 (Data Ingestion)

| 阶段 | 状态 | 完成时间 | 备注 |
|------|------|----------|------|
| 6. FETCH_AUTHORITATIVE_SKILLS | ⏳ 待开始 | - | - |
| 7. FIXTURE_CLEANUP | ⏳ 待开始 | - | - |

### 阶段组 C: 中间表示层设计 (IR Layer)

| 阶段 | 状态 | 完成时间 | 备注 |
|------|------|----------|------|
| 8. IR_STRUCT_DEF | ⏳ 待开始 | - | - |
| 9. LIFETIME_OPTIMIZATION | ⏳ 待开始 | - | - |
| 10. IR_VALIDATION | ⏳ 待开始 | - | - |

### 阶段组 D: 前端解析管线 (Frontend Parser)

| 阶段 | 状态 | 完成时间 | 备注 |
|------|------|----------|------|
| 11. FRONTMATTER_EXTRACT | ⏳ 待开始 | - | - |
| 12. MARKDOWN_EVENT_STREAM | ⏳ 待开始 | - | - |
| 13. AST_LIFTING | ⏳ 待开始 | - | - |

### 阶段组 E: 中端语义分析与智能校验 (Mid-end Analyzer)

| 阶段 | 状态 | 完成时间 | 备注 |
|------|------|----------|------|
| 14. SCHEMA_AUDIT | ⏳ 待开始 | - | - |
| 15. STATIC_SECURITY_CHECK | ⏳ 待开始 | - | - |
| 16. ANTI_SKILL_INJECT | ⏳ 待开始 | - | - |
| 17. LLM_SEMANTIC_CHECK | ⏳ 待开始 | - | - |

### 阶段组 F: 后端多态发射器 (Backend Emitters)

| 阶段 | 状态 | 完成时间 | 备注 |
|------|------|----------|------|
| 18. EMITTER_TRAIT | ⏳ 待开始 | - | - |
| 19. TEMPLATE_INIT | ⏳ 待开始 | - | - |
| 20. CLAUDE_TARGET | ⏳ 待开始 | - | - |
| 21. CODEX_TARGET | ⏳ 待开始 | - | - |
| 22. GEMINI_TARGET | ⏳ 待开始 | - | - |

### 阶段组 G: CLI 集成、异步化与终极质控 (Integration & Hardening)

| 阶段 | 状态 | 完成时间 | 备注 |
|------|------|----------|------|
| 23. CLI_ROUTING_AND_FLAGS | ⏳ 待开始 | - | - |
| 24. PIPELINE_GLUE | ⏳ 待开始 | - | - |
| 25. UNIT_TESTING | ⏳ 待开始 | - | - |
| 26. E2E_CORPUS_TESTING | ⏳ 待开始 | - | - |
| 27. LLM_EVALUATOR | ⏳ 待开始 | - | - |
| 28. MEMORY_LEAK_CHECK | ⏳ 待开始 | - | - |
| 29. GIT_COMMIT_HISTORY | ⏳ 待开始 | - | - |
| 30. FINAL_AUDIT | ⏳ 待开始 | - | - |

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

---

## ⚠️ 技术债务 (Tech Debt)

| ID | 描述 | 优先级 | 状态 | 计划解决时间 |
|----|------|--------|------|--------------|
| TD-001 | WASM 绑定暂不实现 | 低 | 暂缓 | Phase 2 或更晚 |
| TD-002 | LLM 语义检查作为可选 Feature | 中 | 待定 | 阶段 17 |
| TD-003 | 多语言支持暂不考虑 | 低 | 暂缓 | 未来版本 |

---

## 📊 统计信息

- **总阶段数**: 30
- **已完成**: 1
- **进行中**: 1
- **待开始**: 28
- **完成率**: 6.7%

---

## 🕐 更新日志

### 2026-04-03
- 完成 DOCS_INGESTION 阶段：深度阅读所有文档
- 开始 WORKSPACE_INIT 阶段：创建 Cargo.toml 工作空间配置
- 创建 PROGRESS.md 文件
- 记录 ADR-001 ~ ADR-004 核心决策