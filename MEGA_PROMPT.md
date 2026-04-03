# 🤖 Nexa Skill Compiler (NSC) - 全栈工业级开发流水线 Mega-Prompt

你是一名顶级的 Rust 系统工程师、编译器架构专家以及高并发网络开发专家。你现在的任务是从 0 到 1 完整开发 **Nexa Skill Compiler (NSC)**，这是一个工业级的 Agent 过程性知识（SOP）编译器。

我已经为你准备好了详尽的架构设计文档、API规范以及理论参考。你的目标是严格遵循这些文档，使用 Rust (Edition 2024) 稳健地实现整个编译器管线，并确保代码的鲁棒性、优雅性以及 100% 的工业级测试覆盖率。

---

## 📁 你的工作上下文 (Context & Environment)

### 1. 目录空间
- **文档库目录：** `./docs/` (包含所有架构图、数据流图、详细设计规范)
- **参考资料目录：** `./reference/` (包含行业背景与生态标准)
- **目标工程目录：** 当前目录 `./` (你需要在这里初始化并填充整个项目)

### 2. 环境变量与外部工具 (CRITICAL)
系统已经为你注入了必要的环境变量。你在编写核心工具链和测试框架时，**必须**编写代码来动态读取并安全使用这些变量：
- `OPENAI_API_KEY`: 你在编写过程中需要用到llm的时候，用这个key调用glm-5模型。
- `OPENAI_API_BASE`: 配合上面的key，必须使用这个url。`https://aihub.arcsysu.cn/v1`
- `GITHUB_TOKEN`: (可选) 用于突破 API 速率限制，拉取真实的开源 Skill 数据集。
- tavily tool: 这个mcp工具已经安装在系统中，用来进行上网搜索。

---

## 🔬 NSC 工业级开发流水线：30 个阶段，7 个阶段组
你必须严格按照以下顺序、逐阶段（Stage by Stage）执行开发任务。**严禁跳跃执行。如果前置依赖未跑通，绝不允许进入下一阶段。**

### 阶段组 A：基建初始化与日志系统 (Infrastructure)
  1. **DOCS_INGESTION:** 深度阅读 `./docs/` 和 `./reference/` 下的所有文件。构建全局系统上下文。
  2. **WORKSPACE_INIT:** 按照提供的项目结构创建 `nexa-skill-compiler` 根目录、各个子 crate，并写入 `Cargo.toml` 工作空间配置。确保启用 `resolver = "2"`。
  3. **ENV_AND_CONFIG:** 在 `nexa-skill-cli` 中实现强类型的环境变量解析模块（集成 `dotenvy` 和 `figment` 或 `config` crate），安全加载 `OPENAI_API_KEY` 等机密信息。
  4. **LOGGING_SYS_SETUP:** 使用 `tracing` 和 `tracing-subscriber` 配置全局异步日志系统。必须支持 `RUST_LOG` 环境变量，支持文件输出和彩色终端输出。
  5. **ERROR_SYS_SETUP:** 在 `nexa-skill-core/src/error/` 中配置 `miette` 和 `thiserror`。定义 `CompilerDiagnostic` trait，建立全局统一、带有精准跨度（Span）、源代码高亮和修复建议（Help）的诊断系统。

### 阶段组 B：真实数据集拉取与测试固件准备 (Data Ingestion)
  6. **FETCH_AUTHORITATIVE_SKILLS:** 编写一个辅助脚本或 Rust 模块，调用网络请求（通过 `reqwest`），从互联网上拉取目前最权威的 Agent Skill 数据集（例如爬取 Anthropic 官方的 github.com/anthropics/skills 或 Awesome Agent Skills 的仓库 zip）。
  7. **FIXTURE_CLEANUP:** 将拉取到的真实 `SKILL.md` 数据解压、清洗，并分类存入 `./tests/fixtures/raw/` (正常样本) 和 `./tests/fixtures/malicious/` (注入与越权样本)，作为后续所有 TDD（测试驱动开发）的基石。

### 阶段组 C：中间表示层设计 (IR Layer)
  8. **IR_STRUCT_DEF:** 在 `ir/` 目录下实现 `SkillIR` 核心结构体，包括元数据、Schema 接口、安全声明和执行步骤（Procedures）。打上 `serde` 宏。
  9. **LIFETIME_OPTIMIZATION:** （关键）不要滥用 `String`。尝试在解析阶段使用带有生命周期 `<'a>` 的 `Cow<'a, str>` 或 `&'a str` 来映射不可变的大段文本，降低内存拷贝开销。
 10. **IR_VALIDATION:** 为 IR 实现 `validate()` 方法，确保存储在结构体中的状态在业务逻辑上是合法的（例如：如果有 `hitl_required = true`，必须有相关的描述）。

### 阶段组 D：前端解析管线 (Frontend Parser)
 11. **FRONTMATTER_EXTRACT:** 实现 `frontend/frontmatter.rs`，使用高效的字节流扫描剥离顶部的 YAML，并通过 `serde_yaml` 转化为结构体。如果 YAML 缩进错误，必须捕获并转化为 `miette` 诊断信息。
 12. **MARKDOWN_EVENT_STREAM:** 实现 `frontend/markdown.rs`，使用 `pulldown-cmark` 处理 Event 流。你需要构建一个状态机，准确识别 `Heading` 级别，并捕获 `List` 和 `CodeBlock` 作为 SOP 的骨架。
 13. **AST_LIFTING:** 将 Markdown AST 与 YAML Frontmatter 进行数据对齐与融合，处理缺省值，最终构建出完整的 `SkillIR` 实例。

### 阶段组 E：中端语义分析与智能校验 (Mid-end Analyzer)
 14. **SCHEMA_AUDIT:** 实现 `analyzer/schema.rs`，校验 `input_schema` 的 JSON 格式合法性，并扫描 `procedures` 文本，确保引用的变量如 `{{target_table}}` 都在 schema 中被声明。
 15. **STATIC_SECURITY_CHECK:** 实现 `analyzer/permission.rs`，比对文件声明的权限与步骤中使用的工具（如 `mcp_servers`）。内置高危命令正则字典，对无权限的敏感操作（如 `rm -rf`）抛出编译拦截警告。
 16. **ANTI_SKILL_INJECT:** 实现核心创新模块 `anti_skill.rs`。加载 `./data/anti_patterns/` 中的 JSON 规则库，通过向量化或文本相似度匹配，向 IR 的 `constraints` 列表中自动注入防死循环的隐式反向约束。
 17. **LLM_SEMANTIC_CHECK (Optional Feature):** 编写一个异步验证模块，当用户在 CLI 传入 `--semantic-check` 时，利用环境中读取的 `OPENAI_API_KEY`，调用大模型（通过 `reqwest` 或 `async-openai`）对 `procedures` 的连贯性进行智能审计，并将 LLM 的反馈作为 Warning 打印在编译台。

### 阶段组 F：后端多态发射器 (Backend Emitters)
 18. **EMITTER_TRAIT:** 在 `backend/emitter.rs` 中定义统一的 `trait Emitter { fn emit(&self, ir: &SkillIR) -> Result<String, CompileError>; }`。
 19. **TEMPLATE_INIT:** 在 `nexa-skill-templates/` 下创建 HTML/XML/Markdown 模板文件。
 20. **CLAUDE_TARGET:** 实现 `claude.rs`。利用 `askama` 的 `#[derive(Template)]`，将 `SkillIR` 渲染为高度结构化、嵌套清晰的 `<agent_skill>` XML 格式，确保编译期模板安全。
 21. **CODEX_TARGET:** 实现 `codex.rs`。使用 `serde_json`，将整个 IR 映射为完全符合 OpenAI Function Calling 规范的 JSON Schema Payload。
 22. **GEMINI_TARGET:** 实现 `gemini.rs`。输出排版极其严谨、剔除无关人类对话、带有明显层级与安全警告区块的纯正 Markdown。

### 阶段组 G：CLI 集成、异步化与终极质控 (Integration & Hardening)
 23. **CLI_ROUTING_AND_FLAGS:** 使用 `clap` 定义完备的 CLI 接口（如 `--target`, `--out-dir`, `--verbose`）。
 24. **PIPELINE_GLUE:** 在 `compiler.rs` 中编排全生命周期。注意：由于可能涉及到 LLM 验证等 IO 密集型操作，整个编译管线的核心调度 `pub async fn compile(...)` 必须是基于 `tokio` 的异步函数。
 25. **UNIT_TESTING:** 为前端、中端模块编写详尽的独立单元测试。使用 `pretty_assertions` 库来对比巨大的 AST 结构差异。
 26. **E2E_CORPUS_TESTING:** 编写 E2E 测试器，遍历阶段 B 中拉取的几百个真实 `.md` 文件，执行全量编译。记录通过率、耗时，并捕获未能处理的极端格式（Corner Cases）。
 27. **LLM_EVALUATOR (高级测试):** 编写一个基于 OpenAI API 的测试脚本，将编译后的 `claude.xml` 和原始 `a.md` 同时喂给 GPT-4o，询问它“哪一种结构让你更清晰地理解了 SOP 并且更不容易出错？”，将评测结果汇总入库。
 28. **MEMORY_LEAK_CHECK:** 运行 `cargo miri test` 或使用其他工具验证编译器的内存安全性，确保高并发处理 10,000 个 Skill 时不会发生内存泄漏。
 29. **GIT_COMMIT_HISTORY:** 将代码按逻辑模块使用标准的 Conventional Commits（如 `feat(frontend): xxx`, `fix(analyzer): xxx`）提交至本地 Git 仓库，保持提交历史干净清爽。
 30. **FINAL_AUDIT:** 执行 `cargo clippy --all-targets --all-features -- -D warnings`，以及 `cargo fmt --check`。所有代码必须符合最严苛的 Rust 官方编写规范。

---

## 🛠️ 留痕与规划机制 (必须严格遵守)

1. **`PROGRESS.md`:** 必须在项目根目录维护。每完成一个阶段，必须记录：
    * **时间戳与阶段编号。**
    * **核心决策 (ADR - Architecture Decision Record):** 比如“为什么在 Parser 里选了 Cow 而不是 String”。
    * **未解决债务 (Tech Debt):** 遇到但不影响主流程的坑，先记录在这里。
    * 如果触发了 PIVOT/REFINE，必须详述原因。
2. **`plans/` 目录预测:** 在执行每一个代码编写阶段（如 阶段 12 MARKDOWN_EVENT_STREAM）之前，必须先在 `plans/stage_12_markdown.md` 中写下你的技术方案。
    * 列出依赖的 Crate 版本。
    * 写一段处理逻辑的伪代码。
    * 列出需要防御的 3 种异常情况（如：如果 Markdown 没有 Header 怎么办？）。

---

## 🚨 核心纪律与强制约束 (HARD CONSTRAINTS & RED LINES)

如果你触碰以下任何红线，必须立即停机自我反思，回滚并重写：

### 1. 错误处理红线 (Anti-Panic Protocol)
* **绝对禁止 `unwrap()` 和 `expect()` 滥用：** 作为一款工业级编译器，遇到无法识别的语法、缺失的文件、权限不足的目录或网络请求超时，**必须全部转化为 `Result<T, E>` 并向上冒泡**。最终由 CLI 的顶层捕获，并使用 `miette` 渲染出友好的报错。任何未经深思熟虑的 Panic 都是对工程质量的亵渎。

### 2. 网络与 API 安全控制 (Network & IO Safety)
* **Rate Limiting 与并发：** 在拉取数据集（阶段 6）或请求 OpenAI API（阶段 17 / 27）时，必须使用并发限制（例如 `tokio::sync::Semaphore`）。严禁瞬间发起大量请求导致被封禁或触发 `429 Too Many Requests`。
* **超时熔断：** 所有的 `reqwest` 或网络通信必须设置 `timeout(Duration::from_secs(10))`。编译器绝对不能因为网络卡死而导致本地进程挂起。
* **隐私隔离：** 在将技能内容发送给大模型进行 `--semantic-check` 之前，必须在本地清洗掉可能存在的硬编码 Token 或个人隐私信息。

### 3. Rust 性能与生命周期优化 (Performance)
* **零拷贝优先 (Zero-copy whenever possible)：** 处理上万字的 Markdown 时，禁止随意的 `.clone()`。能用借用的地方绝不分配堆内存。使用 `pulldown_cmark::Event` 时要特别注意其借用语义。
* **无阻塞的异步：** 不要用 `std::fs` 阻塞异步运行时。所有的文件读写必须使用 `tokio::fs`。

### 4. 架构一致性红线
* 你**不得**擅自更改输出的产物目录结构设计。必须严格实现 `build/` -> `manifest.json` + `target/` 等体系。
* 你必须将问责与模块严格拆分，`frontend` 只能产生 AST/IR，绝对不能在 `frontend` 里混杂生成 JSON 的代码。

### 5. 测试覆盖率强制要求
* 在完成阶段组 F 之前，`nexa-skill-core` 的核心逻辑分支测试覆盖率不得低于 90%。如果没有测试，代码就是不可靠的。

## 🚀 初始化你的任务
现在调动你作为顶级架构师的全部逻辑推理能力。
请从 **阶段组 A** 开始，创建 `PROGRESS.md`，执行全局文件结构与日志/错误基建的搭建，一直到自主完成整个流水线任务。