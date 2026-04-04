# Nexa Skill Compiler (NSC)

<div align="center">

[![Crates.io](https://img.shields.io/crates/v/nexa-skill-compiler?style=flat-square&logo=rust)](https://crates.io/crates/nexa-skill-compiler)
[![License](https://img.shields.io/badge/license-MIT-blue?style=flat-square)](LICENSE)
[![Build Status](https://img.shields.io/github/actions/workflow/status/ouyangyipeng/Skill-Compiler/ci.yml?branch=main&style=flat-square&logo=github)](https://github.com/ouyangyipeng/Skill-Compiler/actions)
[![Docs](https://img.shields.io/badge/docs-latest-green?style=flat-square&logo=gitbook)](docs/)
[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange?style=flat-square&logo=rust)](https://www.rust-lang.org/)

**A Multi-Target Skill Compiler for AI Agent Platforms**

[English](#overview) · [中文文档](docs/USER_GUIDE.md) · [Documentation](docs/) · [API Reference](docs/API_REFERENCE.md)

</div>

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