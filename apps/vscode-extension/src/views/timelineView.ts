import * as vscode from 'vscode';
import type { CoreClient } from '../core/client';
import type { TimelineEvent } from '../types';
import type { WorkspaceManager } from '../workspace/workspaceManager';

export class TimelineTreeItem extends vscode.TreeItem {
  constructor(public readonly event: TimelineEvent) {
    super(event.title, vscode.TreeItemCollapsibleState.None);

    const dateStr = event.timestamp.substring(0, 10);
    this.description = `[${event.event_type.toUpperCase()}] ${dateStr}`;

    const authorStr = event.author ? `\nAuthor: ${event.author}` : '';
    const summaryStr = event.summary ? `\nSummary: ${event.summary}` : '';
    this.tooltip = `${event.title} (${event.event_type})\nTime: ${event.timestamp}${authorStr}\nID: ${event.identifier}${summaryStr}`;
    this.accessibilityInformation = {
      label: `Timeline Event: ${event.event_type}, ${event.title}, date: ${dateStr}${event.author ? ', author: ' + event.author : ''}`,
      role: 'treeitem',
    };

    switch (event.event_type) {
      case 'commit':
        this.iconPath = new vscode.ThemeIcon('git-commit');
        break;
      case 'note':
        this.iconPath = new vscode.ThemeIcon('note');
        break;
      case 'decision':
        this.iconPath = new vscode.ThemeIcon('law');
        break;
      case 'milestone':
        this.iconPath = new vscode.ThemeIcon('milestone');
        break;
      case 'log_entry':
        this.iconPath = new vscode.ThemeIcon('output');
        break;
      default:
        this.iconPath = new vscode.ThemeIcon('circle-outline');
    }
  }
}

/**
 * TreeDataProvider displaying the unified development timeline.
 */
export class TimelineTreeDataProvider implements vscode.TreeDataProvider<TimelineTreeItem> {
  private readonly _onDidChangeTreeData = new vscode.EventEmitter<TimelineTreeItem | undefined | null | void>();
  public readonly onDidChangeTreeData = this._onDidChangeTreeData.event;
  private currentFilterType: string | undefined = undefined;

  constructor(
    private readonly workspaceManager: WorkspaceManager,
    private readonly getClient: () => CoreClient | null
  ) {
    this.workspaceManager.onProjectChanged(() => this.refresh());
  }

  public setFilterType(type?: string): void {
    this.currentFilterType = type;
    this.refresh();
  }

  public refresh(): void {
    this._onDidChangeTreeData.fire();
  }

  public getTreeItem(element: TimelineTreeItem): vscode.TreeItem {
    return element;
  }

  public async getChildren(element?: TimelineTreeItem): Promise<TimelineTreeItem[]> {
    if (element) {
      return [];
    }

    const client = this.getClient();
    if (!client) {
      return [];
    }

    const projects = this.workspaceManager.getWorkspaceProjects();
    const initialized = projects.filter((p) => p.isInitialized);
    if (initialized.length === 0) {
      return [];
    }

    const items: TimelineTreeItem[] = [];

    for (const project of initialized) {
      try {
        const timelineView = await client.timeline(project.uri.fsPath, {
          limit: 50,
          eventType: this.currentFilterType,
        });

        for (const ev of timelineView.events) {
          items.push(new TimelineTreeItem(ev));
        }
      } catch {
        // Ignored or handled via overview view
      }
    }

    return items;
  }
}
