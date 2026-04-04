<div align="center">
  <img src="docs/img/nsc-logo.png" alt="NSC Logo" width="100" />
  <h1>Nexa Skill Compiler</h1>
  <p><b><i>Write Once, Run Anywhere for AI Agent Skills</i></b></p>
  <p>
    <img src="https://img.shields.io/badge/License-MIT-blue.svg?style=for-the-badge" alt="License"/>
    <img src="https://img.shields.io/badge/Version-v1.0-brightgreen.svg?style=for-the-badge" alt="Version"/>
    <img src="https://img.shields.io/badge/Rust-1.75%2B-orange.svg?style=for-the-badge" alt="Rust"/>
    <img src="https://img.shields.io/badge/Status-Stable-green.svg?style=for-the-badge" alt="Status"/>
  </p>
  
  **中文版** | **[English](#overview)**
  
  📚 **文档**: [中文](docs/USER_GUIDE.md) | [API Reference](docs/API_REFERENCE.md)
</div>

---

## 📦 Installation

```bash
# Via npm (recommended for Node.js users)
npm install -g nexa-skill-compiler

# Via cargo (recommended for Rust users)
cargo install nexa-skill-cli

# From source
git clone https://github.com/ouyangyipeng/Skill-Compiler.git
cd Skill-Compiler
cargo install --path nexa-skill-cli
```

### VS Code Extension

Search "Nexa Skill Compiler" in VS Code Marketplace, or install manually:

```bash
code --install-extension nexa-skill-compiler-1.0.0.vsix
```

---

## 🚀 Quick Start

```bash
# Compile a skill for all platforms
nsc build skill.md

# Compile for specific target
nsc build skill.md --target claude

# Validate a skill file
nsc validate skill.md

# Initialize a new skill from template
nsc init my-skill
```

---

## ⚡ What is NSC?

**Nexa Skill Compiler (NSC)** 是一个工业级的多目标编译器，将统一的 `SKILL.md` 规范转换为平台特定的 AI Agent 指令。它实现了 AI Agent 技能的 **"一次编写，到处运行"**——支持 Claude Code、OpenAI Codex/GPT 和 Google Gemini 等多个平台。

---

## 🔥 Key Features

### 🔍 Frontend: Parsing & Validation
前端解析与静态分析：
- **YAML Frontmatter Parser** - 高性能事件流解析
- **Type Validation** - 字段类型检查与必填项验证
- **Permission Auditor** - 权限静态分析与安全审计
- **MCP Dependency Checker** - 依赖关系分析与验证

### 🧠 Mid-end: IR & Optimization
中端优化与安全增强：
- **SkillIR** - 统一中间表示，平台无关的抽象
- **Anti-Skill Injection** - 反向模式注入，自动防御危险行为
- **Security Level Analyzer** - 四级安全模型验证
- **HITL Triggers** - 高风险操作自动触发人机交互确认

### 🚀 Backend: Multi-Target Emission
后端多平台代码生成：
- **Claude Target** - 生成 Claude Code 兼容的 SKILL.md
- **Codex Target** - 生成 OpenAI Codex/GPT 格式
- **Gemini Target** - 生成 Google Gemini 系统指令
- **Parallel Emission** - 并行多目标生成，提升编译效率

### ⚡ High Performance
实验验证的执行效率提升：
- **16.9% faster** - 编译后的技能执行速度提升（大规模对比实验验证）
- **Rust Native** - 高性能原生编译器实现
- **Zero-Copy Parsing** - 零拷贝解析，内存高效

---

## 📈 Performance Highlights

基于 25 个真实任务的大规模对比实验结果：

| Metric | Original Skills | Compiled Skills | Improvement |
|--------|-----------------|-----------------|-------------|
| Avg Duration | 45.2s | 37.6s | **16.9% faster** |
| Success Rate | 96% | 100% | +4% |
| Quality Score | 0.92 | 0.94 | +2.2% |

详见 [Large-Scale Experiment Report](experiments/LARGE_SCALE_EXPERIMENT_REPORT.md)

---

## Overview

**Nexa Skill Compiler (NSC)** is an industrial-grade compiler that transforms unified `SKILL.md` specifications into platform-specific agent instructions. It enables **Write Once, Run Anywhere** for AI agent skills across Claude Code, OpenAI Codex/GPT, and Google Gemini.

### Why NSC?

| Problem | Solution |
|---------|----------|
| Skills are platform-specific | Unified `SKILL.md` specification |
| Manual adaptation is error-prone | Automated compilation pipeline |
| No semantic validation | Built-in analyzer with 100+ checks |
| Security risks in skills | Permission auditor & Anti-Skill injection |

### Key Features

- 🚀 **Multi-Target Compilation** - Single source to Claude, Codex, Gemini
- 🔒 **Security-First Design** - Permission auditing, HITL triggers, Anti-Skill patterns
- 📊 **Semantic Validation** - 100+ validation rules with actionable diagnostics
- ⚡ **High Performance** - 16.9% faster execution with compiled skills (validated by experiments)
- 🛠️ **Developer Experience** - Beautiful CLI with miette error reporting
- 📦 **Extensible Architecture** - Plugin-based Analyzer and Emitter system

---

## Quick Start

### Installation

```bash
# From crates.io (recommended)
cargo install nexa-skill-compiler

# From source
git clone https://github.com/ouyangyipeng/Skill-Compiler
cd Skill-Compiler
cargo install --path nexa-skill-cli
```

### Basic Usage

```bash
# Compile a skill for all platforms
nsc build skill.md --target all --output dist/

# Compile for specific platform
nsc build skill.md --target claude --output dist/

# Validate skill specification
nsc check skill.md

# Initialize a new skill project
nsc init my-skill --author "Your Name"
```

### Example: Database Migration Skill

```markdown
---
name: database-migration
description: PostgreSQL schema migration with safety guarantees
version: 1.0.0
security_level: critical
permissions:
  - kind: database
    scope: "postgresql://localhost:5432/*"
mcp_servers:
  - postgres-admin
hitl_required: true
---

# PostgreSQL Schema Migration

## Procedures

### 1. Context Gathering
- Analyze current schema state
- Identify affected tables and constraints
- Estimate migration impact

### 2. Execution
- Generate migration SQL with rollback
- Execute in transaction block
- Verify schema integrity

### 3. Fallbacks
- On failure: rollback transaction
- Log error details for debugging
- Notify administrator for manual intervention

## Examples

### Adding a Column
```sql
ALTER TABLE users ADD COLUMN last_login TIMESTAMP;
```

## Constraints
- NEVER execute without backup
- ALWAYS use transaction blocks
- REQUIRE human approval for production
```

Compile it:

```bash
nsc build database-migration.md --target claude codex gemini
```

Output structure:

```
dist/
├── claude/
│   └── database-migration/
│       ├── SKILL.md
│       └── manifest.json
├── codex/
│   └── database-migration/
│       ├── AGENTS.md
│       └── manifest.json
├── gemini/
│   └── database-migration/
│       ├── SYSTEM.md
│       └── manifest.json
└── manifest.json
```

---

## Architecture

NSC follows a classic compiler architecture with four phases:

```
┌─────────────────────────────────────────────────────────────────┐
│                    Nexa Skill Compiler Pipeline                  │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌──────────┐    ┌──────────┐    ┌──────────┐    ┌──────────┐  │
│  │ Frontend │───▶│ IR Build │───▶│ Analyzer │───▶│ Backend  │  │
│  │          │    │          │    │          │    │          │  │
│  │ • YAML   │    │ • SkillIR│    │ • Schema │    │ • Claude │  │
│  │ • Markdown│    │ • Valid │    │ • MCP    │    │ • Codex  │  │
│  │ • AST    │    │          │    │ • Perm   │    │ • Gemini │  │
│  └──────────┘    └──────────┘    │ • Anti   │    └──────────┘  │
│                                   └──────────┘                  │
│                                                                 │
│  Input: SKILL.md  ──────────────────────────────▶  Output:     │
│                                                    Platform     │
│                                                    Skills       │
└─────────────────────────────────────────────────────────────────┘
```

### Module Structure

| Crate | Purpose |
|-------|---------|
| [`nexa-skill-core`](nexa-skill-core/) | Compiler pipeline, IR, Analyzer, Backend |
| [`nexa-skill-cli`](nexa-skill-cli/) | Command-line interface with beautiful UX |
| [`nexa-skill-templates`](nexa-skill-templates/) | Askama-based template engine |

---

## Performance

Large-scale comparative experiments demonstrate **16.9% execution speed improvement** with compiled skills:

| Metric | Original Skill | Compiled Skill | Improvement |
|--------|---------------|----------------|-------------|
| Mean Execution Time | 45.2s | 37.6s | **16.9% faster** |
| Success Rate | 92% | 96% | +4% |
| Error Recovery | 68% | 85% | +17% |

*Based on 25 tasks across document processing, code analysis, and data operations. See [Experiment Report](experiments/LARGE_SCALE_EXPERIMENT_REPORT.md).*

---

## Documentation

| Document | Description |
|----------|-------------|
| [User Guide](docs/USER_GUIDE.md) | Comprehensive user documentation |
| [Specification](docs/SPECIFICATION.md) | SKILL.md format specification |
| [Architecture](docs/ARCHITECTURE.md) | System architecture overview |
| [API Reference](docs/API_REFERENCE.md) | Compiler API documentation |
| [Development Guide](docs/DEVELOPMENT_GUIDE.md) | Contributing guidelines |
| [Security Model](docs/SECURITY_MODEL.md) | Security architecture |

---

## Roadmap

### v1.0 (Current)
- ✅ Multi-target compilation (Claude, Codex, Gemini)
- ✅ Semantic validation with 100+ rules
- ✅ Permission auditing & security levels
- ✅ Anti-Skill pattern injection
- ✅ Beautiful CLI with miette diagnostics

### v1.1 (Planned)
- 🔲 VS Code extension
- 🔲 Web-based skill editor
- 🔲 Skill package registry
- 🔲 Auto-update mechanism

### v2.0 (Future)
- 🔲 Custom target platform SDK
- 🔲 Skill dependency management
- 🔲 WASM-based sandbox execution
- 🔲 Distributed skill orchestration

---

## Contributing

We welcome contributions! Please see [Development Guide](docs/DEVELOPMENT_GUIDE.md) for:

- Code style guidelines
- Commit message conventions
- PR review process
- Testing requirements

### Quick Contribution Guide

```bash
# 1. Fork and clone
git clone https://github.com/YOUR_USERNAME/Skill-Compiler

# 2. Create feature branch
git checkout -b feat/my-feature

# 3. Make changes and test
cargo test
cargo clippy

# 4. Commit with conventional format
git commit -m ":sparkles: feat: add new feature"

# 5. Push and create PR
git push origin feat/my-feature
```

---

## License

MIT License - see [LICENSE](LICENSE) for details.

---

## Acknowledgments

NSC is inspired by:
- [Anthropic's Skill System](https://docs.anthropic.com) - Skill specification design
- [awesome-claude-skills](https://github.com/alexanderatallah/awesome-claude-skills) - Community skill corpus
- [Composio](https://composio.dev) - SaaS automation patterns

---

## Citation

If you use NSC in your research, please cite:

```bibtex
@software{nexa_skill_compiler_2026,
  author = {Ouyang, Yipeng},
  title = {Nexa Skill Compiler: A Multi-Target Skill Compiler for AI Agent Platforms},
  year = {2026},
  url = {https://github.com/ouyangyipeng/Skill-Compiler},
  note = {Version 1.0}
}
```

---

<div align="center">

**Made with ❤️ by the Nexa Team**

[GitHub](https://github.com/ouyangyipeng/Skill-Compiler) · [Issues](https://github.com/ouyangyipeng/Skill-Compiler/issues) · [Discussions](https://github.com/ouyangyipeng/Skill-Compiler/discussions)

</div>