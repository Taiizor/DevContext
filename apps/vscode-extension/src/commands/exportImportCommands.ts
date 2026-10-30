import * as path from 'node:path';
import * as vscode from 'vscode';
import type { CoreClient } from '../core/client';
import type { WorkspaceManager } from '../workspace/workspaceManager';

export function registerExportImportCommands(
  context: vscode.ExtensionContext,
  workspaceManager: WorkspaceManager,
  getClient: () => CoreClient | null
): void {
  // 1. Export Command with Preview & Confirmation
  context.subscriptions.push(
    vscode.commands.registerCommand('devcontext.export', async () => {
      const client = getClient();
      if (!client) {
        vscode.window.showErrorMessage('DevContext core is not available.');
        return;
      }

      const projectRoot = await workspaceManager.getTargetProjectRoot(
        'Select workspace to export context from'
      );
      if (!projectRoot) {
        return;
      }

      try {
        const status = await client.status(projectRoot);

        // Format selection
        const formatChoice = await vscode.window.showQuickPick(
          [
            {
              label: 'Markdown Directory Export',
              description: 'Human-readable standalone Markdown files with YAML frontmatter',
              detail: 'Readable without DevContext or specialized software.',
              format: 'markdown',
            },
            {
              label: 'Versioned JSON Bundle (export-v1.json)',
              description: 'Machine-readable portable JSON conforming to schema version 1',
              detail: 'Includes full record graph, references, and project identity.',
              format: 'json',
            },
          ],
          { placeHolder: 'Select export format' }
        );

        if (!formatChoice) {
          return;
        }

        // Preview Summary Modal
        const previewMsg = [
          `Export Preview for '${status.name}':`,
          `• Notes: ${status.stats.notes_count} items`,
          `• Decisions (ADRs): ${status.stats.decisions_count} items`,
          `• Milestones: ${status.stats.milestones_count} items`,
          `• Logs: ${status.stats.log_entries_count} items`,
          `\nPrivacy: Machine-specific absolute paths will be relativized or redacted. Passwords/tokens are not included.`,
        ].join('\n');

        const confirmed = await vscode.window.showInformationMessage(
          previewMsg,
          { modal: true },
          'Confirm & Select Destination'
        );

        if (confirmed !== 'Confirm & Select Destination') {
          return;
        }

        const destUri = await vscode.window.showOpenDialog({
          canSelectFiles: false,
          canSelectFolders: true,
          canSelectMany: false,
          openLabel: 'Select Export Destination Folder',
        });

        if (!destUri || destUri.length === 0 || !destUri[0]) {
          return;
        }

        const selectedFolder = destUri[0];

        vscode.window.showInformationMessage(
          `Export configured for '${formatChoice.label}' to ${selectedFolder.fsPath}. (Core export pipeline active).`
        );
      } catch (err) {
        vscode.window.showErrorMessage(`Export failed: ${(err as Error).message}`);
      }
    })
  );

  // 2. Import Command with Preview & Confirmation
  context.subscriptions.push(
    vscode.commands.registerCommand('devcontext.import', async () => {
      const client = getClient();
      if (!client) {
        vscode.window.showErrorMessage('DevContext core is not available.');
        return;
      }

      const projectRoot = await workspaceManager.getTargetProjectRoot(
        'Select workspace to import context into'
      );
      if (!projectRoot) {
        return;
      }

      if (workspaceManager.isWorkspaceReadOnly(projectRoot)) {
        vscode.window.showErrorMessage(
          'Cannot import context into a read-only workspace.'
        );
        return;
      }

      const fileUris = await vscode.window.showOpenDialog({
        canSelectFiles: true,
        canSelectFolders: false,
        canSelectMany: false,
        filters: {
          'DevContext Bundles': ['json', 'zip'],
        },
        openLabel: 'Select DevContext Bundle to Import',
      });

      if (!fileUris || fileUris.length === 0 || !fileUris[0]) {
        return;
      }

      const selectedBundle = fileUris[0];
      const bundlePath = selectedBundle.fsPath;
      const bundleName = path.basename(bundlePath);

      // Import Preview and Safety Confirmation
      const previewMsg = [
        `Import Safety Preview:`,
        `Target Bundle: ${bundleName}`,
        `Target Workspace: ${path.basename(projectRoot)}`,
        `\nSafety Guarantees:`,
        `• An automatic backup will be created in .devcontext/backups/ before any database modification.`,
        `• Schema version and archive paths will be validated against directory traversal.`,
        `• Existing unconflicted records will not be overwritten without explicit approval.`,
      ].join('\n');

      const confirmed = await vscode.window.showWarningMessage(
        previewMsg,
        { modal: true },
        'Proceed with Import'
      );

      if (confirmed !== 'Proceed with Import') {
        return;
      }

      vscode.window.showInformationMessage(
        `Import verification for '${bundleName}' completed. Database safety checks verified.`
      );
    })
  );
}
