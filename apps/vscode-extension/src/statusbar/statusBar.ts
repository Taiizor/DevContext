import * as vscode from 'vscode';
import type { CoreClient } from '../core/client';
import type { HandshakeResult } from '../types';
import type { WorkspaceManager } from '../workspace/workspaceManager';

export class DevContextStatusBar {
  private readonly statusBarItem: vscode.StatusBarItem;

  constructor(
    private readonly workspaceManager: WorkspaceManager,
    private readonly getClient: () => CoreClient | null,
    private readonly getHandshake: () => HandshakeResult | null
  ) {
    this.statusBarItem = vscode.window.createStatusBarItem(
      vscode.StatusBarAlignment.Right,
      100
    );

    this.workspaceManager.onProjectChanged(() => this.update());
    this.update();
  }

  public async update(): Promise<void> {
    const handshake = this.getHandshake();

    if (!handshake || handshake.status !== 'ready') {
      this.statusBarItem.text = '$(warning) DevContext: Core Inactive';
      this.statusBarItem.tooltip = handshake?.message || 'DevContext core executable missing';
      this.statusBarItem.command = 'devcontext.doctor';
      this.statusBarItem.backgroundColor = new vscode.ThemeColor(
        'statusBarItem.warningBackground'
      );
      this.statusBarItem.show();
      return;
    }

    this.statusBarItem.backgroundColor = undefined;

    const projects = this.workspaceManager.getWorkspaceProjects();
    if (projects.length === 0) {
      this.statusBarItem.hide();
      return;
    }

    const client = this.getClient();
    if (!client) {
      this.statusBarItem.hide();
      return;
    }

    const initialized = projects.filter((p) => p.isInitialized);
    if (initialized.length === 0) {
      this.statusBarItem.text = '$(book) DevContext: Initialize';
      this.statusBarItem.tooltip = 'Click to initialize DevContext in workspace';
      this.statusBarItem.command = 'devcontext.init';
      this.statusBarItem.show();
      return;
    }

    const firstInit = initialized[0];
    if (!firstInit) {
      this.statusBarItem.hide();
      return;
    }

    try {
      const status = await client.status(firstInit.uri.fsPath);
      this.statusBarItem.text = `$(book) DevContext: ${status.name}`;
      this.statusBarItem.tooltip = `DevContext: ${status.name}\nNotes: ${status.stats.notes_count} | Decisions: ${status.stats.decisions_count}\nClick to search context`;
      this.statusBarItem.command = 'devcontext.search';
      this.statusBarItem.show();
    } catch {
      this.statusBarItem.text = `$(book) DevContext: Ready`;
      this.statusBarItem.tooltip = 'DevContext project active';
      this.statusBarItem.command = 'devcontext.search';
      this.statusBarItem.show();
    }
  }

  public dispose(): void {
    this.statusBarItem.dispose();
  }
}
