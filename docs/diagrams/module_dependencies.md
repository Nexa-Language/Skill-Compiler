# 模块依赖关系图

> **Nexa Skill Compiler 各模块之间的依赖关系**

---

## Crate 依赖关系

```mermaid
graph TB
    subgraph "应用层"
        CLI[nexa-skill-cli]
    end
    
    subgraph "核心层"
        CORE[nexa-skill-core]
        TPL[nexa-skill-templates]
    end
    
    subgraph "扩展层"
        WASM[nexa-skill-wasm]
    end
    
    subgraph "外部依赖"
        CLAP[clap]
        MIETTE[miette]
        PULLDOWN[pulldown-cmark]
        SERDE[serde]
        SERDE_JSON[serde_json]
        SERDE_YAML[serde_yaml]
        ASKAMA[askama]
        TOKIO[tokio]
        TRACING[tracing]
    end
    
    CLI --> CORE
    CLI --> TPL
    CLI --> CLAP
    CLI --> MIETTE
    
    CORE --> TPL
    CORE --> PULLDOWN
    CORE --> SERDE
    CORE --> SERDE_JSON
    CORE --> SERDE_YAML
    CORE --> TOKIO
    CORE --> TRACING
    
    TPL --> ASKAMA
    
    WASM --> CORE
```

---

## 核心模块依赖

```mermaid
graph LR
    subgraph "Frontend"
        FM[frontmatter]
        MD[markdown]
        AST[ast]
    end
    
    subgraph "IR"
        IR[skill_ir]
        PROC[procedure]
        PERM[permission]
        CONS[constraint]
    end
    
    subgraph "Analyzer"
        SCH[schema]
        MCP[mcp]
        PAUD[permission]
        ANTI[anti_skill]
    end
    
    subgraph "Backend"
        EMIT[emitter]
        CLAUDE[claude]
        CODEX[codex]
        GEMINI[gemini]
    end
    
    subgraph "Security"
        SEC_LVL[level]
        SEC_PERM[permission]
        HITL[hitl]
    end
    
    subgraph "Error"
        DIAG[diagnostic]
        CODES[codes]
    end
    
    FM --> AST
    MD --> AST
    AST --> IR
    
    IR --> PROC
    IR --> PERM
    IR --> CONS
    
    IR --> SCH
    IR --> MCP
    IR --> PAUD
    IR --> ANTI
    
    IR --> EMIT
    EMIT --> CLAUDE
    EMIT --> CODEX
    EMIT --> GEMINI
    
    IR --> SEC_LVL
    IR --> SEC_PERM
    IR --> HITL
    
    AST --> DIAG
    IR --> DIAG
    ANTI --> DIAG
```

---

## 数据流依赖

```mermaid
flowchart TD
    subgraph "输入数据"
        A1[SKILL.md 文件]
        A2[配置文件]
        A3[Anti-Skill 模式库]
    end
    
    subgraph "中间数据"
        B1[RawAST]
        B2[SkillIR]
        B3[ValidatedSkillIR]
    end
    
    subgraph "输出数据"
        C1[manifest.json]
        C2[目标产物]
        C3[诊断报告]
    end
    
    A1 --> B1
    A2 --> B2
    A3 --> B3
    
    B1 --> B2
    B2 --> B3
    
    B3 --> C1
    B3 --> C2
    B3 --> C3
```

---

## 测试依赖

```mermaid
graph TB
    subgraph "测试类型"
        UNIT[单元测试]
        INT[集成测试]
        E2E[端到端测试]
        BENCH[性能测试]
    end
    
    subgraph "测试固件"
        FIX_SKILL[测试技能文件]
        FIX_CONFIG[测试配置]
        FIX_EXPECT[预期产物]
    end
    
    subgraph "测试工具"
        TEMPFILE[tempfile]
        ASSERT[pretty_assertions]
        SERIAL[serial_test]
    end
    
    UNIT --> CORE[nexa-skill-core]
    INT --> CORE
    INT --> CLI[nexa-skill-cli]
    E2E --> CLI
    
    UNIT --> FIX_SKILL
    INT --> FIX_SKILL
    INT --> FIX_CONFIG
    INT --> FIX_EXPECT
    
    UNIT --> TEMPFILE
    UNIT --> ASSERT
    INT --> SERIAL
```

---

## 相关文档

- [ARCHITECTURE.md](../ARCHITECTURE.md) - 系统架构总览
- [DEVELOPMENT_GUIDE.md](../DEVELOPMENT_GUIDE.md) - 开发指南
- [TESTING_STRATEGY.md](../TESTING_STRATEGY.md) - 测试策略