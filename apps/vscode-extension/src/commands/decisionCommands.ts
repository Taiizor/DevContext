import * as vscode from 'vscode';
import type { CoreClient } from '../core/client';
import type { WorkspaceManager } from '../workspace/workspaceManager';

export function registerDecisionCommands(
  context: vscode.ExtensionContext,
  workspaceManager: WorkspaceManager,
  getClient: () => CoreClient | null
): void {
  // 1. New Decision
  context.subscriptions.push(
    vscode.commands.registerCommand(
      'devcontext.newDecision',
      async (explicitDir?: string) => {
        const client = getClient();
        if (!client) {
          vscode.window.showErrorMessage('DevContext core is not available.');
          return;
        }

        const projectRoot =
          explicitDir ||
          (await workspaceManager.getTargetProjectRoot('Select workspace for new decision'));
        if (!projectRoot) {
          return;
        }

        if (workspaceManager.isWorkspaceReadOnly(projectRoot)) {
          vscode.window.showErrorMessage(
            'Cannot record decisions in a read-only workspace.'
          );
          return;
        }

        // Step 1: Title
        const title = await vscode.window.showInputBox({
          title: 'New Architecture Decision (Step 1 of 4: Title)',
          prompt: 'Enter decision title (e.g., ADR 0001: Storage Engine Selection)',
          validateInput: (v) => (!v || !v.trim() ? 'Title cannot be empty.' : null),
        });
        if (!title) {
          return;
        }

        // Step 2: Context
        const decisionContext = await vscode.window.showInputBox({
          title: 'New Architecture Decision (Step 2 of 4: Context)',
          prompt: 'What architectural context or problem motivates this decision?',
          validateInput: (v) => (!v || !v.trim() ? 'Context cannot be empty.' : null),
        });
        if (!decisionContext) {
          return;
        }

        // Step 3: Decision
        const decision = await vscode.window.showInputBox({
          title: 'New Architecture Decision (Step 3 of 4: Decision)',
          prompt: 'What was decided? State the technical choice clearly.',
          validateInput: (v) => (!v || !v.trim() ? 'Decision cannot be empty.' : null),
        });
        if (!decision) {
          return;
        }

        // Step 4: Consequences
        const consequences = await vscode.window.showInputBox({
          title: 'New Architecture Decision (Step 4 of 4: Consequences)',
          prompt: 'What are the positive and negative consequences/trade-offs?',
          validateInput: (v) => (!v || !v.trim() ? 'Consequences cannot be empty.' : null),
        });
        if (!consequences) {
          return;
        }

        const statusChoice = await vscode.window.showQuickPick(
          [
            { label: 'accepted', description: 'Decision is accepted and in effect' },
            { label: 'proposed', description: 'Decision is under review' },
          ],
          { placeHolder: 'Select initial status' }
        );

        try {
          await client.addDecision(
            projectRoot,
            title,
            decisionContext,
            decision,
            consequences,
            statusChoice?.label || 'accepted'
          );
          workspaceManager.notifyProjectChanged();
          vscode.window.showInformationMessage(`Decision '${title}' recorded.`);
        } catch (err) {
          vscode.window.showErrorMessage(`Failed to record decision: ${(err as Error).message}`);
        }
      }
    )
  );

  // 2. Show Decision Details
  context.subscriptions.push(
    vscode.commands.registerCommand(
      'devcontext.showDecision',
      async (decisionId: string, projectRoot?: string) => {
        const client = getClient();
        if (!client) {
          return;
        }

        const root = projectRoot || (await workspaceManager.getTargetProjectRoot());
        if (!root) {
          return;
        }

        try {
          const d = await client.getDecision(root, decisionId);
          let docContent = `# ${d.record.title}\n\n`;
          docContent += `- **Status:** \`${d.status.toUpperCase()}\`\n`;
          docContent += `- **Date:** ${d.record.created_at}\n`;
          docContent += `- **ID:** \`${d.record.id}\`\n`;
          if (d.superseded_by) {
            docContent += `- **Superseded by:** \`${d.superseded_by}\`\n`;
          }
          docContent += `\n## Context\n\n${d.context}\n\n`;
          docContent += `## Decision\n\n${d.decision}\n\n`;
          docContent += `## Consequences\n\n${d.consequences}\n\n`;
          if (d.alternatives_considered) {
            docContent += `## Alternatives Considered\n\n${d.alternatives_considered}\n\n`;
          }

          const doc = await vscode.workspace.openTextDocument({
            content: docContent,
            language: 'markdown',
          });
          await vscode.window.showTextDocument(doc, { preview: true });
        } catch (err) {
          vscode.window.showErrorMessage(`Failed to load decision: ${(err as Error).message}`);
        }
      }
    )
  );

  // 3. Supersede Decision
  context.subscriptions.push(
    vscode.commands.registerCommand(
      'devcontext.supersedeDecision',
      async (itemOrId?: { id: string; projectRoot: string } | string) => {
        const client = getClient();
        if (!client) {
          return;
        }

        let decisionId: string | undefined;
        let projectRoot: string | null = null;

        if (typeof itemOrId === 'string') {
          decisionId = itemOrId;
          projectRoot = await workspaceManager.getTargetProjectRoot();
        } else if (itemOrId && typeof itemOrId === 'object') {
          decisionId = itemOrId.id;
          projectRoot = itemOrId.projectRoot;
        }

        if (!decisionId || !projectRoot) {
          return;
        }

        if (workspaceManager.isWorkspaceReadOnly(projectRoot)) {
          vscode.window.showErrorMessage(
            'Cannot update decisions in a read-only workspace.'
          );
          return;
        }

        const decisions = await client.listDecisions(projectRoot);
        const candidates = decisions.filter((d) => d.id !== decisionId);

        if (candidates.length === 0) {
          vscode.window.showInformationMessage(
            'No other decisions found in project to mark as superseding.'
          );
          return;
        }

        const selected = await vscode.window.showQuickPick(
          candidates.map((d) => ({
            label: d.title,
            description: `[${d.status.toUpperCase()}]`,
            detail: d.id,
            id: d.id,
          })),
          { placeHolder: 'Select the newer decision that supersedes this one' }
        );

        if (!selected) {
          return;
        }

        try {
          await client.supersedeDecision(projectRoot, decisionId, selected.id);
          workspaceManager.notifyProjectChanged();
          vscode.window.showInformationMessage(
            `Decision marked as superseded by '${selected.label}'.`
          );
        } catch (err) {
          vscode.window.showErrorMessage(
            `Failed to supersede decision: ${(err as Error).message}`
          );
        }
      }
    )
  );
}
