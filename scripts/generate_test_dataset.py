#!/usr/bin/env python3
"""
Generate Large Scale Test Dataset for NSC
Generates 50+ skill files covering various scenarios
"""

import os
import json
import random
import string
from pathlib import Path

# Output directory
OUTPUT_DIR = Path("tests/fixtures/large_corpus")
OUTPUT_DIR.mkdir(parents=True, exist_ok=True)

# Templates for different skill types
SKILL_TEMPLATES = {
    "simple": '''---
name: {name}
description: {description}
version: "1.0.0"
---

# {title}

## Description

{description}

## Procedures

1. Analyze the input data.
2. Process according to requirements.
3. Return the result.

## Examples

### Example 1

**Input**: Sample input data
**Output**: Expected output
''',

    "medium": '''---
name: {name}
description: {description}
version: "1.0.0"
mcp_servers:
  - filesystem
  - network
hitl_required: false
security_level: medium
permissions:
  - kind: filesystem
    scope: /tmp/*
  - kind: network
    scope: https://api.example.com/*
---

# {title}

## Description

{description}

## Triggers

- When user requests {trigger}
- When system detects {trigger}

## Context Gathering

1. Check current state.
2. Gather required resources.
3. Validate prerequisites.

## Procedures

1. **Initialize**: Set up the environment.
2. **Process**: Execute the main logic.
3. **Validate**: Check the results.
4. **Cleanup**: Release resources.

## Examples

### Example 1: Basic Usage

**Input**: 
```
{example_input}
```

**Output**:
```
{example_output}
```

## Edge Cases

- Handle empty input gracefully.
- Retry on transient failures.
- Log all operations.

## Fallbacks

- If primary method fails, use backup.
- Report errors to monitoring.
''',

    "complex": '''---
name: {name}
description: {description}
version: "2.0.0"
mcp_servers:
  - filesystem
  - network
  - database
hitl_required: true
security_level: high
pre_conditions:
  - System is initialized
  - Network is available
post_conditions:
  - Data is persisted
  - Resources are released
permissions:
  - kind: filesystem
    scope: /data/**
    description: Read and write data files
  - kind: network
    scope: https://api.example.com/**
    description: API access
  - kind: database
    scope: postgres:main:SELECT,INSERT,UPDATE
    description: Database operations
input_schema:
  type: object
  properties:
    query:
      type: string
      description: Search query
    limit:
      type: integer
      default: 10
    filters:
      type: object
      properties:
        category:
          type: string
        date_range:
          type: object
          properties:
            start:
              type: string
              format: date
            end:
              type: string
              format: date
  required:
    - query
output_schema:
  type: object
  properties:
    results:
      type: array
      items:
        type: object
    total:
      type: integer
    metadata:
      type: object
---

# {title}

## Description

{description}

This skill implements a comprehensive workflow for {workflow_target}.

## Triggers

- User explicitly requests {trigger}
- Scheduled automation at {schedule}
- Event-driven from {event_source}

## Context Gathering

### Step 1: Environment Check

Verify all required services are available:
- Database connection
- API endpoints
- File system permissions

### Step 2: Resource Allocation

- Reserve memory buffer
- Open database connections
- Initialize logging context

### Step 3: Input Validation

- Validate input schema
- Check permission scope
- Verify rate limits

## Procedures

### Phase 1: Initialization

1. **[CRITICAL]** Load configuration from environment.
2. Establish database connection pool.
3. Initialize API clients with authentication.

### Phase 2: Processing

4. Parse and validate input parameters.
5. Execute primary query logic.
6. **[CRITICAL]** Apply business rules transformation.
7. Aggregate results from multiple sources.

### Phase 3: Output Generation

8. Format output according to schema.
9. Apply pagination if needed.
10. **[CRITICAL]** Persist audit log.

### Phase 4: Cleanup

11. Close database connections.
12. Release allocated resources.
13. Send completion notification.

## Examples

### Example 1: Basic Search

**Input**:
```json
{{
  "query": "test query",
  "limit": 5
}}
```

**Output**:
```json
{{
  "results": [
    {{"id": 1, "name": "Result 1"}},
    {{"id": 2, "name": "Result 2"}}
  ],
  "total": 2,
  "metadata": {{
    "query_time_ms": 45
  }}
}}
```

### Example 2: Filtered Search

**Input**:
```json
{{
  "query": "advanced query",
  "limit": 10,
  "filters": {{
    "category": "technology",
    "date_range": {{
      "start": "2024-01-01",
      "end": "2024-12-31"
    }}
  }}
}}
```

## Edge Cases

### Empty Results

When no results are found, return:
```json
{{
  "results": [],
  "total": 0,
  "metadata": {{
    "message": "No results found"
  }}
}}
```

### Rate Limiting

If rate limit is exceeded:
1. Wait for reset time
2. Retry with exponential backoff
3. Fail gracefully after max retries

### Permission Denied

If user lacks required permissions:
1. Log the access attempt
2. Return clear error message
3. Suggest required permissions

## Fallbacks

### Primary Database Unavailable

1. Switch to read replica
2. Use cached results if available
3. Return degraded response with warning

### API Timeout

1. Retry with increased timeout
2. Use cached response
3. Return partial results

## Post-Conditions

- All database transactions are committed
- Audit log is persisted
- Resources are properly released
- Metrics are recorded

## References

- API Documentation: https://api.example.com/docs
- Database Schema: https://db.example.com/schema
- Runbook: https://wiki.example.com/runbooks/{name}
''',

    "minimal": '''---
name: {name}
description: {description}
---

## Procedures

1. Execute task.
''',

    "error_test": '''---
name: {name}
{yaml_content}
---

{body_content}
'''
}

# Skill categories and their configurations
SKILL_CATEGORIES = [
    # Simple skills (20)
    ("simple", "data-processing", "Data Processing Skill"),
    ("simple", "text-analysis", "Text Analysis Skill"),
    ("simple", "file-operations", "File Operations Skill"),
    ("simple", "basic-calculation", "Basic Calculation Skill"),
    ("simple", "string-manipulation", "String Manipulation Skill"),
    ("simple", "json-parser", "JSON Parser Skill"),
    ("simple", "csv-handler", "CSV Handler Skill"),
    ("simple", "xml-processor", "XML Processor Skill"),
    ("simple", "markdown-converter", "Markdown Converter Skill"),
    ("simple", "log-analyzer", "Log Analyzer Skill"),
    ("simple", "email-validator", "Email Validator Skill"),
    ("simple", "url-parser", "URL Parser Skill"),
    ("simple", "date-formatter", "Date Formatter Skill"),
    ("simple", "number-formatter", "Number Formatter Skill"),
    ("simple", "encoding-converter", "Encoding Converter Skill"),
    ("simple", "hash-generator", "Hash Generator Skill"),
    ("simple", "uuid-generator", "UUID Generator Skill"),
    ("simple", "random-generator", "Random Generator Skill"),
    ("simple", "base64-handler", "Base64 Handler Skill"),
    ("simple", "regex-matcher", "Regex Matcher Skill"),
    
    # Medium skills (15)
    ("medium", "api-client", "API Client Skill"),
    ("medium", "database-query", "Database Query Skill"),
    ("medium", "file-sync", "File Sync Skill"),
    ("medium", "web-scraper", "Web Scraper Skill"),
    ("medium", "email-sender", "Email Sender Skill"),
    ("medium", "notification-service", "Notification Service Skill"),
    ("medium", "cache-manager", "Cache Manager Skill"),
    ("medium", "queue-processor", "Queue Processor Skill"),
    ("medium", "batch-processor", "Batch Processor Skill"),
    ("medium", "report-generator", "Report Generator Skill"),
    ("medium", "data-validator", "Data Validator Skill"),
    ("medium", "transform-pipeline", "Transform Pipeline Skill"),
    ("medium", "workflow-engine", "Workflow Engine Skill"),
    ("medium", "scheduler", "Scheduler Skill"),
    ("medium", "monitor", "Monitor Skill"),
    
    # Complex skills (10)
    ("complex", "ml-pipeline", "ML Pipeline Skill"),
    ("complex", "data-warehouse", "Data Warehouse Skill"),
    ("complex", "etl-framework", "ETL Framework Skill"),
    ("complex", "microservice-orchestrator", "Microservice Orchestrator Skill"),
    ("complex", "event-processor", "Event Processor Skill"),
    ("complex", "streaming-analytics", "Streaming Analytics Skill"),
    ("complex", "distributed-compute", "Distributed Compute Skill"),
    ("complex", "security-scanner", "Security Scanner Skill"),
    ("complex", "compliance-checker", "Compliance Checker Skill"),
    ("complex", "audit-logger", "Audit Logger Skill"),
    
    # Minimal skills (5)
    ("minimal", "quick-action", "Quick Action Skill"),
    ("minimal", "simple-trigger", "Simple Trigger Skill"),
    ("minimal", "basic-hook", "Basic Hook Skill"),
    ("minimal", "lite-processor", "Lite Processor Skill"),
    ("minimal", "nano-service", "Nano Service Skill"),
]

# Error test cases
ERROR_CASES = [
    ("missing-name", "name:\ndescription: Test"),
    ("missing-description", "name: test\ndescription:"),
    ("invalid-yaml", "name: [unclosed"),
    ("empty-frontmatter", ""),
    ("invalid-security", "name: test\ndescription: test\nsecurity_level: invalid"),
    ("invalid-version", "name: test\ndescription: test\nversion: not-semver"),
    ("invalid-mcp", "name: test\ndescription: test\nmcp_servers: not-a-list"),
    ("invalid-permissions", "name: test\ndescription: test\npermissions: invalid"),
    ("missing-frontmatter", None),  # No frontmatter at all
    ("malformed-frontmatter", "---\nname: test\n---\n---\nname: duplicate\n---"),
]


def generate_name(category: str, idx: int) -> str:
    """Generate a unique skill name."""
    return f"{category}-skill-{idx:03d}"


def generate_description(name: str, category: str) -> str:
    """Generate a description for the skill."""
    descriptions = {
        "data-processing": "Process and transform data according to specified rules.",
        "text-analysis": "Analyze text content and extract meaningful insights.",
        "file-operations": "Perform file system operations safely and efficiently.",
        "api-client": "Interact with external APIs and manage responses.",
        "database-query": "Execute database queries with proper connection management.",
        "ml-pipeline": "Execute machine learning pipeline stages.",
    }
    return descriptions.get(category, f"A skill for {category.replace('-', ' ')} operations.")


def generate_skill_file(template_type: str, name: str, title: str, description: str, idx: int) -> str:
    """Generate a skill file from template."""
    template = SKILL_TEMPLATES[template_type]
    
    return template.format(
        name=name,
        description=description,
        title=title,
        trigger=f"trigger condition {idx}",
        workflow_target=f"workflow target {idx}",
        schedule="0 * * * *",
        event_source=f"event source {idx}",
        example_input=f"sample input {idx}",
        example_output=f"sample output {idx}",
    )


def generate_error_skill(case_name: str, yaml_content: str, body_content: str) -> str:
    """Generate an error test skill file."""
    if yaml_content is None:
        return body_content
    
    return f"---\n{yaml_content}\n---\n\n{body_content}"


def main():
    """Generate all test skill files."""
    print("Generating large scale test dataset...")
    
    # Generate normal skills
    for idx, (template_type, category, title) in enumerate(SKILL_CATEGORIES):
        name = generate_name(category, idx + 1)
        description = generate_description(name, category)
        
        content = generate_skill_file(template_type, name, title, description, idx + 1)
        
        output_path = OUTPUT_DIR / f"{name}.md"
        output_path.write_text(content)
        print(f"  Generated: {name}.md")
    
    # Generate error test cases
    for idx, (case_name, yaml_content) in enumerate(ERROR_CASES):
        body_content = f"# Error Test {idx + 1}\n\nThis is an error test case.\n"
        
        if yaml_content is None:
            content = body_content
        else:
            content = generate_error_skill(case_name, yaml_content, body_content)
        
        output_path = OUTPUT_DIR / f"error-{case_name}.md"
        output_path.write_text(content)
        print(f"  Generated: error-{case_name}.md")
    
    # Generate manifest
    manifest = {
        "total_files": len(SKILL_CATEGORIES) + len(ERROR_CASES),
        "categories": {
            "simple": sum(1 for t, _, _ in SKILL_CATEGORIES if t == "simple"),
            "medium": sum(1 for t, _, _ in SKILL_CATEGORIES if t == "medium"),
            "complex": sum(1 for t, _, _ in SKILL_CATEGORIES if t == "complex"),
            "minimal": sum(1 for t, _, _ in SKILL_CATEGORIES if t == "minimal"),
            "error": len(ERROR_CASES),
        },
        "generated_at": "2026-04-03T14:20:00Z",
    }
    
    manifest_path = OUTPUT_DIR / "manifest.json"
    manifest_path.write_text(json.dumps(manifest, indent=2))
    print(f"\nGenerated {manifest['total_files']} test files")
    print(f"Categories: {manifest['categories']}")


if __name__ == "__main__":
    main()