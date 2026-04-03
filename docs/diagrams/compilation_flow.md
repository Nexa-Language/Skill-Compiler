# 编译流程图

> **Nexa Skill Compiler 完整编译流程的可视化展示**

---

## 编译管线流程图

```mermaid
flowchart TB
    subgraph Input
        A[SKILL.md 源文件]
    end
    
    subgraph "Phase 1: Frontend"
        B1[Frontmatter 剥离]
        B2[YAML 解析<br/>serde_yaml]
        B3[Markdown 解析<br/>pulldown-cmark]
        B4[AST 构建]
        
        B1 --> B2
        B1 --> B3
        B2 --> B4
        B3 --> B4
    end
    
    subgraph "Phase 2: IR Construction"
        C1[类型映射]
        C2[SkillIR 构建]
        C3[默认值填充]
        
        C1 --> C2
        C2 --> C3
    end
    
    subgraph "Phase 3: Analyzer"
        D1[Schema Validator]
        D2[MCP Dependency Checker]
        D3[Permission Auditor]
        D4[Anti-Skill Injector]
        
        D1 --> D2
        D2 --> D3
        D3 --> D4
    end
    
    subgraph "Phase 4: Backend"
        E1{选择目标平台}
        E2[Claude Emitter<br/>XML]
        E3[Codex Emitter<br/>JSON Schema]
        E4[Gemini Emitter<br/>Markdown]
        E5[Kimi Emitter<br/>Text]
        
        E1 --> E2
        E1 --> E3
        E1 --> E4
        E1 --> E5
    end
    
    subgraph Output
        F1[manifest.json]
        F2[claude.xml]
        F3[codex_schema.json]
        F4[gemini.md]
        F5[signature.sha256]
    end
    
    A --> B1
    B4 --> C1
    C3 --> D1
    D4 --> E1
    E2 --> F2
    E3 --> F3
    E4 --> F4
    C2 --> F1
    F1 --> F5
```

---

## 数据流图

```mermaid
flowchart LR
    subgraph 输入
        A[SKILL.md]
    end
    
    subgraph 解析
        B[RawAST]
    end
    
    subgraph 中间表示
        C[SkillIR]
        D[ValidatedSkillIR]
    end
    
    subgraph 产物
        E[Claude XML]
        F[Codex JSON]
        G[Gemini MD]
    end
    
    A -->|Frontend| B
    B -->|IR Builder| C
    C -->|Analyzer| D
    D -->|Claude Emitter| E
    D -->|Codex Emitter| F
    D -->|Gemini Emitter| G
```

---

## 错误处理流程

```mermaid
flowchart TD
    A[编译开始] --> B{解析成功?}
    
    B -->|否| C[ParseError]
    B -->|是| D{IR 构建成功?}
    
    C --> E[生成诊断报告]
    E --> F[显示错误]
    F --> G[编译终止]
    
    D -->|否| H[IRError]
    D -->|是| I{验证通过?}
    
    H --> E
    
    I -->|否| J[AnalyzeError]
    I -->|是| K{发射成功?}
    
    J --> E
    
    K -->|否| L[EmitError]
    K -->|是| M[生成产物]
    
    L --> E
    
    M --> N[编译成功]
```

---

## HITL 审批流程

```mermaid
sequenceDiagram
    participant User as 用户
    participant CLI as CLI
    participant Compiler as 编译器
    participant HITL as HITL 管理器
    
    User->>CLI: nexa-skill build skill.md
    CLI->>Compiler: 开始编译
    
    Compiler->>Compiler: Frontend 解析
    Compiler->>Compiler: IR 构建
    Compiler->>Compiler: Analyzer 分析
    
    alt 需要 HITL
        Compiler->>HITL: 检查 HITL 需求
        HITL->>CLI: 请求审批
        
        CLI->>User: 显示技能详情
        CLI->>User: 显示风险描述
        CLI->>User: 请求确认
        
        alt 用户确认
            User->>CLI: 确认
            CLI->>HITL: 用户批准
            HITL->>Compiler: 继续
        else 用户拒绝
            User->>CLI: 拒绝
            CLI->>HITL: 用户拒绝
            HITL->>Compiler: 终止
            Compiler->>CLI: 返回取消
        end
    end
    
    Compiler->>Compiler: Backend 生成
    Compiler->>CLI: 返回产物
    CLI->>User: 显示成功
```

---

## 相关文档

- [ARCHITECTURE.md](../ARCHITECTURE.md) - 系统架构总览
- [COMPILER_PIPELINE.md](../COMPILER_PIPELINE.md) - 编译管线详细设计
- [SECURITY_MODEL.md](../SECURITY_MODEL.md) - HITL 审批流程