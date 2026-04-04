/**
 * Skill Preview Panel - Show compiled output preview
 */

import * as vscode from 'vscode';
import { SkillCompiler } from './compiler';

export class SkillPreviewPanel {
    public static currentPanel: SkillPreviewPanel | undefined;
    public static readonly viewType = 'nscPreview';

    private readonly _panel: vscode.WebviewPanel;
    private readonly _extensionUri: vscode.Uri;
    private _disposables: vscode.Disposable[] = [];
    private _compiler: SkillCompiler;

    public static createOrShow(extensionUri: vscode.Uri, document: vscode.TextDocument): void {
        const column = vscode.window.activeTextEditor
            ? vscode.window.activeTextEditor.viewColumn
            : undefined;

        // If we already have a panel, show it
        if (SkillPreviewPanel.currentPanel) {
            SkillPreviewPanel.currentPanel._panel.reveal(column);
            SkillPreviewPanel.currentPanel._update(document);
            return;
        }

        // Create a new panel
        const panel = vscode.window.createWebviewPanel(
            SkillPreviewPanel.viewType,
            'NSC Preview',
            column || vscode.ViewColumn.Two,
            {
                enableScripts: true,
                retainContextWhenHidden: true
            }
        );

        SkillPreviewPanel.currentPanel = new SkillPreviewPanel(panel, extensionUri, document);
    }

    private constructor(
        panel: vscode.WebviewPanel,
        extensionUri: vscode.Uri,
        document: vscode.TextDocument
    ) {
        this._panel = panel;
        this._extensionUri = extensionUri;
        this._compiler = new SkillCompiler();

        // Set the webview's initial html content
        this._update(document);

        // Listen for when the panel is disposed
        this._panel.onDidDispose(() => this.dispose(), null, this._disposables);

        // Handle messages from the webview
        this._panel.webview.onDidReceiveMessage(
            message => {
                switch (message.command) {
                    case 'compile':
                        this._handleCompile(document);
                        break;
                    case 'copy':
                        vscode.env.clipboard.writeText(message.text);
                        vscode.window.showInformationMessage('Copied to clipboard');
                        break;
                }
            },
            null,
            this._disposables
        );
    }

    private async _update(document: vscode.TextDocument): Promise<void> {
        const content = document.getText();
        this._panel.webview.html = this._getHtmlForWebview(content, document.uri.fsPath);
    }

    private async _handleCompile(document: vscode.TextDocument): Promise<void> {
        const config = vscode.workspace.getConfiguration('nsc');
        const target = config.get<string>('defaultTarget') || 'all';
        const outputDir = config.get<string>('outputDirectory') || 'dist';

        try {
            const result = await this._compiler.compile(document.uri.fsPath, {
                target: target as 'claude' | 'codex' | 'gemini' | 'all',
                outputDir
            });

            if (result.success) {
                this._panel.webview.postMessage({
                    command: 'compileResult',
                    success: true,
                    message: `Compiled successfully to ${result.outputPath}`
                });
            } else {
                this._panel.webview.postMessage({
                    command: 'compileResult',
                    success: false,
                    message: result.error
                });
            }
        } catch (error) {
            this._panel.webview.postMessage({
                command: 'compileResult',
                success: false,
                message: error instanceof Error ? error.message : String(error)
            });
        }
    }

    private _getHtmlForWebview(content: string, filePath: string): string {
        const fileName = filePath.split('/').pop() || 'SKILL.md';
        
        // Parse frontmatter
        const frontmatterMatch = content.match(/^---\n([\s\S]*?)\n---/);
        const frontmatter = frontmatterMatch ? this._parseYaml(frontmatterMatch[1]) : {};
        const body = frontmatterMatch ? content.slice(frontmatterMatch[0].length).trim() : content;

        return `<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>NSC Preview - ${fileName}</title>
    <style>
        * {
            box-sizing: border-box;
            margin: 0;
            padding: 0;
        }
        
        body {
            font-family: var(--vscode-font-family);
            background-color: var(--vscode-editor-background);
            color: var(--vscode-editor-foreground);
            padding: 16px;
        }
        
        .header {
            display: flex;
            justify-content: space-between;
            align-items: center;
            margin-bottom: 16px;
            padding-bottom: 12px;
            border-bottom: 1px solid var(--vscode-panel-border);
        }
        
        .file-name {
            font-size: 14px;
            font-weight: 600;
            color: var(--vscode-titleBar-activeForeground);
        }
        
        .actions {
            display: flex;
            gap: 8px;
        }
        
        button {
            background-color: var(--vscode-button-background);
            color: var(--vscode-button-foreground);
            border: none;
            padding: 6px 12px;
            border-radius: 4px;
            cursor: pointer;
            font-size: 12px;
        }
        
        button:hover {
            background-color: var(--vscode-button-hoverBackground);
        }
        
        .section {
            margin-bottom: 20px;
        }
        
        .section-title {
            font-size: 12px;
            font-weight: 600;
            text-transform: uppercase;
            letter-spacing: 0.5px;
            color: var(--vscode-descriptionForeground);
            margin-bottom: 8px;
        }
        
        .metadata-grid {
            display: grid;
            grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
            gap: 12px;
        }
        
        .metadata-item {
            background-color: var(--vscode-editor-inactiveSelectionBackground);
            padding: 8px 12px;
            border-radius: 4px;
        }
        
        .metadata-label {
            font-size: 11px;
            color: var(--vscode-descriptionForeground);
            margin-bottom: 2px;
        }
        
        .metadata-value {
            font-size: 13px;
            font-weight: 500;
        }
        
        .tag {
            display: inline-block;
            background-color: var(--vscode-badge-background);
            color: var(--vscode-badge-foreground);
            padding: 2px 8px;
            border-radius: 12px;
            font-size: 11px;
            margin-right: 4px;
        }
        
        .body-preview {
            background-color: var(--vscode-textCodeBlock-background);
            padding: 12px;
            border-radius: 4px;
            font-family: var(--vscode-editor-font-family);
            font-size: 13px;
            line-height: 1.5;
            white-space: pre-wrap;
            overflow-x: auto;
        }
        
        .status {
            padding: 8px 12px;
            border-radius: 4px;
            margin-top: 12px;
            font-size: 12px;
        }
        
        .status.success {
            background-color: rgba(0, 128, 0, 0.2);
            color: #4caf50;
        }
        
        .status.error {
            background-color: rgba(255, 0, 0, 0.2);
            color: #f44336;
        }
        
        .hidden {
            display: none;
        }
    </style>
</head>
<body>
    <div class="header">
        <span class="file-name">${fileName}</span>
        <div class="actions">
            <button onclick="compile()">Compile</button>
            <button onclick="copyBody()">Copy Body</button>
        </div>
    </div>
    
    <div class="section">
        <div class="section-title">Metadata</div>
        <div class="metadata-grid">
            <div class="metadata-item">
                <div class="metadata-label">Name</div>
                <div class="metadata-value">${frontmatter.name || 'Unnamed'}</div>
            </div>
            <div class="metadata-item">
                <div class="metadata-label">Version</div>
                <div class="metadata-value">${frontmatter.version || '1.0.0'}</div>
            </div>
            <div class="metadata-item">
                <div class="metadata-label">Security Level</div>
                <div class="metadata-value">${frontmatter.security_level || 'medium'}</div>
            </div>
            <div class="metadata-item">
                <div class="metadata-label">HITL Required</div>
                <div class="metadata-value">${frontmatter.hitl_required || false}</div>
            </div>
        </div>
    </div>
    
    ${frontmatter.description ? `
    <div class="section">
        <div class="section-title">Description</div>
        <div class="metadata-value">${frontmatter.description}</div>
    </div>
    ` : ''}
    
    ${frontmatter.compatibility ? `
    <div class="section">
        <div class="section-title">Compatibility</div>
        <div>
            ${(Array.isArray(frontmatter.compatibility) ? frontmatter.compatibility : [frontmatter.compatibility])
                .map((c: string) => `<span class="tag">${c}</span>`).join('')}
        </div>
    </div>
    ` : ''}
    
    <div class="section">
        <div class="section-title">Body Preview</div>
        <div class="body-preview">${this._escapeHtml(body.slice(0, 2000))}${body.length > 2000 ? '\n...(truncated)' : ''}</div>
    </div>
    
    <div id="status" class="status hidden"></div>
    
    <script>
        const vscode = acquireVsCodeApi();
        
        function compile() {
            vscode.postMessage({ command: 'compile' });
        }
        
        function copyBody() {
            vscode.postMessage({ 
                command: 'copy', 
                text: \`${this._escapeJs(body)}\`
            });
        }
        
        window.addEventListener('message', event => {
            const message = event.data;
            const status = document.getElementById('status');
            
            switch (message.command) {
                case 'compileResult':
                    status.textContent = message.message;
                    status.className = 'status ' + (message.success ? 'success' : 'error');
                    break;
            }
        });
    </script>
</body>
</html>`;
    }

    private _parseYaml(yaml: string): Record<string, unknown> {
        const result: Record<string, unknown> = {};
        const lines = yaml.split('\n');
        
        for (const line of lines) {
            const match = line.match(/^(\w+):\s*(.+)$/);
            if (match) {
                const [, key, value] = match;
                // Handle arrays
                if (value.startsWith('[') && value.endsWith(']')) {
                    result[key] = value.slice(1, -1).split(',').map(s => s.trim());
                } else if (value === 'true') {
                    result[key] = true;
                } else if (value === 'false') {
                    result[key] = false;
                } else {
                    result[key] = value;
                }
            }
        }
        
        return result;
    }

    private _escapeHtml(text: string): string {
        return text
            .replace(/&/g, '&')
            .replace(/</g, '<')
            .replace(/>/g, '>')
            .replace(/"/g, '"')
            .replace(/'/g, '&#039;');
    }

    private _escapeJs(text: string): string {
        return text
            .replace(/\\/g, '\\\\')
            .replace(/`/g, '\\`')
            .replace(/\$/g, '\\$');
    }

    public dispose(): void {
        SkillPreviewPanel.currentPanel = undefined;

        this._panel.dispose();

        while (this._disposables.length) {
            const x = this._disposables.pop();
            if (x) {
                x.dispose();
            }
        }
    }
}