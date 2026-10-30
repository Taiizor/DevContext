import * as vscode from 'vscode';
import type { CoreClient } from '../core/client';
import type { WorkspaceManager } from '../workspace/workspaceManager';

export function registerNoteCommands(
  context: vscode.ExtensionContext,
  workspaceManager: WorkspaceManager,
  getClient: () => CoreClient | null
): void {
  // 1. New Note
  context.subscriptions.push(
    vscode.commands.registerCommand(
      'devcontext.newNote',
      async (explicitDir?: string) => {
        const client = getClient();
        if (!client) {
          vscode.window.showErrorMessage('DevContext core is not available.');
          return;
        }

        const projectRoot =
          explicitDir ||
          (await workspaceManager.getTargetProjectRoot('Select workspace for new note'));
        if (!projectRoot) {
          return;
        }

        if (workspaceManager.isWorkspaceReadOnly(projectRoot)) {
          vscode.window.showErrorMessage('Cannot create notes in a read-only workspace.');
          return;
        }

        const title = await vscode.window.showInputBox({
          title: 'New DevContext Note',
          prompt: 'Enter note title',
          validateInput: (v) => (!v || !v.trim() ? 'Title cannot be empty.' : null),
        });
        if (!title) {
          return;
        }

        const content = await vscode.window.showInputBox({
          title: 'New DevContext Note',
          prompt: 'Enter Markdown content for this note',
          value: '',
          validateInput: (v) => (!v || !v.trim() ? 'Content cannot be empty.' : null),
        });
        if (!content) {
          return;
        }

        const rawTags = await vscode.window.showInputBox({
          title: 'New DevContext Note',
          prompt: 'Enter comma-separated tags (optional)',
          placeHolder: 'architecture, database, security',
        });

        const tags = rawTags
          ? rawTags
              .split(',')
              .map((t) => t.trim())
              .filter(Boolean)
          : [];

        try {
          await client.addNote(projectRoot, title, content, tags);
          workspaceManager.notifyProjectChanged();
          vscode.window.showInformationMessage(`Note '${title}' saved successfully.`);
        } catch (err) {
          vscode.window.showErrorMessage(`Failed to save note: ${(err as Error).message}`);
        }
      }
    )
  );

  // 2. New Note from Active Editor Selection
  context.subscriptions.push(
    vscode.commands.registerCommand('devcontext.newNoteFromSelection', async () => {
      const editor = vscode.window.activeTextEditor;
      if (!editor || editor.selection.isEmpty) {
        vscode.window.showWarningMessage('Please select text in the editor to create a note.');
        return;
      }

      const client = getClient();
      if (!client) {
        vscode.window.showErrorMessage('DevContext core is not available.');
        return;
      }

      const selectedText = editor.document.getText(editor.selection);
      const relativeFile = vscode.workspace.asRelativePath(editor.document.uri);
      const lineStart = editor.selection.start.line + 1;
      const lineEnd = editor.selection.end.line + 1;

      const projectRoot = await workspaceManager.getTargetProjectRoot();
      if (!projectRoot) {
        return;
      }

      const title = await vscode.window.showInputBox({
        title: 'Create Note from Code Selection',
        prompt: 'Enter note title',
        value: `Snippet from ${relativeFile}:${lineStart}`,
        validateInput: (v) => (!v || !v.trim() ? 'Title cannot be empty.' : null),
      });
      if (!title) {
        return;
      }

      const content = `Reference: [${relativeFile}#L${lineStart}-L${lineEnd}](${relativeFile})\n\n\`\`\`\n${selectedText}\n\`\`\``;

      try {
        await client.addNote(projectRoot, title, content, ['snippet', 'code-reference']);
        workspaceManager.notifyProjectChanged();
        vscode.window.showInformationMessage(`Note created with code reference.`);
      } catch (err) {
        vscode.window.showErrorMessage(`Failed to create note: ${(err as Error).message}`);
      }
    })
  );

  // 3. Show Note Details
  context.subscriptions.push(
    vscode.commands.registerCommand(
      'devcontext.showNote',
      async (noteId: string, projectRoot?: string) => {
        const client = getClient();
        if (!client) {
          return;
        }

        const root = projectRoot || (await workspaceManager.getTargetProjectRoot());
        if (!root) {
          return;
        }

        try {
          const note = await client.getNote(root, noteId);
          const docContent = `# ${note.record.title}\n\n**Created:** ${note.record.created_at}  \n**Updated:** ${note.record.updated_at}  \n**Tags:** ${note.record.tags.map((t) => `\`#${t}\``).join(' ') || 'None'}\n\n---\n\n${note.content_markdown}`;

          const doc = await vscode.workspace.openTextDocument({
            content: docContent,
            language: 'markdown',
          });
          await vscode.window.showTextDocument(doc, { preview: true });
        } catch (err) {
          vscode.window.showErrorMessage(`Failed to load note: ${(err as Error).message}`);
        }
      }
    )
  );

  // 4. Edit Note
  context.subscriptions.push(
    vscode.commands.registerCommand(
      'devcontext.editNote',
      async (itemOrId?: { id: string; projectRoot: string } | string) => {
        const client = getClient();
        if (!client) {
          return;
        }

        let noteId: string | undefined;
        let projectRoot: string | null = null;

        if (typeof itemOrId === 'string') {
          noteId = itemOrId;
          projectRoot = await workspaceManager.getTargetProjectRoot();
        } else if (itemOrId && typeof itemOrId === 'object') {
          noteId = itemOrId.id;
          projectRoot =
            itemOrId.projectRoot || (await workspaceManager.getTargetProjectRoot());
        }

        if (!noteId || !projectRoot) {
          return;
        }

        if (workspaceManager.isWorkspaceReadOnly(projectRoot)) {
          vscode.window.showErrorMessage('Cannot edit notes in a read-only workspace.');
          return;
        }

        try {
          const current = await client.getNote(projectRoot, noteId);

          const newTitle = await vscode.window.showInputBox({
            title: 'Edit Note Title',
            value: current.record.title,
            validateInput: (v) => (!v || !v.trim() ? 'Title cannot be empty.' : null),
          });
          if (!newTitle) {
            return;
          }

          const newContent = await vscode.window.showInputBox({
            title: 'Edit Note Content',
            value: current.content_markdown,
            validateInput: (v) => (!v || !v.trim() ? 'Content cannot be empty.' : null),
          });
          if (!newContent) {
            return;
          }

          await client.editNote(projectRoot, noteId, {
            title: newTitle,
            content: newContent,
          });

          workspaceManager.notifyProjectChanged();
          vscode.window.showInformationMessage(`Note updated.`);
        } catch (err) {
          vscode.window.showErrorMessage(`Failed to update note: ${(err as Error).message}`);
        }
      }
    )
  );

  // 5. Delete Note
  context.subscriptions.push(
    vscode.commands.registerCommand(
      'devcontext.deleteNote',
      async (itemOrId?: { id: string; projectRoot: string } | string) => {
        const client = getClient();
        if (!client) {
          return;
        }

        let noteId: string | undefined;
        let projectRoot: string | null = null;

        if (typeof itemOrId === 'string') {
          noteId = itemOrId;
          projectRoot = await workspaceManager.getTargetProjectRoot();
        } else if (itemOrId && typeof itemOrId === 'object') {
          noteId = itemOrId.id;
          projectRoot =
            itemOrId.projectRoot || (await workspaceManager.getTargetProjectRoot());
        }

        if (!noteId || !projectRoot) {
          return;
        }

        if (workspaceManager.isWorkspaceReadOnly(projectRoot)) {
          vscode.window.showErrorMessage('Cannot delete notes in a read-only workspace.');
          return;
        }

        const confirm = await vscode.window.showWarningMessage(
          'Are you sure you want to delete this note?',
          { modal: true },
          'Delete Note'
        );

        if (confirm !== 'Delete Note') {
          return;
        }

        try {
          await client.deleteNote(projectRoot, noteId);
          workspaceManager.notifyProjectChanged();
          vscode.window.showInformationMessage('Note deleted.');
        } catch (err) {
          vscode.window.showErrorMessage(`Failed to delete note: ${(err as Error).message}`);
        }
      }
    )
  );
}
