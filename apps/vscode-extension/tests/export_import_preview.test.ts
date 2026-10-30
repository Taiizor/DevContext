import { describe, it, expect, beforeEach } from 'bun:test';
import * as path from 'node:path';
import { WorkspaceManager } from '../src/workspace/workspaceManager';
import { registerExportImportCommands } from '../src/commands/exportImportCommands';
import type { CoreClient } from '../src/core/client';
import type { ProjectStatus } from '../src/types';
import { mockVscode } from './mockVscode';

describe('Export and Import Preview & Safety Confirmation Workflows', () => {
  let workspaceManager: WorkspaceManager;
  let mockClient: Partial<CoreClient>;
  const mockContext: import('vscode').ExtensionContext = {
    subscriptions: [],
  } as unknown as import('vscode').ExtensionContext;

  const testProjectRoot = '/test/workspace/project-alpha';
  const mockStatus: ProjectStatus = {
    project_id: 'proj-1234-uuid',
    name: 'Project Alpha',
    root_path: testProjectRoot,
    created_at: '2026-10-30T20:00:00Z',
    vcs_identifier: 'git:main:abc1234',
    stats: {
      notes_count: 14,
      decisions_count: 6,
      milestones_count: 3,
      log_entries_count: 22,
    },
  };

  beforeEach(() => {
    mockVscode.commands.registered.clear();
    mockContext.subscriptions.length = 0;
    workspaceManager = new WorkspaceManager();
    workspaceManager.getTargetProjectRoot = async () => testProjectRoot;
    workspaceManager.isWorkspaceReadOnly = () => false;

    mockClient = {
      status: async () => mockStatus,
    };

    registerExportImportCommands(
      mockContext,
      workspaceManager,
      () => mockClient as CoreClient
    );
  });

  describe('Export Preview & User Confirmation', () => {
    it('presents accurate record counts and path redaction notice in modal preview', async () => {
      mockVscode.window.showQuickPick = async () =>
        ({ label: 'Markdown Directory Export', format: 'markdown' }) as unknown;

      let modalContent = '';
      let modalOptions: unknown = null;
      let confirmedButton = '';

      mockVscode.window.showInformationMessage = async (msg: string, ...opts: unknown[]) => {
        if (opts.length > 0 && typeof opts[0] === 'object') {
          modalContent = msg;
          modalOptions = opts[0];
          confirmedButton = (opts[1] as string) || '';
        }
        return 'Confirm & Select Destination' as unknown;
      };

      mockVscode.window.showOpenDialog = async () => [
        { fsPath: '/dest/export/folder' } as unknown as import('vscode').Uri,
      ];

      const exportCmd = mockVscode.commands.registered.get('devcontext.export');
      await exportCmd?.();

      // 1. Verify modal presentation flag
      expect(modalOptions).toEqual({ modal: true });
      expect(confirmedButton).toBe('Confirm & Select Destination');

      // 2. Verify accurate record counts from client.status
      expect(modalContent).toContain("Export Preview for 'Project Alpha':");
      expect(modalContent).toContain('• Notes: 14 items');
      expect(modalContent).toContain('• Decisions (ADRs): 6 items');
      expect(modalContent).toContain('• Milestones: 3 items');
      expect(modalContent).toContain('• Logs: 22 items');

      // 3. Verify path redaction and privacy notice
      expect(modalContent).toContain('Privacy: Machine-specific absolute paths will be relativized or redacted.');
      expect(modalContent).toContain('Passwords/tokens are not included.');
    });

    it('cancelling export preview dialog never prompts destination or initiates export', async () => {
      mockVscode.window.showQuickPick = async () =>
        ({ label: 'Versioned JSON Bundle (export-v1.json)', format: 'json' }) as unknown;

      // User dismisses modal without clicking confirmation
      mockVscode.window.showInformationMessage = async () => undefined;

      let destinationPrompted = false;
      mockVscode.window.showOpenDialog = async () => {
        destinationPrompted = true;
        return [];
      };

      const exportCmd = mockVscode.commands.registered.get('devcontext.export');
      await exportCmd?.();

      expect(destinationPrompted).toBe(false);
    });

    it('cancelling format selection aborts without opening preview modal', async () => {
      // User cancels format selection
      mockVscode.window.showQuickPick = async () => undefined;

      let previewShown = false;
      mockVscode.window.showInformationMessage = async () => {
        previewShown = true;
        return undefined;
      };

      const exportCmd = mockVscode.commands.registered.get('devcontext.export');
      await exportCmd?.();

      expect(previewShown).toBe(false);
    });
  });

  describe('Import Preview & Safety Confirmation', () => {
    it('presents safety guarantees and requires explicit confirmation before import', async () => {
      const mockBundlePath = '/bundles/devcontext-export-v1.json';
      mockVscode.window.showOpenDialog = async () => [
        { fsPath: mockBundlePath } as unknown as import('vscode').Uri,
      ];

      let warningMessage = '';
      let warningOptions: unknown = null;
      let proceedAction = '';

      mockVscode.window.showWarningMessage = async (msg: string, ...opts: unknown[]) => {
        warningMessage = msg;
        warningOptions = opts[0];
        proceedAction = (opts[1] as string) || '';
        return 'Proceed with Import' as unknown;
      };

      let infoMessage = '';
      mockVscode.window.showInformationMessage = async (msg: string) => {
        infoMessage = msg;
        return undefined;
      };

      const importCmd = mockVscode.commands.registered.get('devcontext.import');
      await importCmd?.();

      // 1. Verify safety modal flag
      expect(warningOptions).toEqual({ modal: true });
      expect(proceedAction).toBe('Proceed with Import');

      // 2. Verify bundle and target workspace description
      expect(warningMessage).toContain('Import Safety Preview:');
      expect(warningMessage).toContain('Target Bundle: devcontext-export-v1.json');
      expect(warningMessage).toContain('Target Workspace: project-alpha');

      // 3. Verify safety guarantees
      expect(warningMessage).toContain('An automatic backup will be created in .devcontext/backups/');
      expect(warningMessage).toContain('Schema version and archive paths will be validated against directory traversal.');
      expect(warningMessage).toContain('Existing unconflicted records will not be overwritten without explicit approval.');

      // 4. Verify confirmation notice
      expect(infoMessage).toContain("Import verification for 'devcontext-export-v1.json' completed.");
    });

    it('cancelling import warning modal aborts without applying any changes', async () => {
      mockVscode.window.showOpenDialog = async () => [
        { fsPath: '/bundles/archive.zip' } as unknown as import('vscode').Uri,
      ];

      // User dismisses modal
      mockVscode.window.showWarningMessage = async () => undefined;

      let postNoticeShown = false;
      mockVscode.window.showInformationMessage = async () => {
        postNoticeShown = true;
        return undefined;
      };

      const importCmd = mockVscode.commands.registered.get('devcontext.import');
      await importCmd?.();

      expect(postNoticeShown).toBe(false);
    });

    it('blocks import attempt in a read-only workspace', async () => {
      workspaceManager.isWorkspaceReadOnly = () => true;

      let errorMessage = '';
      mockVscode.window.showErrorMessage = async (msg: string) => {
        errorMessage = msg;
        return undefined;
      };

      let openDialogCalled = false;
      mockVscode.window.showOpenDialog = async () => {
        openDialogCalled = true;
        return [];
      };

      const importCmd = mockVscode.commands.registered.get('devcontext.import');
      await importCmd?.();

      expect(errorMessage).toContain('Cannot import context into a read-only workspace.');
      expect(openDialogCalled).toBe(false);
    });
  });
});
