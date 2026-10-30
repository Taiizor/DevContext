import * as vscode from 'vscode';
import type { CoreClient } from '../core/client';
import type { WorkspaceManager } from '../workspace/workspaceManager';

export function registerMilestoneCommands(
  context: vscode.ExtensionContext,
  workspaceManager: WorkspaceManager,
  getClient: () => CoreClient | null
): void {
  // 1. New Milestone
  context.subscriptions.push(
    vscode.commands.registerCommand('devcontext.newMilestone', async (explicitDir?: string) => {
      const client = getClient();
      if (!client) {
        vscode.window.showErrorMessage('DevContext core is not available.');
        return;
      }

      const projectRoot =
        explicitDir ||
        (await workspaceManager.getTargetProjectRoot('Select workspace for new milestone'));
      if (!projectRoot) {
        return;
      }

      if (workspaceManager.isWorkspaceReadOnly(projectRoot)) {
        vscode.window.showErrorMessage('Cannot create milestones in a read-only workspace.');
        return;
      }

      const title = await vscode.window.showInputBox({
        title: 'New Project Milestone',
        prompt: 'Enter milestone title (e.g., MVP Core Release)',
        validateInput: (v) => (!v || !v.trim() ? 'Title cannot be empty.' : null),
      });
      if (!title) {
        return;
      }

      const description = await vscode.window.showInputBox({
        title: 'New Project Milestone',
        prompt: 'Enter milestone description (optional)',
      });

      const targetDate = await vscode.window.showInputBox({
        title: 'New Project Milestone',
        prompt: 'Enter target completion date (YYYY-MM-DD, optional)',
        validateInput: (v) => {
          if (!v) {
            return null;
          }
          if (!/^\d{4}-\d{2}-\d{2}$/.test(v)) {
            return 'Format must be YYYY-MM-DD';
          }
          return null;
        },
      });

      try {
        await client.addMilestone(
          projectRoot,
          title,
          description || undefined,
          targetDate || undefined
        );
        workspaceManager.notifyProjectChanged();
        vscode.window.showInformationMessage(`Milestone '${title}' created.`);
      } catch (err) {
        vscode.window.showErrorMessage(`Failed to create milestone: ${(err as Error).message}`);
      }
    })
  );

  // 2. Complete Milestone
  context.subscriptions.push(
    vscode.commands.registerCommand('devcontext.completeMilestone', async () => {
      const client = getClient();
      if (!client) {
        return;
      }

      const projectRoot = await workspaceManager.getTargetProjectRoot();
      if (!projectRoot) {
        return;
      }

      if (workspaceManager.isWorkspaceReadOnly(projectRoot)) {
        vscode.window.showErrorMessage('Cannot update milestones in a read-only workspace.');
        return;
      }

      try {
        const milestones = await client.listMilestones(projectRoot);
        const openMilestones = milestones.filter((m) => m.status !== 'completed');

        if (openMilestones.length === 0) {
          vscode.window.showInformationMessage('No uncompleted milestones found.');
          return;
        }

        const selected = await vscode.window.showQuickPick(
          openMilestones.map((m) => ({
            label: m.title,
            description: m.target_date ? `Target: ${m.target_date}` : undefined,
            detail: m.id,
            id: m.id,
          })),
          { placeHolder: 'Select milestone to mark as completed' }
        );

        if (!selected) {
          return;
        }

        await client.completeMilestone(projectRoot, selected.id);
        workspaceManager.notifyProjectChanged();
        vscode.window.showInformationMessage(`Milestone '${selected.label}' marked as complete!`);
      } catch (err) {
        vscode.window.showErrorMessage(`Failed to complete milestone: ${(err as Error).message}`);
      }
    })
  );
}
