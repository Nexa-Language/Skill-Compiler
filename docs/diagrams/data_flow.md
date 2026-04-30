# 数据流图

> **Nexa Skill Compiler 数据在各阶段之间的流动**

---

## 完整数据流

```mermaid
flowchart TD
    subgraph "输入阶段"
        A1[SKILL.md 文件]
        A2[目录结构]
        A3[配置文件 nsc.toml]
    end
    
    subgraph "Phase 1: Frontend"
        B1[文件读取]
        B2[Frontmatter 提取]
        B3[YAML 解析]
        B4[Markdown 解析]
        B5[RawAST 构建]
        
        B1 --> B2
        B2 --> B3
        B2 --> B4
        B3 --> B5
        B4 --> B5
    end
    
    subgraph "Phase 2: IR Construction"
        C1[类型映射]
        C2[字段验证]
        C3[默认值填充]
        C4[SkillIR 构建]
        
        C1 --> C2
        C2 --> C3
        C3 --> C4
    end
    
    subgraph "Phase 3: Analyzer"
        D1[Schema 验证]
        D2[MCP 检查]
        D3[权限审计]
        D4[Anti-Skill 注入]
        D5[ValidatedSkillIR]
        
        D1 --> D2
        D2 --> D3
        D3 --> D4
        D4 --> D5
    end
    
    subgraph "Phase 4: Backend"
        E1[Emitter 选择]
        E2[模板渲染]
        E3[产物序列化]
        E4[文件写入]
        
        E1 --> E2
        E2 --> E3
        E3 --> E4
    end
    
    subgraph "输出阶段"
        F1[manifest.json]
        F2[target/claude.xml]
        F3[target/codex_schema.json]
        F4[target/gemini.md]
        F5[meta/signature.sha256]
    end
    
    A1 --> B1
    A2 --> B1
    A3 --> C1
    
    B5 --> C1
    C4 --> D1
    D5 --> E1
    E4 --> F1
    E4 --> F2
    E4 --> F3
    E4 --> F4
    E4 --> F5
```

---

## SkillIR 数据结构流

```mermaid
classDiagram
    class SKILL_md {
        +YAML Frontmatter
        +Markdown Body
    }
    
    class RawAST {
        +source_path: String
        +frontmatter: FrontmatterMeta
        +body: MarkdownBody
        +source_hash: String
    }
    
    class SkillIR {
        +name: Arc~str~
        +version: Arc~str~
        +description: String
        +mcp_servers: Vec~Arc~str~~
        +input_schema: Option~Value~
        +hitl_required: bool
        +permissions: Vec~Permission~
        +procedures: Vec~ProcedureStep~
        +anti_skill_constraints: Vec~Constraint~
    }
    
    class ValidatedSkillIR {
        +inner: SkillIR
        +diagnostics: Vec~Diagnostic~
        +validated_at: DateTime
    }
    
    class Permission {
        +kind: PermissionKind
        +scope: String
    }
    
    class ProcedureStep {
        +order: u32
        +instruction: String
        +is_critical: bool
    }
    
    class Constraint {
        +source: String
        +content: String
        +level: ConstraintLevel
    }
    
    SKILL_md --> RawAST : Frontend
    RawAST --> SkillIR : IR Builder
    SkillIR --> ValidatedSkillIR : Analyzer
    SkillIR --> Permission
    SkillIR --> ProcedureStep
    SkillIR --> Constraint
```

---

## 错误数据流

```mermaid
flowchart TD
    subgraph "错误来源"
        E1[ParseError]
        E2[IRError]
        E3[AnalyzeError]
        E4[EmitError]
        E5[IOError]
    end
    
    subgraph "错误聚合"
        D[Diagnostic]
        DC[DiagnosticCollector]
    end
    
    subgraph "错误处理"
        R1[终端渲染<br/>miette]
        R2[JSON 输出]
        R3[HTML 报告]
    end
    
    subgraph "错误恢复"
        S1[Abort]
        S2[Skip]
        S3[AutoFix]
        S4[RequestInput]
    end
    
    E1 --> D
    E2 --> D
    E3 --> D
    E4 --> D
    E5 --> D
    
    D --> DC
    
    DC --> R1
    DC --> R2
    DC --> R3
    
    D --> S1
    D --> S2
    D --> S3
    D --> S4
```

---

## 安全数据流

```mermaid
flowchart TD
    subgraph "安全输入"
        S1[权限声明]
        S2[安全等级]
        S3[Anti-Skill 模式库]
        S4[安全基线配置]
    end
    
    subgraph "安全分析"
        A1[PermissionAuditor]
        A2[SecurityLevelValidator]
        A3[AntiSkillInjector]
        A4[HITLManager]
    end
    
    subgraph "安全输出"
        O1[权限检查结果]
        O2[注入的约束]
        O3[HITL 审批请求]
        O4[审计日志]
    end
    
    S1 --> A1
    S2 --> A2
    S3 --> A3
    S4 --> A1
    
    A1 --> O1
    A2 --> O1
    A3 --> O2
    A4 --> O3
    
    O1 --> O4
    O2 --> O4
    O3 --> O4
```

---

## 产物生成数据流

```mermaid
flowchart LR
    subgraph "ValidatedSkillIR"
        IR[IR 数据]
    end
    
    subgraph "Emitter 选择"
        SEL{Target Platform}
        CL[Claude Emitter]
        CO[Codex Emitter]
        GE[Gemini Emitter]
        KI[Kimi Emitter]
    end
    
    subgraph "模板渲染"
        T1[Askama 模板]
        T2[JSON 序列化]
        T3[Markdown 生成]
    end
    
    subgraph "产物文件"
        O1[claude.xml]
        O2[codex_schema.json]
        O3[gemini.md]
        O4[kimi.md]
    end
    
    IR --> SEL
    SEL -->|Claude| CL
    SEL -->|Codex| CO
    SEL -->|Gemini| GE
    SEL -->|Kimi| KI
    
    CL --> T1
    CO --> T2
    GE --> T3
    KI --> T3
    
    T1 --> O1
    T2 --> O2
    T3 --> O3
    T3 --> O4
```

---

## 相关文档

- [ARCHITECTURE.md](../ARCHITECTURE.md) - 系统架构总览
- [IR_DESIGN.md](../IR_DESIGN.md) - 中间表示设计
- [COMPILER_PIPELINE.md](../COMPILER_PIPELINE.md) - 编译管线详细设计
- [SECURITY_MODEL.md](../SECURITY_MODEL.md) - 安全模型设计