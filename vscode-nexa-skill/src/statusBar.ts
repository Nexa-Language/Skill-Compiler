/**
 * Status Bar Manager - Show NSC status in the status bar
 */

import * as vscode from 'vscode';

export class StatusBarManager implements vscode.Disposable {
    private statusBarItem: vscode.StatusBarItem;

    constructor() {
        this.statusBarItem = vscode.window.createStatusBarItem(
            vscode.StatusBarAlignment.Right,
            100
        );
        this.statusBarItem.command = 'nsc.showOutput';
        this.setIdle();
    }

    show(): void {
        this.statusBarItem.show();
    }

    hide(): void {
        this.statusBarItem.hide();
    }

    setIdle(): void {
        this.statusBarItem.text = '$(gear) NSC';
        this.statusBarItem.tooltip = 'Nexa Skill Compiler - Ready';
        this.statusBarItem.backgroundColor = undefined;
    }

    setCompiling(): void {
        this.statusBarItem.text = '$(sync~spin) NSC Compiling...';
        this.statusBarItem.tooltip = 'Compiling skill...';
        this.statusBarItem.backgroundColor = undefined;
    }

    setSuccess(): void {
        this.statusBarItem.text = '$(check) NSC';
        this.statusBarItem.tooltip = 'Compilation successful';
        this.statusBarItem.backgroundColor = undefined;
        
        // Reset to idle after 3 seconds
        setTimeout(() => this.setIdle(), 3000);
    }

    setError(): void {
        this.statusBarItem.text = '$(error) NSC Error';
        this.statusBarItem.tooltip = 'Compilation failed - click for details';
        this.statusBarItem.backgroundColor = new vscode.ThemeColor('statusBarItem.errorBackground');
        
        // Reset to idle after 5 seconds
        setTimeout(() => this.setIdle(), 5000);
    }

    setWarning(): void {
        this.statusBarItem.text = '$(warning) NSC Warning';
        this.statusBarItem.tooltip = 'Compilation completed with warnings';
        this.statusBarItem.backgroundColor = new vscode.ThemeColor('statusBarItem.warningBackground');
        
        // Reset to idle after 4 seconds
        setTimeout(() => this.setIdle(), 4000);
    }

    setVersion(version: string): void {
        this.statusBarItem.text = `$(gear) NSC v${version}`;
        this.statusBarItem.tooltip = `Nexa Skill Compiler v${version} - Ready`;
    }

    dispose(): void {
        this.statusBarItem.dispose();
    }
}