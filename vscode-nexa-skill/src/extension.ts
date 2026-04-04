/**
 * Nexa Skill Compiler - VS Code Extension
 * 
 * Provides syntax highlighting, validation, and compilation for SKILL.md files
 */

import * as vscode from 'vscode';
import { SkillCompiler } from './compiler';
import { SkillDiagnosticsProvider } from './diagnostics';
import { SkillPreviewPanel } from './preview';
import { SkillCompletionProvider } from './completion';
import { SkillHoverProvider } from './hover';
import { StatusBarManager } from './statusBar';

let diagnosticsProvider: SkillDiagnosticsProvider;
let statusBarManager: StatusBarManager;
let compiler: SkillCompiler;

export function activate(context: vscode.ExtensionContext) {
    console.log('Nexa Skill Compiler extension is now active');

    // Initialize components
    compiler = new SkillCompiler();
    diagnosticsProvider = new SkillDiagnosticsProvider(compiler);
    statusBarManager = new StatusBarManager();

    // Register language configuration
    const languageConfig = vscode.languages.setLanguageConfiguration('skill', {
        wordPattern: /(-?\d*\.\d\w*)|([^\`\~\!\@\#\$\%\^\&\*\(\)\=\+\[\{\]\}\\\|\;\:\'\"\,\.\<\>\/\?\s]+)/g,
        comments: {
            lineComment: '#',
            blockComment: ['<!--', '-->']
        },
        brackets: [
            ['{', '}'],
            ['[', ']'],
            ['(', ')']
        ],
        autoClosingPairs: [
            { open: '{', close: '}' },
            { open: '[', close: ']' },
            { open: '(', close: ')' },
            { open: '"', close: '"' },
            { open: "'", close: "'" },
            { open: '`', close: '`' }
        ]
    });

    // Register commands
    const compileCommand = vscode.commands.registerCommand(
        'nsc.compile',
        async (uri?: vscode.Uri) => {
            const fileUri = uri || vscode.window.activeTextEditor?.document.uri;
            if (!fileUri) {
                vscode.window.showErrorMessage('No skill file selected');
                return;
            }
            await compileSkill(fileUri);
        }
    );

    const compileAllCommand = vscode.commands.registerCommand(
        'nsc.compileAll',
        async () => {
            await compileAllSkills();
        }
    );

    const validateCommand = vscode.commands.registerCommand(
        'nsc.validate',
        async (uri?: vscode.Uri) => {
            const fileUri = uri || vscode.window.activeTextEditor?.document.uri;
            if (!fileUri) {
                vscode.window.showErrorMessage('No skill file selected');
                return;
            }
            await validateSkill(fileUri);
        }
    );

    const checkCommand = vscode.commands.registerCommand(
        'nsc.check',
        async (uri?: vscode.Uri) => {
            const fileUri = uri || vscode.window.activeTextEditor?.document.uri;
            if (!fileUri) {
                vscode.window.showErrorMessage('No skill file selected');
                return;
            }
            await checkSkill(fileUri);
        }
    );

    const initCommand = vscode.commands.registerCommand(
        'nsc.init',
        async (uri?: vscode.Uri) => {
            await initNewSkill(uri);
        }
    );

    const previewCommand = vscode.commands.registerCommand(
        'nsc.showPreview',
        () => {
            const editor = vscode.window.activeTextEditor;
            if (editor) {
                SkillPreviewPanel.createOrShow(context.extensionUri, editor.document);
            }
        }
    );

    // Register completion provider
    const completionProvider = vscode.languages.registerCompletionItemProvider(
        'skill',
        new SkillCompletionProvider(),
        '.', '-', ' ', ':'
    );

    // Register hover provider
    const hoverProvider = vscode.languages.registerHoverProvider(
        'skill',
        new SkillHoverProvider()
    );

    // Register document listener for diagnostics
    const documentListener = vscode.workspace.onDidChangeTextDocument(
        (event) => {
            if (event.document.languageId === 'skill' || 
                event.document.fileName.endsWith('SKILL.md')) {
                diagnosticsProvider.provideDiagnostics(event.document);
            }
        }
    );

    // Register save listener for auto-compile
    const saveListener = vscode.workspace.onDidSaveTextDocument(
        async (document) => {
            const config = vscode.workspace.getConfiguration('nsc');
            if (config.get<boolean>('enableAutoCompile')) {
                if (document.languageId === 'skill' || 
                    document.fileName.endsWith('SKILL.md')) {
                    await compileSkill(document.uri);
                }
            }
        }
    );

    // Register all disposables
    context.subscriptions.push(
        languageConfig,
        compileCommand,
        compileAllCommand,
        validateCommand,
        checkCommand,
        initCommand,
        previewCommand,
        completionProvider,
        hoverProvider,
        documentListener,
        saveListener,
        statusBarManager
    );

    // Initial diagnostics for open documents
    vscode.workspace.textDocuments.forEach((document) => {
        if (document.languageId === 'skill' || document.fileName.endsWith('SKILL.md')) {
            diagnosticsProvider.provideDiagnostics(document);
        }
    });

    // Show status bar
    const config = vscode.workspace.getConfiguration('nsc');
    if (config.get<boolean>('showStatusBar')) {
        statusBarManager.show();
    }
}

export function deactivate() {
    console.log('Nexa Skill Compiler extension deactivated');
    if (statusBarManager) {
        statusBarManager.hide();
    }
}

/**
 * Compile a single skill file
 */
async function compileSkill(uri: vscode.Uri): Promise<void> {
    const config = vscode.workspace.getConfiguration('nsc');
    const target = config.get<string>('defaultTarget') || 'all';
    const outputDir = config.get<string>('outputDirectory') || 'dist';

    statusBarManager.setCompiling();

    try {
        const result = await compiler.compile(uri.fsPath, {
            target: target as 'claude' | 'codex' | 'gemini' | 'all',
            outputDir
        });

        if (result.success) {
            statusBarManager.setSuccess();
            vscode.window.showInformationMessage(
                `Skill compiled successfully to ${result.outputPath}`
            );
        } else {
            statusBarManager.setError();
            vscode.window.showErrorMessage(
                `Compilation failed: ${result.error}`
            );
        }
    } catch (error) {
        statusBarManager.setError();
        vscode.window.showErrorMessage(
            `Compilation error: ${error instanceof Error ? error.message : String(error)}`
        );
    }
}

/**
 * Compile all skill files in the workspace
 */
async function compileAllSkills(): Promise<void> {
    const skillFiles = await vscode.workspace.findFiles('**/SKILL.md');
    
    if (skillFiles.length === 0) {
        vscode.window.showWarningMessage('No skill files found in workspace');
        return;
    }

    const config = vscode.workspace.getConfiguration('nsc');
    const target = config.get<string>('defaultTarget') || 'all';
    const outputDir = config.get<string>('outputDirectory') || 'dist';

    statusBarManager.setCompiling();

    let successCount = 0;
    let failCount = 0;

    await vscode.window.withProgress(
        {
            location: vscode.ProgressLocation.Notification,
            title: 'Compiling skills...',
            cancellable: false
        },
        async (progress) => {
            for (let i = 0; i < skillFiles.length; i++) {
                const file = skillFiles[i];
                progress.report({
                    message: `Compiling ${file.fsPath.split('/').pop()} (${i + 1}/${skillFiles.length})`
                });

                try {
                    const result = await compiler.compile(file.fsPath, {
                        target: target as 'claude' | 'codex' | 'gemini' | 'all',
                        outputDir
                    });

                    if (result.success) {
                        successCount++;
                    } else {
                        failCount++;
                    }
                } catch {
                    failCount++;
                }
            }
        }
    );

    if (failCount === 0) {
        statusBarManager.setSuccess();
        vscode.window.showInformationMessage(
            `All ${successCount} skills compiled successfully`
        );
    } else {
        statusBarManager.setWarning();
        vscode.window.showWarningMessage(
            `${successCount} skills compiled, ${failCount} failed`
        );
    }
}

/**
 * Validate a skill file
 */
async function validateSkill(uri: vscode.Uri): Promise<void> {
    try {
        const result = await compiler.validate(uri.fsPath);

        if (result.valid) {
            vscode.window.showInformationMessage('Skill is valid');
        } else {
            const errors = result.errors.map(e => e.message).join('\n');
            vscode.window.showErrorMessage(`Validation failed:\n${errors}`);
        }
    } catch (error) {
        vscode.window.showErrorMessage(
            `Validation error: ${error instanceof Error ? error.message : String(error)}`
        );
    }
}

/**
 * Check a skill file
 */
async function checkSkill(uri: vscode.Uri): Promise<void> {
    try {
        const result = await compiler.check(uri.fsPath);

        if (result.diagnostics.length === 0) {
            vscode.window.showInformationMessage('No issues found');
        } else {
            const outputChannel = vscode.window.createOutputChannel('NSC Check');
            outputChannel.clear();
            
            result.diagnostics.forEach(d => {
                outputChannel.appendLine(
                    `${d.severity}: ${d.message} at line ${d.line}, column ${d.column}`
                );
            });

            outputChannel.show();
            vscode.window.showWarningMessage(
                `Found ${result.diagnostics.length} issues. See output for details.`
            );
        }
    } catch (error) {
        vscode.window.showErrorMessage(
            `Check error: ${error instanceof Error ? error.message : String(error)}`
        );
    }
}

/**
 * Initialize a new skill file
 */
async function initNewSkill(uri?: vscode.Uri): Promise<void> {
    const folderUri = uri || 
        (vscode.workspace.workspaceFolders?.[0]?.uri);

    if (!folderUri) {
        vscode.window.showErrorMessage('No folder selected');
        return;
    }

    // Prompt for skill name
    const skillName = await vscode.window.showInputBox({
        prompt: 'Enter skill name',
        placeHolder: 'my-skill',
        validateInput: (value) => {
            if (!value) {
                return 'Skill name is required';
            }
            if (!/^[a-z0-9-]+$/.test(value)) {
                return 'Skill name must be lowercase with hyphens only';
            }
            return null;
        }
    });

    if (!skillName) {
        return;
    }

    // Prompt for description
    const description = await vscode.window.showInputBox({
        prompt: 'Enter skill description',
        placeHolder: 'A skill that does something useful'
    });

    // Prompt for author
    const author = await vscode.window.showInputBox({
        prompt: 'Enter author name',
        placeHolder: 'Your Name'
    });

    try {
        const skillContent = generateSkillTemplate(skillName, description || '', author || '');

        const skillDir = vscode.Uri.joinPath(folderUri, skillName);
        await vscode.workspace.fs.createDirectory(skillDir);

        const skillFile = vscode.Uri.joinPath(skillDir, 'SKILL.md');
        const encoder = new TextEncoder();
        await vscode.workspace.fs.writeFile(skillFile, encoder.encode(skillContent));

        // Open the new skill file
        const document = await vscode.workspace.openTextDocument(skillFile);
        await vscode.window.showTextDocument(document);

        vscode.window.showInformationMessage(
            `Skill "${skillName}" created successfully`
        );
    } catch (error) {
        vscode.window.showErrorMessage(
            `Failed to create skill: ${error instanceof Error ? error.message : String(error)}`
        );
    }
}

/**
 * Generate skill template content
 */
function generateSkillTemplate(name: string, description: string, author: string): string {
    return `---
name: ${name}
description: ${description || 'A skill for...'}
version: 1.0.0
author: ${author || 'Anonymous'}
compatibility:
  - claude
  - codex
  - gemini
---

# ${name.split('-').map(w => w.charAt(0).toUpperCase() + w.slice(1)).join(' ')}

${description || 'Describe what this skill does here.'}

## Triggers

- When the user asks to...
- When working with...

## Procedures

### 1. Context Gathering

- Gather necessary context before proceeding
- Identify key information needed

### 2. Execution

- Execute the main task
- Follow best practices

### 3. Verification

- Verify the output
- Ensure quality standards

## Examples

### Example 1: Basic Usage

\`\`\`
User: [Example user request]
Assistant: [Example response]
\`\`\`

## Constraints

- ALWAYS follow the procedures in order
- NEVER skip verification steps
- REQUIRE user confirmation for destructive actions

## Fallbacks

- If the primary approach fails, try...
- On error, report to user and suggest alternatives
`;
}