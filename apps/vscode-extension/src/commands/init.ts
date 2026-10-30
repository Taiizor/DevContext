import * as path from 'node:path';
import * as vscode from 'vscode';
import type { CoreClient } from '../core/client';
import type { WorkspaceManager } from '../workspace/workspaceManager';

export function registerInitCommand(
  context: vscode.ExtensionContext,
  workspaceManager: WorkspaceManager,
  getClient: () => CoreClient | null
): void {
  const disposable = vscode.commands.registerCommand(
    'devcontext.init',
    async (explicitDir?: string) => {
      const client = getClient();
      if (!client) {
        vscode.window.showErrorMessage(
          'DevContext core executable is not available. Please check settings.'
        );
        return;
      }

      const projectRoot =
        explicitDir ||
        (await workspaceManager.getTargetProjectRoot(
          'Select folder to initialize DevContext project'
        ));

      if (!projectRoot) {
        vscode.window.showWarningMessage('No workspace folder selected for initialization.');
        return;
      }

      if (workspaceManager.isWorkspaceReadOnly(projectRoot)) {
        vscode.window.showErrorMessage(
          'Cannot initialize DevContext in a read-only workspace folder.'
        );
        return;
      }

      const defaultName = path.basename(projectRoot);
      const projectName = await vscode.window.showInputBox({
        title: 'Initialize DevContext Project',
        prompt: 'Enter a project name',
        value: defaultName,
        validateInput: (value) => {
          if (!value || value.trim().length === 0) {
            return 'Project name cannot be empty.';
          }
          return null;
        },
      });

      if (!projectName) {
        return; // User cancelled
      }

      try {
        await vscode.window.withProgress(
          {
            location: vscode.ProgressLocation.Notification,
            title: `Initializing DevContext project '${projectName}'...`,
            cancellable: false,
          },
          async () => {
            await client.init(projectRoot, projectName);
          }
        );

        workspaceManager.notifyProjectChanged();

        const action = await vscode.window.showInformationMessage(
          `DevContext project '${projectName}' initialized successfully!`,
          'Create First Note',
          'Dismiss'
        );

        if (action === 'Create First Note') {
          vscode.commands.executeCommand('devcontext.newNote', projectRoot);
        }
      } catch (err) {
        vscode.window.showErrorMessage(
          `Initialization failed: ${(err as Error).message}`
        );
      }
    }
  );

  context.subscriptions.push(disposable);
}
