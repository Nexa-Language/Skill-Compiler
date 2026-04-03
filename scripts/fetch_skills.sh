#!/bin/bash
# Fetch Authoritative Agent Skill Datasets
#
# This script downloads real SKILL.md files from authoritative sources
# and stores them in tests/fixtures/ for testing purposes.
#
# Usage:
#   ./scripts/fetch_skills.sh

set -e

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
FIXTURES_DIR="$PROJECT_ROOT/tests/fixtures"
TEMP_DIR="$PROJECT_ROOT/target/skill_fetch"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Create directories
mkdir -p "$FIXTURES_DIR"
mkdir -p "$TEMP_DIR"

log_info "Fetching authoritative Agent Skill datasets..."

# ============================================================================
# Source 1: agent-browser skill (from .agents/skills)
# ============================================================================
log_info "Fetching agent-browser skill..."

if [ -d "$HOME/.agents/skills/agent-browser" ]; then
    cp "$HOME/.agents/skills/agent-browser/SKILL.md" "$FIXTURES_DIR/agent-browser.md" 2>/dev/null || true
    log_info "  ✓ agent-browser skill copied from local"
else
    log_warn "  agent-browser not found locally, skipping"
fi

# ============================================================================
# Source 2: Create sample skills based on reference patterns
# ============================================================================
log_info "Creating sample skill fixtures..."

# Basic skill - minimal required fields
cat > "$FIXTURES_DIR/basic-skill.md" << 'EOF'
---
name: basic-skill
description: A basic skill with minimal required fields for testing parsing.
---

# Basic Skill

## Description

This is a basic skill for testing the parser with minimal required fields.

## Procedures

1. First, analyze the input.
2. Then, process the data.
3. Finally, return the result.

## Examples

### Example 1: Basic Usage

Run the skill with default parameters.
EOF

# Advanced skill - with all NSC extensions
cat > "$FIXTURES_DIR/advanced-skill.md" << 'EOF'
---
name: database-migration
description: PostgreSQL schema migration skill with full NSC extension support
version: "1.0.0"
mcp_servers:
  - filesystem
  - postgres
input_schema:
  type: object
  properties:
    migration_file:
      type: string
      description: Path to migration SQL file
    dry_run:
      type: boolean
      default: true
  required:
    - migration_file
hitl_required: true
pre_conditions:
  - Database backup exists
  - Migration file is valid SQL
post_conditions:
  - Migration applied successfully
  - Schema version updated
fallbacks:
  - Rollback migration
  - Restore from backup
permissions:
  - kind: filesystem
    scope: read:/data/migrations
  - kind: database
    scope: write:postgres://localhost/app
security_level: critical
---

# PostgreSQL Schema Migration

## Description

Safely apply database schema migrations with rollback support.

## Triggers

- User requests schema migration
- New migration file detected
- Manual deployment trigger

## Context Gathering

1. Check current schema version
2. Validate migration file syntax
3. Verify database connectivity
4. Check for pending transactions

## Procedures

### 1. Pre-Migration Validation

- Parse migration SQL
- Check for destructive operations
- Validate against current schema

### 2. Backup Creation

```sql
CREATE SCHEMA IF NOT EXISTS backup;
CREATE TABLE backup.schema_version AS SELECT * FROM schema_version;
```

### 3. Apply Migration

Execute migration in transaction:

```sql
BEGIN;
-- Migration SQL here
UPDATE schema_version SET version = '1.1.0';
COMMIT;
```

### 4. Post-Migration Verification

- Verify schema changes
- Run test queries
- Check application compatibility

## Strict Constraints

- NEVER run migrations without backup
- ALWAYS use transactions
- NEVER skip pre-conditions check
- REQUIRE HITL approval for critical changes

## Fallbacks

### Rollback Procedure

1. Restore from backup schema
2. Re-run previous version migration
3. Verify data integrity

## Post-Conditions

- [ ] Migration applied successfully
- [ ] Schema version updated
- [ ] Application tests pass
- [ ] Backup preserved

## Examples

### Example 1: Add Column

Migration file: `add_user_email.sql`

```sql
ALTER TABLE users ADD COLUMN email VARCHAR(255);
CREATE INDEX idx_users_email ON users(email);
```

### Example 2: Modify Column Type

Migration file: `modify_status_column.sql`

```sql
ALTER TABLE orders ALTER COLUMN status TYPE VARCHAR(50);
```

## References

- [PostgreSQL Migration Best Practices](https://www.postgresql.org/docs/current/ddl.html)
- [Schema Version Control](https://github.com/golang-migrate/migrate)
EOF

# Skill with security concerns - for testing permission auditor
cat > "$FIXTURES_DIR/system-admin-skill.md" << 'EOF'
---
name: system-admin
description: System administration skill with elevated permissions
permissions:
  - kind: shell
    scope: execute
  - kind: filesystem
    scope: read-write:/
security_level: critical
---

# System Administration

## Description

Perform system administration tasks with elevated permissions.

## Procedures

### 1. System Update

```bash
sudo apt update && sudo apt upgrade -y
```

### 2. Service Management

```bash
sudo systemctl restart nginx
```

### 3. Log Cleanup

```bash
rm -rf /var/log/old/*
```

## Strict Constraints

- REQUIRE explicit user approval for destructive operations
- NEVER run unverified scripts
- ALWAYS log operations

## Fallbacks

- Restore from system backup
- Contact system administrator
EOF

# Invalid skill - missing required fields (for error testing)
cat > "$FIXTURES_DIR/invalid-skill.md" << 'EOF'
---
description: This skill is missing the name field
---

# Invalid Skill

This skill is intentionally malformed for testing error handling.
EOF

# Empty frontmatter skill
cat > "$FIXTURES_DIR/empty-frontmatter.md" << 'EOF'
---
---

# Empty Frontmatter

This skill has empty frontmatter.
EOF

# No frontmatter skill
cat > "$FIXTURES_DIR/no-frontmatter.md" << 'EOF'
# No Frontmatter

This skill has no frontmatter at all.

## Procedures

Just some content without proper frontmatter.
EOF

log_info "  ✓ Sample skill fixtures created"

# ============================================================================
# Source 3: Download from GitHub (if available)
# ============================================================================
log_info "Attempting to fetch skills from GitHub..."

# Check for GitHub token
if [ -n "$GITHUB_TOKEN" ]; then
    AUTH_HEADER="Authorization: token $GITHUB_TOKEN"
else
    AUTH_HEADER=""
fi

# Try to fetch from awesome-agent-skills repository
REPO_URL="https://api.github.com/repos/awesome-agent-skills/skills/contents"
TEMP_SKILLS="$TEMP_DIR/github_skills"
mkdir -p "$TEMP_SKILLS"

# Function to download a file from GitHub
download_github_file() {
    local url="$1"
    local output="$2"
    
    if command -v curl &> /dev/null; then
        curl -sL ${AUTH_HEADER:+-H "$AUTH_HEADER"} "$url" -o "$output" 2>/dev/null
    elif command -v wget &> /dev/null; then
        wget -q ${AUTH_HEADER:+--header="$AUTH_HEADER"} "$url" -O "$output" 2>/dev/null
    else
        log_warn "Neither curl nor wget available"
        return 1
    fi
}

# List of known skill repositories to fetch from
SKILL_SOURCES=(
    "https://raw.githubusercontent.com/skills/agent-browser/main/SKILL.md"
    "https://raw.githubusercontent.com/skills/webapp-testing/main/SKILL.md"
)

for source in "${SKILL_SOURCES[@]}"; do
    filename=$(basename "$source" | sed 's/SKILL.md$/-github.md/')
    output="$FIXTURES_DIR/$filename"
    
    if download_github_file "$source" "$output"; then
        if [ -s "$output" ]; then
            log_info "  ✓ Downloaded: $filename"
        else
            rm -f "$output"
            log_warn "  Empty file: $filename"
        fi
    else
        log_warn "  Failed to download: $source"
    fi
done

# ============================================================================
# Summary
# ============================================================================
log_info "Skill fixtures summary:"
echo ""
echo "Fixtures directory: $FIXTURES_DIR"
echo ""
echo "Files:"
ls -la "$FIXTURES_DIR" 2>/dev/null || echo "  (no files)"
echo ""
log_info "Fetch complete!"

# Cleanup
rm -rf "$TEMP_DIR"