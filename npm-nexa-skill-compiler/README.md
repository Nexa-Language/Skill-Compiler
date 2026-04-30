<div align="center">
  <img src="https://raw.githubusercontent.com/ouyangyipeng/Skill-Compiler/main/docs/img/nsc-logo.png" alt="NSC Logo" width="100" />
  <h1>Nexa Skill Compiler</h1>
  <p><b><i>Write Once, Run Anywhere for AI Agent Skills</i></b></p>
  <p>
    <img src="https://img.shields.io/badge/License-MIT-blue.svg?style=for-the-badge" alt="License"/>
    <img src="https://img.shields.io/badge/Version-v1.0-brightgreen.svg?style=for-the-badge" alt="Version"/>
    <img src="https://img.shields.io/badge/Rust-1.75%2B-orange.svg?style=for-the-badge" alt="Rust"/>
    <img src="https://img.shields.io/badge/Status-Stable-green.svg?style=for-the-badge" alt="Status"/>
  </p>
</div>

---

## Installation

```bash
# Install via npm
npm install -g nexa-skill-compiler

# Install the Rust binary (required)
cargo install nexa-skill-cli
```

## Quick Start

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

## What is NSC?

**Nexa Skill Compiler (NSC)** is an industrial-grade multi-target compiler that transforms unified `SKILL.md` specifications into platform-specific agent instructions.

### 🔍 Frontend: Parsing & Validation
- YAML Frontmatter Parser with type validation
- Permission Auditor for security analysis
- MCP Dependency Checker

### 🧠 Mid-end: IR & Optimization
- SkillIR - Platform-independent intermediate representation
- Anti-Skill Injection - Automatic defense against dangerous behaviors
- Security Level Analyzer with 4-tier model

### 🚀 Backend: Multi-Target Emission
- Claude Target - Claude Code compatible output
- Codex Target - OpenAI Codex/GPT format
- Gemini Target - Google Gemini system instructions

## Performance

Based on large-scale comparative experiments:

| Metric | Original Skills | Compiled Skills | Improvement |
|--------|-----------------|-----------------|-------------|
| Avg Duration | 45.2s | 37.6s | **16.9% faster** |
| Success Rate | 96% | 100% | +4% |

## Links

- 📚 [Documentation](https://github.com/ouyangyipeng/Skill-Compiler#readme)
- 🐛 [Issue Tracker](https://github.com/ouyangyipeng/Skill-Compiler/issues)
- 💬 [Discussions](https://github.com/ouyangyipeng/Skill-Compiler/discussions)

## License

MIT License - see [LICENSE](LICENSE) for details.