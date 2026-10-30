import * as vscode from 'vscode';
import type { CoreClient } from '../core/client';
import type { NoteSummary } from '../types';
import type { WorkspaceManager } from '../workspace/workspaceManager';

export class NoteItem extends vscode.TreeItem {
  constructor(
    public readonly note: NoteSummary,
    public readonly projectRoot: string
  ) {
    super(note.title, vscode.TreeItemCollapsibleState.None);

    this.id = note.id;
    this.description = note.tags.length > 0 ? note.tags.map((t) => `#${t}`).join(' ') : undefined;
    this.tooltip = `Note: ${note.title}\nID: ${note.id}\nUpdated: ${note.updated_at}\nTags: ${note.tags.join(', ') || 'none'}`;
    this.accessibilityInformation = {
      label: `Developer Note: ${note.title}${note.is_pinned ? ', pinned' : ''}${note.tags.length > 0 ? ', tags ' + note.tags.join(', ') : ''}`,
      role: 'treeitem',
    };

    if (note.is_pinned) {
      this.iconPath = new vscode.ThemeIcon('pin');
    } else if (note.is_archived) {
      this.iconPath = new vscode.ThemeIcon('archive');
    } else {
      this.iconPath = new vscode.ThemeIcon('note');
    }

    this.contextValue = 'devcontext.note';

    this.command = {
      command: 'devcontext.showNote',
      title: 'Show Note Details',
      arguments: [note.id, projectRoot],
    };
  }
}

/**
 * TreeDataProvider displaying authored developer notes.
 */
export class NotesTreeDataProvider implements vscode.TreeDataProvider<NoteItem> {
  private readonly _onDidChangeTreeData = new vscode.EventEmitter<NoteItem | undefined | null | void>();
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

  public getTreeItem(element: NoteItem): vscode.TreeItem {
    return element;
  }

  public async getChildren(element?: NoteItem): Promise<NoteItem[]> {
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

    const items: NoteItem[] = [];

    for (const project of initialized) {
      try {
        const notes = await client.listNotes(project.uri.fsPath);
        for (const note of notes) {
          items.push(new NoteItem(note, project.uri.fsPath));
        }
      } catch {
        // Ignored or handled via overview view
      }
    }

    return items;
  }
}
