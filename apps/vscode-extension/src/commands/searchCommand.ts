import * as vscode from 'vscode';
import type { CoreClient } from '../core/client';
import type { SearchResult } from '../types';
import type { WorkspaceManager } from '../workspace/workspaceManager';

interface SearchQuickPickItem extends vscode.QuickPickItem {
  result: SearchResult;
  projectRoot: string;
}

export function registerSearchCommand(
  context: vscode.ExtensionContext,
  workspaceManager: WorkspaceManager,
  getClient: () => CoreClient | null
): void {
  context.subscriptions.push(
    vscode.commands.registerCommand('devcontext.search', async () => {
      const client = getClient();
      if (!client) {
        vscode.window.showErrorMessage('DevContext core is not available.');
        return;
      }

      const projectRoot = await workspaceManager.getTargetProjectRoot(
        'Select workspace to search'
      );
      if (!projectRoot) {
        return;
      }

      const quickPick = vscode.window.createQuickPick<SearchQuickPickItem>();
      quickPick.title = 'DevContext: Full-Text Search';
      quickPick.placeholder = 'Type to search notes, decisions, milestones, logs (e.g. SQLite WAL, OAuth)...';
      quickPick.matchOnDescription = true;
      quickPick.matchOnDetail = true;

      let debounceTimer: NodeJS.Timeout | undefined;

      quickPick.onDidChangeValue((query) => {
        if (debounceTimer) {
          clearTimeout(debounceTimer);
        }

        const trimmed = query.trim();
        if (!trimmed) {
          quickPick.items = [];
          return;
        }

        quickPick.busy = true;

        debounceTimer = setTimeout(async () => {
          try {
            const results = await client.search(projectRoot, trimmed, { limit: 20 });
            quickPick.items = results.map((r) => {
              let icon = '$(note)';
              if (r.record_type === 'decision') {
                icon = '$(law)';
              } else if (r.record_type === 'milestone') {
                icon = '$(milestone)';
              } else if (r.record_type === 'log_entry') {
                icon = '$(output)';
              }

              const cleanSnippet = r.snippet.replace(/\[\/?match\]/g, '**');

              return {
                label: `${icon} ${r.title}`,
                description: `[${r.record_type.toUpperCase()}]`,
                detail: cleanSnippet || (r.tags.length > 0 ? `Tags: ${r.tags.join(', ')}` : undefined),
                result: r,
                projectRoot,
              };
            });
          } catch {
            quickPick.items = [];
          } finally {
            quickPick.busy = false;
          }
        }, 150);
      });

      quickPick.onDidAccept(async () => {
        const selected = quickPick.selectedItems[0];
        quickPick.hide();

        if (!selected) {
          return;
        }

        const { result, projectRoot: root } = selected;
        if (result.record_type === 'note') {
          vscode.commands.executeCommand('devcontext.showNote', result.record_id, root);
        } else if (result.record_type === 'decision') {
          vscode.commands.executeCommand('devcontext.showDecision', result.record_id, root);
        } else {
          vscode.window.showInformationMessage(
            `Selected ${result.record_type}: '${result.title}' (ID: ${result.record_id})`
          );
        }
      });

      quickPick.onDidHide(() => {
        if (debounceTimer) {
          clearTimeout(debounceTimer);
        }
        quickPick.dispose();
      });

      quickPick.show();
    })
  );
}
