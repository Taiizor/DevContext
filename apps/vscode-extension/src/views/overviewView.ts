import * as vscode from 'vscode';
import type { CoreClient } from '../core/client';
import type { HandshakeResult } from '../types';
import type { WorkspaceManager } from '../workspace/workspaceManager';

export class OverviewItem extends vscode.TreeItem {
  constructor(
    label: string,
    description?: string,
    collapsibleState: vscode.TreeItemCollapsibleState = vscode.TreeItemCollapsibleState.None,
    command?: vscode.Command
  ) {
    super(label, collapsibleState);
    this.description = description;
    this.command = command;
  }
}

/**
 * TreeDataProvider displaying project overview, identity, database health, and stats.
 */
export class OverviewTreeDataProvider implements vscode.TreeDataProvider<OverviewItem> {
  private readonly _onDidChangeTreeData = new vscode.EventEmitter<OverviewItem | undefined | null | void>();
  public readonly onDidChangeTreeData = this._onDidChangeTreeData.event;

  constructor(
    private readonly workspaceManager: WorkspaceManager,
    private readonly getClient: () => CoreClient | null,
    private readonly getHandshake: () => HandshakeResult | null
  ) {
    this.workspaceManager.onProjectChanged(() => this.refresh());
  }

  public refresh(): void {
    this._onDidChangeTreeData.fire();
  }

  public getTreeItem(element: OverviewItem): vscode.TreeItem {
    return element;
  }

  public async getChildren(element?: OverviewItem): Promise<OverviewItem[]> {
    if (element) {
      return [];
    }

    const handshake = this.getHandshake();
    if (!handshake || handshake.status !== 'ready') {
      const errorItem = new OverviewItem(
        'DevContext Core Inactive',
        handshake?.status || 'Unknown State'
      );
      errorItem.iconPath = new vscode.ThemeIcon('warning');
      errorItem.tooltip = handshake?.message || 'Core executable not connected.';
      errorItem.command = {
        command: 'devcontext.doctor',
        title: 'Run Diagnostic Doctor',
      };
      return [errorItem];
    }

    const projects = this.workspaceManager.getWorkspaceProjects();
    if (projects.length === 0) {
      const item = new OverviewItem('No Workspace Open', 'Open a folder to start');
      item.iconPath = new vscode.ThemeIcon('folder');
      return [item];
    }

    const client = this.getClient();
    if (!client) {
      return [];
    }

    const items: OverviewItem[] = [];

    for (const project of projects) {
      if (!project.isInitialized) {
        const uninitItem = new OverviewItem(
          project.name,
          'Not Initialized — Click to setup'
        );
        uninitItem.iconPath = new vscode.ThemeIcon('add');
        uninitItem.tooltip = `Initialize DevContext in ${project.uri.fsPath}`;
        uninitItem.command = {
          command: 'devcontext.init',
          title: 'Initialize DevContext',
          arguments: [project.uri.fsPath],
        };
        items.push(uninitItem);
        continue;
      }

      try {
        const status = await client.status(project.uri.fsPath);

        const projectHeader = new OverviewItem(
          status.name,
          project.isReadOnly ? '(Read-Only)' : `v${handshake.cliVersion}`
        );
        projectHeader.iconPath = new vscode.ThemeIcon('repo');
        projectHeader.tooltip = `Root: ${status.root_path}\nID: ${status.project_id}\nCreated: ${status.created_at}`;
        items.push(projectHeader);

        const notesItem = new OverviewItem(
          'Notes',
          `${status.stats.notes_count} authored notes`
        );
        notesItem.iconPath = new vscode.ThemeIcon('note');
        items.push(notesItem);

        const decisionsItem = new OverviewItem(
          'Architecture Decisions',
          `${status.stats.decisions_count} ADRs recorded`
        );
        decisionsItem.iconPath = new vscode.ThemeIcon('law');
        items.push(decisionsItem);

        const milestonesItem = new OverviewItem(
          'Milestones',
          `${status.stats.milestones_count} target milestones`
        );
        milestonesItem.iconPath = new vscode.ThemeIcon('milestone');
        items.push(milestonesItem);

        const logsItem = new OverviewItem(
          'Development Logs',
          `${status.stats.log_entries_count} entries logged`
        );
        logsItem.iconPath = new vscode.ThemeIcon('output');
        items.push(logsItem);

        if (status.vcs_identifier) {
          const vcsItem = new OverviewItem('Git Context', status.vcs_identifier);
          vcsItem.iconPath = new vscode.ThemeIcon('git-branch');
          items.push(vcsItem);
        }
      } catch (err) {
        const failItem = new OverviewItem(
          project.name,
          `Error: ${(err as Error).message}`
        );
        failItem.iconPath = new vscode.ThemeIcon('error');
        items.push(failItem);
      }
    }

    return items;
  }
}
