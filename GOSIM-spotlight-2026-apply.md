# GOSIM Spotlight 2026 Application

## Project Information

### Project Title *

**Nexa Skill Compiler (NSC)**

---

### Project Description *

Nexa Skill Compiler (NSC) is an industrial-grade multi-target compiler that transforms unified SKILL.md specifications into platform-specific AI agent instructions. It enables "Write Once, Run Anywhere" for AI agent skills across Claude Code, OpenAI Codex/GPT, and Google Gemini platforms.

The project addresses a critical challenge in the AI agent ecosystem: platform fragmentation. Different agent platforms use incompatible skill formats, forcing developers to maintain multiple versions. NSC solves this through a classic three-phase compilation pipeline: (1) Frontend parsing with YAML frontmatter and Markdown processing, (2) Intermediate Representation (SkillIR) with semantic validation and security analysis, and (3) Multi-target backend emission for different platforms.

Key innovations include: 100+ semantic validation rules for skill quality assurance, a four-level security model with permission auditing, Anti-Skill pattern injection for security hardening, and Human-in-the-Loop (HITL) triggers for high-risk operations. Large-scale experiments demonstrate 16.9% execution speed improvement with compiled skills compared to original skills.

---

### Work Type *

**Software / Developer Tool**

---

### Physical component? *

**No** - This is a pure software project with no physical components.

---

### How does AI participate in the creative process? *

This project is a tool FOR AI agents, developed through human-AI collaboration:

**Human Creator Contribution:**
- System architecture design based on compiler theory (inspired by LLVM's three-phase design)
- Core algorithm design for parsing, IR construction, and code emission
- Security model design including permission auditing and Anti-Skill patterns
- Rust implementation with focus on performance and type safety
- Experimental design and evaluation methodology

**AI (Claude Code) Contribution:**
- Code implementation assistance during development
- Documentation generation and refinement
- Test case generation and validation
- Code review and optimization suggestions
- Academic paper drafting support

**Collaboration Model:**
The human architect designed the overall system architecture and defined the skill specification format. AI assistants helped implement specific modules, generate boilerplate code, and refine documentation. The iterative development process involved human review of AI-generated code, ensuring correctness and alignment with the architectural vision. This represents a new paradigm of "AI-assisted software engineering" where humans focus on high-level design decisions while AI handles implementation details.

---

### Type of submission *

**Open Source Software Project**

---

### Submission Link *

**https://github.com/ouyangyipeng/Skill-Compiler**

---

## Exhibition & Logistics

### Available for on-site exhibition? *

**Yes**

---

### Willing to come to Paris? *

**Yes**

---

### Require a visa invitation letter?

**Yes**, I would need a visa invitation letter to travel to Paris.

---

### Preferred exhibition format

**Interactive booth / live demo**

---

### Equipment or support required

- [x] Standard screen / monitor
- [x] Power supply
- [x] Internet connection
- [x] Computer

---

### Special installation or technical requirements

The demo requires:
- A computer with Rust toolchain installed (or we can use the npm package)
- Internet connection for downloading dependencies and demonstrating cross-platform compilation
- Monitor for displaying the CLI output and VS Code extension
- Approximately 2m x 2m booth space for interactive demonstration

The live demo will showcase:
1. Compiling skills from unified format to multiple platforms (Claude, Codex, Gemini)
2. Real-time validation with semantic analysis and security auditing
3. VS Code extension for skill development workflow
4. Performance comparison between original and compiled skills

---

## Additional Information

### Key Technical Highlights

| Feature | Description |
|---------|-------------|
| Multi-Target Compilation | Single source to Claude, Codex, Gemini |
| Semantic Validation | 100+ validation rules with actionable diagnostics |
| Security-First Design | Permission auditing, HITL triggers, Anti-Skill patterns |
| High Performance | 16.9% faster execution (validated by experiments) |
| Developer Experience | Beautiful CLI with miette error reporting |

### Performance Metrics

| Metric | Original Skills | Compiled Skills | Improvement |
|--------|-----------------|-----------------|-------------|
| Avg Duration | 143.99s | 119.65s | **16.9% faster** |
| Success Rate | 96% | 100% | +4% |
| Quality Score | 0.92 | 0.94 | +2.2% |

### Technology Stack

- **Language:** Rust (high performance, memory safety)
- **Architecture:** Classic compiler design (Frontend → IR → Backend)
- **Distribution:** npm package, cargo crate, VS Code extension
- **License:** MIT (fully open source)

---

## References

- GitHub Repository: https://github.com/ouyangyipeng/Skill-Compiler
- Academic Paper: [NSC_ACADEMIC_PAPER.md](papers/NSC_ACADEMIC_PAPER.md)
- Experiment Report: [LARGE_SCALE_EXPERIMENT_REPORT.md](experiments/LARGE_SCALE_EXPERIMENT_REPORT.md)
- Documentation: [docs/](docs/)