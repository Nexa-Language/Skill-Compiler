# Nexa Skill Compiler (NSC)

> **将人类可读的 SKILL.md 编译为 AI Agent 可执行的过程性知识库**

[![Rust](https://img.shields.io/badge/Rust-Edition%202024-orange.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Status](https://img.shields.io/badge/Status-Release%20Candidate-green.svg)]()

---

## 🎯 项目愿景

Nexa Skill Compiler (NSC) 是一个针对大语言模型 (LLM) 过程性知识（Procedural Knowledge）的**跨端编译器**。它并非一个单纯的 Markdown 格式化工具，而是一个完整的编译系统，能够：

- **接收**：符合人类阅读直觉与书写习惯的标准化 `SKILL.md`（包含 YAML Frontmatter 和 SOP 流程）
- **编译期 (AOT)**：完成依赖校验、权限审计、JSON Schema 映射和反向逻辑注入
- **生成**：针对特定底层模型（Claude、Codex/GPT、Gemini 等）具备最高指令遵循度的"方言（Dialects）"结构体系

### 核心价值主张

| 传统方式 | NSC 编译方式 |
|---------|-------------|
| 直接读取文本并塞入 Prompt | 解析 → 约束 → 优化 → 多态分发 |
| 松散的自然语言描述 | 强类型元数据 + 结构化 SOP |
| 单一格式输出 | 多平台原生方言适配 |
| 无安全审计 | 编译期权限检查 + Anti-Skill 注入 |

---

## 🚀 快速入门

### 安装

```bash
# 从源码构建（需要 Rust 1.75+）
git clone https://github.com/nexa-org/nexa-skill-compiler
cd nexa-skill-compiler
cargo build --release

# 安装到系统路径
cargo install --path .
```

### 基本用法

```bash
# 将单个 SKILL.md 编译为 Claude 原生的 XML 结构
nexa-skill build --claude database-migration.md

# 编译整个 skill 目录为 Codex 偏好的 JSON Schema
nexa-skill build --codex ./my-skills/web-scraper/

# 多目标编译，输出到指定目录
nexa-skill build --claude --gemini --out-dir ./dist ./skills/

# 验证 SKILL.md 格式但不生成输出
nexa-skill check ./skills/new-feature.md
```

### 编译产物结构

执行 `nexa-skill build --claude --codex database-migration.md` 后，生成的标准目录树：

```text
build/database-migration/
├── manifest.json            # 编译生成的通用元数据
├── target/                  # 针对不同平台的最终注入包
│   ├── claude.xml           # Claude XML 格式 Prompt
│   └── codex_schema.json    # OpenAI Function Calling Schema
├── assets/                  # 静态依赖（从源目录拷贝）
└── meta/
    └── signature.sha256     # 编译产物完整性哈希
```

---

## 📖 文档索引

| 文档 | 描述 |
|------|------|
| [ARCHITECTURE.md](ARCHITECTURE.md) | 系统架构总览、模块划分、数据流图 |
| [SPECIFICATION.md](SPECIFICATION.md) | SKILL.md 源文件规范定义 |
| [COMPILER_PIPELINE.md](COMPILER_PIPELINE.md) | 编译管线四阶段详细设计 |
| [IR_DESIGN.md](IR_DESIGN.md) | 中间表示（SkillIR）数据结构设计 |
| [BACKEND_ADAPTERS.md](BACKEND_ADAPTERS.md) | 后端适配器设计（Claude/Codex/Gemini） |
| [CLI_DESIGN.md](CLI_DESIGN.md) | CLI 交互设计、命令规范 |
| [ERROR_HANDLING.md](ERROR_HANDLING.md) | 错误处理与诊断系统设计 |
| [SECURITY_MODEL.md](SECURITY_MODEL.md) | 安全模型、权限审计、Anti-Skill 注入 |
| [TESTING_STRATEGY.md](TESTING_STRATEGY.md) | 测试策略、测试金字塔 |
| [DEVELOPMENT_GUIDE.md](DEVELOPMENT_GUIDE.md) | 开发指南、环境配置、贡献规范 |
| [API_REFERENCE.md](API_REFERENCE.md) | 公开 API 参考、核心 Trait 定义 |
| [ROADMAP.md](ROADMAP.md) | 项目路线图、里程碑规划 |

---

## 🔧 技术栈

| 组件 | 技术选型 | 选型理由 |
|------|----------|----------|
| **语言基础** | Rust (Edition 2024) | 内存安全、零拷贝解析、WASM 潜力 |
| **Markdown 解析** | `pulldown-cmark` | 基于事件流的 AST 解析，避免正则灾难 |
| **序列化枢纽** | `serde` + `serde_json` | 强类型序列化，属性宏简化代码 |
| **模板系统** | `askama` | 编译期静态检查，杜绝运行时变量丢失 |
| **错误报告** | `miette` | 精美终端报错，带行号和代码片段 |
| **CLI 框架** | `clap` | Rust 生态标准，极简高效 |

---

## 🎨 设计哲学

NSC 遵循 **"静态编译、动态执行、多态分发"** 的设计哲学：

1. **静态编译 (AOT)**：所有校验、优化、注入在编译期完成，运行时零开销
2. **动态执行**：生成的产物可被 Agent 按需加载，支持渐进式披露
3. **多态分发**：同一份源文件，针对不同平台生成最优格式

---

## 🤝 兼容性

NSC 生成的产物兼容以下 Agent 平台：

| 平台 | 底层模型 | 首选格式 | 支持状态 |
|------|----------|----------|----------|
| Claude Code | Claude 4.6 Opus | XML / 强标签化 MD | ✅ 完全支持 |
| Codex CLI | GPT-5.4 | YAML + JSON Schema | ✅ 完全支持 |
| Gemini CLI | Gemini 3.1 Pro | 结构化 Markdown + YAML | ✅ 完全支持 |
| Kimi CLI | K2.5 | 纯文本 / 巨型文档 | 🚧 实验性支持 |
| GitHub Copilot | GPT-4 | JSON Schema | ✅ 完全支持 |
| VS Code Agent | 多模型 | Markdown | ✅ 完全支持 |

---

## 📜 许可证

本项目采用 MIT 许可证。详见 [LICENSE](../LICENSE) 文件。

---

## 🙏 致谢

- [Agent Skills 官方规范](https://agentskills.io/) - SKILL.md 格式标准
- [pulldown-cmark](https://github.com/pulldown-cmark/pulldown-cmark) - Markdown 解析引擎
- [miette](https://github.com/zkat/miette) - 错误报告库
- Rust 社区的所有贡献者