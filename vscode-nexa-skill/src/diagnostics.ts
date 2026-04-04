/**
 * Diagnostics Provider - Real-time skill validation
 */

import * as vscode from 'vscode';
import { SkillCompiler, Diagnostic } from './compiler';

export class SkillDiagnosticsProvider {
    private diagnosticCollection: vscode.DiagnosticCollection;
    private compiler: SkillCompiler;
    private debounceTimer: NodeJS.Timeout | null = null;
    private readonly debounceDelay = 500; // ms

    constructor(compiler: SkillCompiler) {
        this.compiler = compiler;
        this.diagnosticCollection = vscode.languages.createDiagnosticCollection('skill');
    }

    /**
     * Provide diagnostics for a document
     */
    async provideDiagnostics(document: vscode.TextDocument): Promise<void> {
        // Debounce rapid changes
        if (this.debounceTimer) {
            clearTimeout(this.debounceTimer);
        }

        this.debounceTimer = setTimeout(async () => {
            await this.doProvideDiagnostics(document);
        }, this.debounceDelay);
    }

    private async doProvideDiagnostics(document: vscode.TextDocument): Promise<void> {
        const config = vscode.workspace.getConfiguration('nsc');
        if (!config.get<boolean>('enableDiagnostics')) {
            return;
        }

        // Check if compiler is available
        const isAvailable = await this.compiler.isAvailable();
        if (!isAvailable) {
            // Provide basic YAML syntax diagnostics instead
            this.provideYamlDiagnostics(document);
            return;
        }

        try {
            const result = await this.compiler.check(document.uri.fsPath);
            const diagnostics: vscode.Diagnostic[] = result.diagnostics.map(d => 
                this.convertDiagnostic(d, document)
            );

            this.diagnosticCollection.set(document.uri, diagnostics);
        } catch (error) {
            // Clear diagnostics on error
            this.diagnosticCollection.set(document.uri, []);
        }
    }

    /**
     * Provide basic YAML frontmatter diagnostics
     */
    private provideYamlDiagnostics(document: vscode.TextDocument): void {
        const diagnostics: vscode.Diagnostic[] = [];
        const text = document.getText();

        // Check for frontmatter
        const frontmatterMatch = text.match(/^---\n([\s\S]*?)\n---/);
        if (!frontmatterMatch) {
            const range = new vscode.Range(0, 0, 0, 0);
            const diagnostic = new vscode.Diagnostic(
                range,
                'Missing YAML frontmatter. Skills must start with ---',
                vscode.DiagnosticSeverity.Error
            );
            diagnostic.code = 'NSC001';
            diagnostics.push(diagnostic);
        } else {
            const frontmatter = frontmatterMatch[1];
            const frontmatterStart = document.positionAt(4); // After ---

            // Check for required fields
            const requiredFields = ['name', 'description'];
            for (const field of requiredFields) {
                if (!frontmatter.includes(`${field}:`)) {
                    const line = this.findLineInFrontmatter(document, frontmatterStart.line);
                    const range = new vscode.Range(line, 0, line, 100);
                    const diagnostic = new vscode.Diagnostic(
                        range,
                        `Missing required field: ${field}`,
                        vscode.DiagnosticSeverity.Error
                    );
                    diagnostic.code = `NSC002`;
                    diagnostics.push(diagnostic);
                }
            }

            // Check for valid security_level
            if (frontmatter.includes('security_level:')) {
                const levelMatch = frontmatter.match(/security_level:\s*(\w+)/);
                if (levelMatch) {
                    const validLevels = ['low', 'medium', 'high', 'critical'];
                    if (!validLevels.includes(levelMatch[1])) {
                        const line = this.findLineContaining(document, 'security_level:');
                        const range = new vscode.Range(line, 0, line, 100);
                        const diagnostic = new vscode.Diagnostic(
                            range,
                            `Invalid security_level. Must be one of: ${validLevels.join(', ')}`,
                            vscode.DiagnosticSeverity.Error
                        );
                        diagnostic.code = 'NSC003';
                        diagnostics.push(diagnostic);
                    }
                }
            }
        }

        // Check for Procedures section
        if (!text.includes('## Procedures') && !text.includes('## procedures')) {
            const line = document.lineCount - 1;
            const range = new vscode.Range(line, 0, line, 0);
            const diagnostic = new vscode.Diagnostic(
                range,
                'Missing Procedures section. Skills should define execution steps.',
                vscode.DiagnosticSeverity.Warning
            );
            diagnostic.code = 'NSC101';
            diagnostics.push(diagnostic);
        }

        this.diagnosticCollection.set(document.uri, diagnostics);
    }

    /**
     * Convert internal diagnostic to VS Code diagnostic
     */
    private convertDiagnostic(diag: Diagnostic, document: vscode.TextDocument): vscode.Diagnostic {
        const line = Math.max(0, diag.line - 1); // Convert to 0-based
        const column = Math.max(0, diag.column - 1);
        
        const range = new vscode.Range(
            new vscode.Position(line, column),
            new vscode.Position(line, column + 50)
        );

        const severity = this.mapSeverity(diag.severity);
        const diagnostic = new vscode.Diagnostic(range, diag.message, severity);

        return diagnostic;
    }

    /**
     * Map severity to VS Code severity
     */
    private mapSeverity(severity: string): vscode.DiagnosticSeverity {
        switch (severity) {
            case 'error':
                return vscode.DiagnosticSeverity.Error;
            case 'warning':
                return vscode.DiagnosticSeverity.Warning;
            case 'info':
                return vscode.DiagnosticSeverity.Information;
            default:
                return vscode.DiagnosticSeverity.Error;
        }
    }

    /**
     * Find a line in the frontmatter section
     */
    private findLineInFrontmatter(document: vscode.TextDocument, startLine: number): number {
        for (let i = startLine; i < document.lineCount; i++) {
            const line = document.lineAt(i);
            if (line.text === '---') {
                return i - 1;
            }
        }
        return startLine;
    }

    /**
     * Find a line containing specific text
     */
    private findLineContaining(document: vscode.TextDocument, text: string): number {
        for (let i = 0; i < document.lineCount; i++) {
            if (document.lineAt(i).text.includes(text)) {
                return i;
            }
        }
        return 0;
    }

    /**
     * Clear diagnostics for a document
     */
    clearDiagnostics(uri: vscode.Uri): void {
        this.diagnosticCollection.delete(uri);
    }

    /**
     * Dispose
     */
    dispose(): void {
        if (this.debounceTimer) {
            clearTimeout(this.debounceTimer);
        }
        this.diagnosticCollection.dispose();
    }
}