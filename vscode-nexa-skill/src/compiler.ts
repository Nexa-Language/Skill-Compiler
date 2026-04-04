/**
 * Skill Compiler - Interface to the NSC CLI
 */

import * as vscode from 'vscode';
import { exec, spawn } from 'child_process';
import { promisify } from 'util';

const execAsync = promisify(exec);

export interface CompileOptions {
    target: 'claude' | 'codex' | 'gemini' | 'all';
    outputDir: string;
}

export interface CompileResult {
    success: boolean;
    outputPath?: string;
    error?: string;
}

export interface ValidationResult {
    valid: boolean;
    errors: Diagnostic[];
}

export interface Diagnostic {
    severity: 'error' | 'warning' | 'info';
    message: string;
    line: number;
    column: number;
}

export interface CheckResult {
    diagnostics: Diagnostic[];
}

export class SkillCompiler {
    private compilerPath: string = 'nsc';

    constructor() {
        this.updateCompilerPath();
        
        // Listen for configuration changes
        vscode.workspace.onDidChangeConfiguration((e) => {
            if (e.affectsConfiguration('nsc.compilerPath')) {
                this.updateCompilerPath();
            }
        });
    }

    private updateCompilerPath(): void {
        const config = vscode.workspace.getConfiguration('nsc');
        this.compilerPath = config.get<string>('compilerPath') || 'nsc';
    }

    /**
     * Compile a skill file
     */
    async compile(filePath: string, options: CompileOptions): Promise<CompileResult> {
        const args = this.buildCompileArgs(filePath, options);

        try {
            const { stdout, stderr } = await execAsync(
                `"${this.compilerPath}" ${args.join(' ')}`,
                {
                    cwd: vscode.workspace.workspaceFolders?.[0]?.uri.fsPath,
                    maxBuffer: 1024 * 1024 * 10 // 10MB buffer
                }
            );

            // Parse output path from stdout
            const outputMatch = stdout.match(/Output written to: (.+)/);
            const outputPath = outputMatch ? outputMatch[1].trim() : undefined;

            return {
                success: true,
                outputPath
            };
        } catch (error: unknown) {
            const execError = error as { stderr?: string; message?: string };
            return {
                success: false,
                error: execError.stderr || execError.message || 'Unknown error'
            };
        }
    }

    /**
     * Validate a skill file
     */
    async validate(filePath: string): Promise<ValidationResult> {
        const args = ['validate', filePath, '--format', 'json'];

        try {
            const { stdout } = await execAsync(
                `"${this.compilerPath}" ${args.join(' ')}`,
                {
                    cwd: vscode.workspace.workspaceFolders?.[0]?.uri.fsPath
                }
            );

            const result = JSON.parse(stdout);
            return {
                valid: result.valid === true,
                errors: result.errors || []
            };
        } catch (error: unknown) {
            const execError = error as { stderr?: string; message?: string };
            // Try to parse error output
            try {
                const errorOutput = execError.stderr || '';
                const errorResult = JSON.parse(errorOutput);
                return {
                    valid: false,
                    errors: errorResult.errors || [{ 
                        severity: 'error', 
                        message: errorOutput, 
                        line: 1, 
                        column: 1 
                    }]
                };
            } catch {
                return {
                    valid: false,
                    errors: [{ 
                        severity: 'error', 
                        message: execError.message || 'Validation failed', 
                        line: 1, 
                        column: 1 
                    }]
                };
            }
        }
    }

    /**
     * Check a skill file
     */
    async check(filePath: string): Promise<CheckResult> {
        const args = ['check', filePath, '--format', 'json'];

        try {
            const { stdout } = await execAsync(
                `"${this.compilerPath}" ${args.join(' ')}`,
                {
                    cwd: vscode.workspace.workspaceFolders?.[0]?.uri.fsPath
                }
            );

            const result = JSON.parse(stdout);
            return {
                diagnostics: result.diagnostics || []
            };
        } catch (error: unknown) {
            const execError = error as { stderr?: string; message?: string };
            try {
                const errorOutput = execError.stderr || '';
                const errorResult = JSON.parse(errorOutput);
                return {
                    diagnostics: errorResult.diagnostics || []
                };
            } catch {
                return {
                    diagnostics: [{ 
                        severity: 'error', 
                        message: execError.message || 'Check failed', 
                        line: 1, 
                        column: 1 
                    }]
                };
            }
        }
    }

    /**
     * Build compile arguments
     */
    private buildCompileArgs(filePath: string, options: CompileOptions): string[] {
        const args = ['build', filePath];

        if (options.target === 'all') {
            args.push('--target', 'all');
        } else {
            args.push('--target', options.target);
        }

        args.push('--output', options.outputDir);

        return args;
    }

    /**
     * Check if compiler is available
     */
    async isAvailable(): Promise<boolean> {
        try {
            await execAsync(`"${this.compilerPath}" --version`);
            return true;
        } catch {
            return false;
        }
    }

    /**
     * Get compiler version
     */
    async getVersion(): Promise<string | null> {
        try {
            const { stdout } = await execAsync(`"${this.compilerPath}" --version`);
            const versionMatch = stdout.match(/nsc\s+(\d+\.\d+\.\d+)/);
            return versionMatch ? versionMatch[1] : null;
        } catch {
            return null;
        }
    }
}