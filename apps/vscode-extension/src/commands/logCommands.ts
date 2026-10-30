import * as vscode from 'vscode';
import type { CoreClient } from '../core/client';
import type { WorkspaceManager } from '../workspace/workspaceManager';

export function registerLogCommands(
  context: vscode.ExtensionContext,
  workspaceManager: WorkspaceManager,
  getClient: () => CoreClient | null
): void {
  context.subscriptions.push(
    vscode.commands.registerCommand('devcontext.newLog', async (explicitDir?: string) => {
      const client = getClient();
      if (!client) {
        vscode.window.showErrorMessage('DevContext core is not available.');
        return;
      }

      const projectRoot =
        explicitDir ||
        (await workspaceManager.getTargetProjectRoot('Select workspace for log entry'));
      if (!projectRoot) {
        return;
      }

      if (workspaceManager.isWorkspaceReadOnly(projectRoot)) {
        vscode.window.showErrorMessage('Cannot create logs in a read-only workspace.');
        return;
      }

      const summary = await vscode.window.showInputBox({
        title: 'New Development Log Entry',
        prompt: 'Enter summary of work completed or activity',
        validateInput: (v) => (!v || !v.trim() ? 'Summary cannot be empty.' : null),
      });
      if (!summary) {
        return;
      }

      const details = await vscode.window.showInputBox({
        title: 'New Development Log Entry',
        prompt: 'Enter additional details or context (optional)',
      });

      const category = await vscode.window.showQuickPick(
        [
          { label: 'feature', description: 'New feature implementation' },
          { label: 'bugfix', description: 'Bug fix or regression correction' },
          { label: 'refactor', description: 'Code refactoring or cleanup' },
          { label: 'investigation', description: 'Technical research or spikes' },
          { label: 'general', description: 'General development task' },
        ],
        { placeHolder: 'Select log category' }
      );

      try {
        await client.addLog(
          projectRoot,
          summary,
          details || undefined,
          category?.label || 'general'
        );
        workspaceManager.notifyProjectChanged();
        vscode.window.showInformationMessage(`Log entry saved.`);
      } catch (err) {
        vscode.window.showErrorMessage(`Failed to save log entry: ${(err as Error).message}`);
      }
    })
  );
}
