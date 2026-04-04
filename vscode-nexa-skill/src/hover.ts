/**
 * Hover Provider - Show documentation on hover
 */

import * as vscode from 'vscode';

interface FieldDocumentation {
    description: string;
    type: string;
    required: boolean;
    example?: string;
}

const FIELD_DOCS: Record<string, FieldDocumentation> = {
    name: {
        description: 'The unique identifier name for this skill. Used for skill discovery and invocation.',
        type: 'string',
        required: true,
        example: 'database-migration'
    },
    description: {
        description: 'A brief description of what this skill does. Used for skill discovery and documentation.',
        type: 'string',
        required: true,
        example: 'PostgreSQL schema migration with safety guarantees'
    },
    version: {
        description: 'Semantic version of the skill. Used for version management and compatibility checks.',
        type: 'string',
        required: false,
        example: '1.0.0'
    },
    author: {
        description: 'The author or team responsible for this skill.',
        type: 'string',
        required: false,
        example: 'Your Name'
    },
    compatibility: {
        description: 'List of target platforms this skill is compatible with.',
        type: 'array',
        required: false,
        example: '[claude, codex, gemini]'
    },
    security_level: {
        description: 'Security classification determining permission requirements and audit intensity.',
        type: 'enum',
        required: false,
        example: 'medium'
    },
    hitl_required: {
        description: 'Whether Human-in-the-Loop confirmation is required before execution.',
        type: 'boolean',
        required: false,
        example: 'true'
    },
    mcp_servers: {
        description: 'List of MCP servers required by this skill.',
        type: 'array',
        required: false,
        example: '[postgres-admin, slack]'
    },
    permissions: {
        description: 'Permissions required by this skill, such as file system or network access.',
        type: 'array',
        required: false,
        example: '[{kind: database, scope: postgresql://localhost/*}]'
    },
    input_schema: {
        description: 'JSON Schema defining the expected input structure.',
        type: 'object',
        required: false,
        example: '{type: object, properties: {query: {type: string}}}'
    },
    pre_conditions: {
        description: 'Conditions that must be true before skill execution.',
        type: 'array',
        required: false,
        example: '[Database connection is available, Backup exists]'
    },
    post_conditions: {
        description: 'Conditions that will be true after successful skill execution.',
        type: 'array',
        required: false,
        example: '[Schema is updated, Migration log is written]'
    },
    fallbacks: {
        description: 'Fallback strategies when execution fails.',
        type: 'array',
        required: false,
        example: '[Rollback transaction, Notify administrator]'
    }
};

const SECURITY_LEVEL_DOCS: Record<string, string> = {
    low: 'Low security - Minimal restrictions. Suitable for read-only operations.',
    medium: 'Medium security - Standard restrictions. Suitable for most operations.',
    high: 'High security - Elevated restrictions. Requires additional validation.',
    critical: 'Critical security - Maximum restrictions. Requires HITL approval.'
};

export class SkillHoverProvider implements vscode.HoverProvider {
    provideHover(
        document: vscode.TextDocument,
        position: vscode.Position,
        token: vscode.CancellationToken
    ): vscode.ProviderResult<vscode.Hover> {
        const range = document.getWordRangeAtPosition(position);
        if (!range) {
            return undefined;
        }

        const word = document.getText(range);
        const line = document.lineAt(position.line).text;

        // Check if we're in frontmatter
        const textBefore = document.getText(
            new vscode.Range(new vscode.Position(0, 0), position)
        );
        const frontmatterMatch = textBefore.match(/^---\n([\s\S]*?)$/);
        
        if (frontmatterMatch) {
            // Check for field documentation
            if (FIELD_DOCS[word]) {
                return this.createFieldHover(word, FIELD_DOCS[word]);
            }

            // Check for security_level value
            if (line.includes('security_level:') && SECURITY_LEVEL_DOCS[word]) {
                return new vscode.Hover(
                    new vscode.MarkdownString(SECURITY_LEVEL_DOCS[word]),
                    range
                );
            }
        }

        // Check for section headers
        if (line.startsWith('## ')) {
            return this.createSectionHover(word);
        }

        // Check for constraint keywords
        if (['ALWAYS', 'NEVER', 'REQUIRE'].includes(word)) {
            return this.createConstraintHover(word);
        }

        return undefined;
    }

    private createFieldHover(field: string, doc: FieldDocumentation): vscode.Hover {
        const md = new vscode.MarkdownString();
        
        md.appendMarkdown(`**${field}** \`${doc.type}\``);
        if (doc.required) {
            md.appendMarkdown(' *(required)*');
        }
        md.appendMarkdown('\n\n');
        md.appendMarkdown(doc.description);
        
        if (doc.example) {
            md.appendMarkdown('\n\n**Example:**\n```\n');
            md.appendMarkdown(`${field}: ${doc.example}`);
            md.appendMarkdown('\n```');
        }

        return new vscode.Hover(md);
    }

    private createSectionHover(section: string): vscode.Hover {
        const sections: Record<string, string> = {
            Triggers: 'Conditions or events that should cause this skill to be activated.',
            Procedures: 'Step-by-step instructions for executing the skill.',
            Examples: 'Concrete examples demonstrating skill usage.',
            Constraints: 'Rules that must be followed during skill execution.',
            Fallbacks: 'Error handling and recovery strategies.',
            References: 'External documentation or resources.',
            'Context': 'Information gathering steps before execution.',
            'Edge': 'Common pitfalls and edge cases to handle.'
        };

        const description = sections[section] || `Section: ${section}`;
        return new vscode.Hover(new vscode.MarkdownString(description));
    }

    private createConstraintHover(keyword: string): vscode.Hover {
        const constraints: Record<string, string> = {
            ALWAYS: '**ALWAYS** - This action must be performed every time. Use for critical steps that should never be skipped.',
            NEVER: '**NEVER** - This action must never be performed. Use to prevent dangerous or unwanted operations.',
            REQUIRE: '**REQUIRE** - This condition must be met before proceeding. Use for validation and user confirmations.'
        };

        return new vscode.Hover(new vscode.MarkdownString(constraints[keyword]));
    }
}