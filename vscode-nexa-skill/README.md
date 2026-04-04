<div align="center">
  <img src="images/icon.png" alt="NSC Logo" width="128" height="128">
  <h1>Nexa Skill Compiler - VS Code Extension</h1>
</div>

[![VS Code Marketplace](https://img.shields.io/badge/VS%20Code-Extension-blue?style=flat-square&logo=visual-studio-code)](https://marketplace.visualstudio.com/items?itemName=ouyangyipeng.nexa-skill-compiler)
[![License](https://img.shields.io/badge/license-MIT-green?style=flat-square)](LICENSE)

A comprehensive VS Code extension for **Nexa Skill Compiler (NSC)** - providing syntax highlighting, validation, compilation, and intelligent completions for SKILL.md files.

## Features

### 🎨 Syntax Highlighting

Full syntax highlighting for SKILL.md files including:
- YAML frontmatter with field-specific highlighting
- Markdown body content
- Constraint keywords (ALWAYS, NEVER, REQUIRE)
- Security levels and platform names

### ✅ Real-time Validation

Automatic diagnostics as you type:
- Missing required fields detection
- Invalid security level warnings
- YAML syntax validation
- Missing sections warnings

### 🚀 Compilation Commands

One-click compilation directly from VS Code:
- **NSC: Compile Skill** - Compile current skill file
- **NSC: Compile All Skills** - Compile all skills in workspace
- **NSC: Validate Skill** - Validate skill structure
- **NSC: Check Skill** - Run diagnostics check
- **NSC: Initialize New Skill** - Create new skill from template

### 📝 Intelligent Completions

Context-aware auto-completion:
- Frontmatter field names and values
- Section headers (Triggers, Procedures, Examples, etc.)
- Constraint keywords
- Security levels and platforms

### 💡 Hover Documentation

Detailed documentation on hover:
- Field descriptions and types
- Security level explanations
- Constraint keyword usage

### 👁️ Preview Panel

Live preview of skill metadata:
- Parsed frontmatter display
- Body preview
- Quick compile button

### 📊 Status Bar Integration

Real-time compilation status in the status bar.

## Installation

### From VS Code Marketplace

1. Open VS Code
2. Go to Extensions (Ctrl+Shift+X)
3. Search for "Nexa Skill Compiler"
4. Click Install

### From Source

```bash
cd vscode-nexa-skill
npm install
npm run compile
# Press F5 to launch extension development host
```

### Packaging for Distribution

```bash
npm install -g @vscode/vsce
vsce package
# This creates a .vsix file
```

## Usage

### Opening Skill Files

The extension automatically activates for:
- Files named `SKILL.md`
- Files with `.skill.md` or `.skill` extension

### Commands

| Command | Shortcut | Description |
|---------|----------|-------------|
| `NSC: Compile Skill` | - | Compile current skill to all platforms |
| `NSC: Compile All Skills` | - | Find and compile all SKILL.md files |
| `NSC: Validate Skill` | - | Validate skill structure |
| `NSC: Check Skill` | - | Run diagnostics |
| `NSC: Initialize New Skill` | - | Create new skill from template |
| `NSC: Show Preview` | - | Open preview panel |

### Configuration

Open VS Code settings and search for "NSC":

```json
{
  "nsc.compilerPath": "nsc",
  "nsc.defaultTarget": "all",
  "nsc.outputDirectory": "dist",
  "nsc.enableDiagnostics": true,
  "nsc.enableAutoCompile": false,
  "nsc.showStatusBar": true
}
```

| Setting | Description | Default |
|---------|-------------|---------|
| `compilerPath` | Path to NSC binary | `nsc` |
| `defaultTarget` | Default compilation target | `all` |
| `outputDirectory` | Output directory for compiled skills | `dist` |
| `enableDiagnostics` | Enable real-time diagnostics | `true` |
| `enableAutoCompile` | Auto-compile on save | `false` |
| `showStatusBar` | Show status bar item | `true` |

## Snippets

Type the following prefixes and press Tab to expand:

| Prefix | Description |
|--------|-------------|
| `skill` | Complete skill file template |
| `frontmatter` | YAML frontmatter only |
| `procedures` | Procedures section |
| `examples` | Examples section |
| `constraints` | Constraints section |
| `fallbacks` | Fallbacks section |
| `permission` | Permission declaration |
| `mcp` | MCP server declaration |
| `input_schema` | Input schema declaration |
| `always` | ALWAYS constraint |
| `never` | NEVER constraint |
| `require` | REQUIRE constraint |

## Example

Create a new skill file:

1. Create a folder named `my-skill`
2. Create `SKILL.md` inside
3. Type `skill` and press Tab
4. Fill in the template:

```markdown
---
name: my-skill
description: A useful skill for automation
version: 1.0.0
author: Your Name
compatibility:
  - claude
  - codex
  - gemini
security_level: medium
hitl_required: false
---

# My Skill

A skill that automates useful tasks.

## Triggers

- When the user asks to automate a task

## Procedures

### 1. Context Gathering

- Gather necessary context

### 2. Execution

- Execute the main task

### 3. Verification

- Verify the results

## Constraints

- ALWAYS follow best practices
- NEVER skip verification
- REQUIRE user confirmation for destructive actions
```

5. Right-click and select **NSC: Compile Skill**

## Requirements

- VS Code 1.85.0 or higher
- Nexa Skill Compiler (NSC) installed

### Installing NSC

```bash
# From crates.io
cargo install nexa-skill-compiler

# From source
git clone https://github.com/ouyangyipeng/Skill-Compiler
cd Skill-Compiler
cargo install --path nexa-skill-cli
```

## Troubleshooting

### "nsc command not found"

Make sure NSC is installed and in your PATH, or set the full path in settings:

```json
{
  "nsc.compilerPath": "/path/to/nsc"
}
```

### Diagnostics not showing

Ensure `enableDiagnostics` is set to `true` in settings.

### Compilation fails

Check the Output panel (View > Output > NSC) for error details.

## Contributing

Contributions are welcome! Please see the [main repository](https://github.com/ouyangyipeng/Skill-Compiler) for contribution guidelines.

## License

MIT License - see [LICENSE](LICENSE) for details.

## Related

- [Nexa Skill Compiler](https://github.com/ouyangyipeng/Skill-Compiler) - The main compiler project
- [User Guide](https://github.com/ouyangyipeng/Skill-Compiler/blob/main/docs/USER_GUIDE.md) - Complete documentation
- [Skill Specification](https://github.com/ouyangyipeng/Skill-Compiler/blob/main/docs/SPECIFICATION.md) - SKILL.md format