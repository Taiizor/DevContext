import * as path from 'node:path';
import * as vscode from 'vscode';
import type { CoreClient } from '../core/client';
import type { Reference } from '../types';
import type { WorkspaceManager } from '../workspace/workspaceManager';

export function registerReferenceCommands(
  context: vscode.ExtensionContext,
  workspaceManager: WorkspaceManager,
  getClient: () => CoreClient | null
): void {
  // 1. Open Reference Target
  context.subscriptions.push(
    vscode.commands.registerCommand(
      'devcontext.openReference',
      async (reference: Reference, projectRoot?: string) => {
        const root = projectRoot || (await workspaceManager.getTargetProjectRoot());

        switch (reference.ref_type) {
          case 'file': {
            if (!root) {
              return;
            }
            // Parse optional line anchor (e.g. src/foo.rs#L10-L20 or src/foo.rs:10)
            const parts = reference.target.split('#');
            const relPath = parts[0] ?? '';
            const anchor = parts[1];
            const targetFile = path.isAbsolute(relPath) ? relPath : path.join(root, relPath);

            try {
              const doc = await vscode.workspace.openTextDocument(vscode.Uri.file(targetFile));
              let selection: vscode.Range | undefined;

              if (anchor) {
                const match = anchor.match(/L(\d+)(?:-L(\d+))?/i);
                if (match && match[1]) {
                  const startLine = Math.max(0, parseInt(match[1], 10) - 1);
                  const endLine = match[2]
                    ? Math.max(0, parseInt(match[2], 10) - 1)
                    : startLine;
                  selection = new vscode.Range(startLine, 0, endLine, 0);
                }
              }

              await vscode.window.showTextDocument(doc, { selection });
            } catch (err) {
              vscode.window.showErrorMessage(
                `Unable to open file '${reference.target}': ${(err as Error).message}`
              );
            }
            break;
          }

          case 'folder': {
            if (!root) {
              return;
            }
            const targetFolder = path.isAbsolute(reference.target)
              ? reference.target
              : path.join(root, reference.target);

            vscode.commands.executeCommand(
              'revealInExplorer',
              vscode.Uri.file(targetFolder)
            );
            break;
          }

          case 'url': {
            try {
              const uri = vscode.Uri.parse(reference.target);
              await vscode.env.openExternal(uri);
            } catch (err) {
              vscode.window.showErrorMessage(
                `Unable to open URL '${reference.target}': ${(err as Error).message}`
              );
            }
            break;
          }

          case 'git_commit': {
            vscode.window.showInformationMessage(
              `Git Commit: ${reference.target} (associated with project memory)`
            );
            break;
          }

          default: {
            vscode.window.showInformationMessage(
              `Reference (${reference.ref_type}): ${reference.target}`
            );
          }
        }
      }
    )
  );

  // 2. Link Active File to a DevContext Note
  context.subscriptions.push(
    vscode.commands.registerCommand('devcontext.linkFileToNote', async () => {
      const editor = vscode.window.activeTextEditor;
      if (!editor) {
        vscode.window.showWarningMessage('No active editor file to link.');
        return;
      }

      const client = getClient();
      if (!client) {
        return;
      }

      const projectRoot = await workspaceManager.getTargetProjectRoot();
      if (!projectRoot) {
        return;
      }

      const relPath = vscode.workspace.asRelativePath(editor.document.uri);
      const notes = await client.listNotes(projectRoot);

      if (notes.length === 0) {
        vscode.window.showInformationMessage(
          'No notes found in project. Create a note first to link this file.'
        );
        return;
      }

      const selected = await vscode.window.showQuickPick(
        notes.map((n) => ({
          label: n.title,
          description: n.tags.map((t) => `#${t}`).join(' '),
          id: n.id,
        })),
        { placeHolder: `Select note to link '${relPath}' to` }
      );

      if (!selected) {
        return;
      }

      try {
        const noteDetail = await client.getNote(projectRoot, selected.id);
        const updatedContent = `${noteDetail.content_markdown}\n\n- Linked file: [${relPath}](${relPath})`;
        await client.editNote(projectRoot, selected.id, {
          content: updatedContent,
        });

        vscode.window.showInformationMessage(
          `File '${relPath}' linked to note '${selected.label}'.`
        );
      } catch (err) {
        vscode.window.showErrorMessage(
          `Failed to link file: ${(err as Error).message}`
        );
      }
    })
  );
}
