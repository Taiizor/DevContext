import * as vscode from 'vscode';
import type { CoreClient } from '../core/client';
import type { DecisionSummary } from '../types';
import type { WorkspaceManager } from '../workspace/workspaceManager';

export class DecisionItem extends vscode.TreeItem {
  constructor(
    public readonly decision: DecisionSummary,
    public readonly projectRoot: string
  ) {
    super(decision.title, vscode.TreeItemCollapsibleState.None);

    this.id = decision.id;
    this.description = `[${decision.status.toUpperCase()}]`;

    let statusTip = `Status: ${decision.status}`;
    if (decision.superseded_by) {
      statusTip += ` (Superseded by: ${decision.superseded_by})`;
    }

    this.tooltip = `Architecture Decision: ${decision.title}\nID: ${decision.id}\n${statusTip}\nCreated: ${decision.created_at}\nTags: ${decision.tags.join(', ') || 'none'}`;

    switch (decision.status) {
      case 'accepted':
        this.iconPath = new vscode.ThemeIcon('pass', new vscode.ThemeColor('testing.iconPassed'));
        break;
      case 'proposed':
        this.iconPath = new vscode.ThemeIcon('question', new vscode.ThemeColor('editorInfo.foreground'));
        break;
      case 'superseded':
        this.iconPath = new vscode.ThemeIcon('history', new vscode.ThemeColor('disabledForeground'));
        break;
      case 'rejected':
        this.iconPath = new vscode.ThemeIcon('error', new vscode.ThemeColor('testing.iconFailed'));
        break;
      case 'deprecated':
        this.iconPath = new vscode.ThemeIcon('warning', new vscode.ThemeColor('editorWarning.foreground'));
        break;
      default:
        this.iconPath = new vscode.ThemeIcon('law');
    }

    this.contextValue = 'devcontext.decision';

    this.command = {
      command: 'devcontext.showDecision',
      title: 'Show Decision Details',
      arguments: [decision.id, projectRoot],
    };
  }
}

/**
 * TreeDataProvider displaying Architecture Decision Records (ADRs).
 */
export class DecisionsTreeDataProvider implements vscode.TreeDataProvider<DecisionItem> {
  private readonly _onDidChangeTreeData = new vscode.EventEmitter<DecisionItem | undefined | null | void>();
  public readonly onDidChangeTreeData = this._onDidChangeTreeData.event;

  constructor(
    private readonly workspaceManager: WorkspaceManager,
    private readonly getClient: () => CoreClient | null
  ) {
    this.workspaceManager.onProjectChanged(() => this.refresh());
  }

  public refresh(): void {
    this._onDidChangeTreeData.fire();
  }

  public getTreeItem(element: DecisionItem): vscode.TreeItem {
    return element;
  }

  public async getChildren(element?: DecisionItem): Promise<DecisionItem[]> {
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

    const items: DecisionItem[] = [];

    for (const project of initialized) {
      try {
        const decisions = await client.listDecisions(project.uri.fsPath);
        for (const decision of decisions) {
          items.push(new DecisionItem(decision, project.uri.fsPath));
        }
      } catch {
        // Ignored or handled via overview view
      }
    }

    return items;
  }
}
