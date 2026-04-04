/**
 * Completion Provider - Auto-completion for skill files
 */

import * as vscode from 'vscode';

export class SkillCompletionProvider implements vscode.CompletionItemProvider {
    private readonly frontmatterFields: vscode.CompletionItem[] = [
        this.createField('name', 'string', 'Skill name (required)'),
        this.createField('description', 'string', 'Skill description (required)'),
        this.createField('version', 'string', 'Skill version (semver)'),
        this.createField('author', 'string', 'Author name'),
        this.createField('compatibility', 'array', 'Compatible platforms'),
        this.createField('security_level', 'enum', 'Security level: low, medium, high, critical'),
        this.createField('hitl_required', 'boolean', 'Human-in-the-loop required'),
        this.createField('mcp_servers', 'array', 'Required MCP servers'),
        this.createField('permissions', 'array', 'Required permissions'),
        this.createField('input_schema', 'object', 'Input JSON schema'),
        this.createField('pre_conditions', 'array', 'Pre-conditions for skill execution'),
        this.createField('post_conditions', 'array', 'Post-conditions after execution'),
        this.createField('fallbacks', 'array', 'Fallback strategies'),
    ];

    private readonly sections: vscode.CompletionItem[] = [
        this.createSection('Triggers', 'When to use this skill'),
        this.createSection('Context Gathering', 'How to gather context before execution'),
        this.createSection('Procedures', 'Execution steps'),
        this.createSection('Examples', 'Usage examples'),
        this.createSection('Constraints', 'Rules and constraints'),
        this.createSection('Fallbacks', 'Error handling strategies'),
        this.createSection('References', 'External references'),
        this.createSection('Edge Cases', 'Common pitfalls and edge cases'),
    ];

    private readonly securityLevels: vscode.CompletionItem[] = [
        this.createValue('low', 'Low security - minimal restrictions'),
        this.createValue('medium', 'Medium security - standard restrictions'),
        this.createValue('high', 'High security - elevated restrictions'),
        this.createValue('critical', 'Critical security - requires HITL'),
    ];

    private readonly platforms: vscode.CompletionItem[] = [
        this.createValue('claude', 'Anthropic Claude'),
        this.createValue('codex', 'OpenAI Codex/GPT'),
        this.createValue('gemini', 'Google Gemini'),
    ];

    provideCompletionItems(
        document: vscode.TextDocument,
        position: vscode.Position,
        token: vscode.CancellationToken,
        context: vscode.CompletionContext
    ): vscode.ProviderResult<vscode.CompletionItem[] | vscode.CompletionList> {
        const textBefore = document.getText(
            new vscode.Range(new vscode.Position(0, 0), position)
        );

        // Check if we're in frontmatter
        const frontmatterMatch = textBefore.match(/^---\n([\s\S]*?)$/);
        if (frontmatterMatch) {
            return this.getFrontmatterCompletions(textBefore, position);
        }

        // Check if we're after frontmatter (in body)
        const afterFrontmatter = textBefore.match(/^---\n[\s\S]*?\n---\n?/);
        if (afterFrontmatter) {
            return this.getBodyCompletions(textBefore, position);
        }

        // At the very beginning - offer frontmatter template
        if (position.line === 0 && position.character === 0) {
            return [this.createFrontmatterTemplate()];
        }

        return [];
    }

    private getFrontmatterCompletions(text: string, position: vscode.Position): vscode.CompletionItem[] {
        const line = text.split('\n').pop() || '';
        const items: vscode.CompletionItem[] = [];

        // Check for field completion
        if (line.match(/^\s*\w*$/)) {
            items.push(...this.frontmatterFields);
        }

        // Check for security_level value
        if (line.match(/security_level:\s*\w*$/)) {
            items.push(...this.securityLevels);
        }

        // Check for compatibility value
        if (line.match(/compatibility:.*\w*$/)) {
            items.push(...this.platforms);
        }

        return items;
    }

    private getBodyCompletions(text: string, position: vscode.Position): vscode.CompletionItem[] {
        const lines = text.split('\n');
        const currentLine = lines[position.line] || '';
        const items: vscode.CompletionItem[] = [];

        // Check for section header
        if (currentLine.match(/^##?\s*\w*$/)) {
            items.push(...this.sections);
        }

        // Check for procedure step
        if (currentLine.match(/^###?\s*\d*\.?\s*\w*$/)) {
            items.push(
                this.createProcedureStep('Context Gathering', 'Gather necessary context'),
                this.createProcedureStep('Execution', 'Execute the main task'),
                this.createProcedureStep('Verification', 'Verify the results'),
                this.createProcedureStep('Cleanup', 'Clean up resources'),
            );
        }

        // Check for constraint keywords
        if (currentLine.match(/^-\s*(ALWAYS|NEVER|REQUIRE)/i)) {
            items.push(
                this.createConstraint('ALWAYS', 'Always perform this action'),
                this.createConstraint('NEVER', 'Never perform this action'),
                this.createConstraint('REQUIRE', 'Require user confirmation'),
            );
        }

        return items;
    }

    private createField(name: string, type: string, description: string): vscode.CompletionItem {
        const item = new vscode.CompletionItem(name, vscode.CompletionItemKind.Property);
        item.detail = type;
        item.documentation = description;
        item.insertText = new vscode.SnippetString(`${name}: \${1:value}`);
        return item;
    }

    private createSection(name: string, description: string): vscode.CompletionItem {
        const item = new vscode.CompletionItem(name, vscode.CompletionItemKind.Class);
        item.documentation = description;
        item.insertText = new vscode.SnippetString(`## ${name}\n\n$1`);
        return item;
    }

    private createValue(value: string, description: string): vscode.CompletionItem {
        const item = new vscode.CompletionItem(value, vscode.CompletionItemKind.Value);
        item.documentation = description;
        return item;
    }

    private createProcedureStep(name: string, description: string): vscode.CompletionItem {
        const item = new vscode.CompletionItem(name, vscode.CompletionItemKind.Method);
        item.documentation = description;
        item.insertText = new vscode.SnippetString(`### ${name}\n\n- $1`);
        return item;
    }

    private createConstraint(keyword: string, description: string): vscode.CompletionItem {
        const item = new vscode.CompletionItem(keyword, vscode.CompletionItemKind.Keyword);
        item.documentation = description;
        item.insertText = new vscode.SnippetString(`${keyword} $1`);
        return item;
    }

    private createFrontmatterTemplate(): vscode.CompletionItem {
        const item = new vscode.CompletionItem('SKILL.md template', vscode.CompletionItemKind.Snippet);
        item.documentation = 'Create a new skill file with standard frontmatter';
        item.insertText = new vscode.SnippetString(
`---
name: \${1:skill-name}
description: \${2:Skill description}
version: 1.0.0
author: \${3:Author Name}
compatibility:
  - claude
  - codex
  - gemini
security_level: \${4|low,medium,high,critical|}
hitl_required: \${5:false}
---

# \${6:Skill Title}

\${7:Detailed description of the skill.}

## Triggers

- \${8:When to use this skill}

## Procedures

### 1. Context Gathering

- \${9:Gather necessary context}

### 2. Execution

- \${10:Execute the main task}

### 3. Verification

- \${11:Verify the results}

## Examples

### Example 1: Basic Usage

\`\`\`
\${12:Example usage}
\`\`\`

## Constraints

- ALWAYS \${13:Follow best practices}
- NEVER \${14:Skip verification}
- REQUIRE \${15:User confirmation for destructive actions}

## Fallbacks

- \${16:Error handling strategy}
`
        );
        return item;
    }
}