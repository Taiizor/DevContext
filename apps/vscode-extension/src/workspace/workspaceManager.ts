import * as fs from 'node:fs';
import * as path from 'node:path';
import * as vscode from 'vscode';

/**
 * Information regarding a detected workspace root.
 */
export interface WorkspaceProjectInfo {
  uri: vscode.Uri;
  name: string;
  isInitialized: boolean;
  isReadOnly: boolean;
}

/**
 * Manages workspace folder discovery, multi-root isolation, and active project routing.
 */
export class WorkspaceManager {
  private readonly onProjectChangedEmitter = new vscode.EventEmitter<void>();
  public readonly onProjectChanged = this.onProjectChangedEmitter.event;

  constructor() {
    vscode.workspace.onDidChangeWorkspaceFolders(() => {
      this.onProjectChangedEmitter.fire();
    });
  }

  /**
   * Retrieves all detected workspace roots and their DevContext initialization state.
   */
  public getWorkspaceProjects(): WorkspaceProjectInfo[] {
    const folders = vscode.workspace.workspaceFolders || [];
    return folders.map((folder) => {
      const devcontextDir = path.join(folder.uri.fsPath, '.devcontext');
      const isInitialized =
        fs.existsSync(devcontextDir) &&
        fs.existsSync(path.join(devcontextDir, 'devcontext.db'));

      const isReadOnly = this.isWorkspaceReadOnly(folder.uri.fsPath);

      return {
        uri: folder.uri,
        name: folder.name,
        isInitialized,
        isReadOnly,
      };
    });
  }

  /**
   * Resolves the target project root path for a command.
   * If a document is active in an editor, selects its containing folder.
   * If only one workspace folder exists, returns it.
   * If multiple exist and none is active, prompts the user via QuickPick.
   */
  public async getTargetProjectRoot(
    promptTitle = 'Select Target DevContext Workspace'
  ): Promise<string | null> {
    const folders = vscode.workspace.workspaceFolders;
    if (!folders || folders.length === 0) {
      return null;
    }

    // 1. If an active editor belongs to a workspace folder, use that folder
    const activeEditor = vscode.window.activeTextEditor;
    if (activeEditor) {
      const activeFolder = vscode.workspace.getWorkspaceFolder(activeEditor.document.uri);
      if (activeFolder) {
        return activeFolder.uri.fsPath;
      }
    }

    // 2. If single-root workspace, return directly
    if (folders.length === 1) {
      const firstFolder = folders[0];
      return firstFolder ? firstFolder.uri.fsPath : null;
    }

    // 3. Multi-root workspace: prompt user to choose the folder
    const items = folders.map((f) => {
      const isInit = fs.existsSync(path.join(f.uri.fsPath, '.devcontext'));
      return {
        label: f.name,
        description: isInit ? '(Initialized)' : '(Not Initialized)',
        detail: f.uri.fsPath,
        folderPath: f.uri.fsPath,
      };
    });

    const selected = await vscode.window.showQuickPick(items, {
      placeHolder: promptTitle,
    });

    return selected ? selected.folderPath : null;
  }

  /**
   * Checks if a workspace directory is read-only.
   */
  public isWorkspaceReadOnly(dirPath: string): boolean {
    if (vscode.workspace.isTrusted === false) {
      return true;
    }

    try {
      fs.accessSync(dirPath, fs.constants.W_OK);
      return false;
    } catch {
      return true;
    }
  }

  public notifyProjectChanged(): void {
    this.onProjectChangedEmitter.fire();
  }

  public dispose(): void {
    this.onProjectChangedEmitter.dispose();
  }
}
