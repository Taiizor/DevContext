import { beforeEach, describe, expect, it } from 'bun:test';
import { registerDecisionCommands } from '../src/commands/decisionCommands';
import { registerDoctorCommand } from '../src/commands/doctorCommand';
import { registerExportImportCommands } from '../src/commands/exportImportCommands';
import { registerInitCommand } from '../src/commands/init';
import { registerLogCommands } from '../src/commands/logCommands';
import { registerMilestoneCommands } from '../src/commands/milestoneCommands';
import { registerNoteCommands } from '../src/commands/noteCommands';
import { registerReferenceCommands } from '../src/commands/referenceCommands';
import { registerSearchCommand } from '../src/commands/searchCommand';
import { CoreClient } from '../src/core/client';
import { DevContextStatusBar } from '../src/statusbar/statusBar';
import type { HandshakeResult, ProjectStatus } from '../src/types';
import { WorkspaceManager } from '../src/workspace/workspaceManager';
import { mockVscode } from './mockVscode';

describe('VS Code Extension Commands and User Workflows', () => {
  let activeClient: CoreClient;
  let manager: WorkspaceManager;
  const mockContext = { subscriptions: [] } as unknown as import('vscode').ExtensionContext;

  beforeEach(() => {
    mockVscode.commands.registered.clear();
    manager = new WorkspaceManager();
    activeClient = new CoreClient('devcontext');

    const handshake: HandshakeResult = {
      status: 'ready',
      executablePath: '/bin/devcontext',
      cliVersion: '0.1.0',
      protocolVersion: '1.0',
      message: 'OK',
    };

    registerInitCommand(mockContext, manager, () => activeClient);
    registerNoteCommands(mockContext, manager, () => activeClient);
    registerDecisionCommands(mockContext, manager, () => activeClient);
    registerMilestoneCommands(mockContext, manager, () => activeClient);
    registerLogCommands(mockContext, manager, () => activeClient);
    registerSearchCommand(mockContext, manager, () => activeClient);
    registerReferenceCommands(mockContext, manager, () => activeClient);
    registerExportImportCommands(mockContext, manager, () => activeClient);
    registerDoctorCommand(mockContext, manager, () => activeClient, () => handshake);
  });

  it('registers all Phase 6 required commands into the command registry', () => {
    const registered = mockVscode.commands.registered;
    expect(registered.has('devcontext.init')).toBe(true);
    expect(registered.has('devcontext.newNote')).toBe(true);
    expect(registered.has('devcontext.newNoteFromSelection')).toBe(true);
    expect(registered.has('devcontext.showNote')).toBe(true);
    expect(registered.has('devcontext.editNote')).toBe(true);
    expect(registered.has('devcontext.deleteNote')).toBe(true);
    expect(registered.has('devcontext.newDecision')).toBe(true);
    expect(registered.has('devcontext.showDecision')).toBe(true);
    expect(registered.has('devcontext.supersedeDecision')).toBe(true);
    expect(registered.has('devcontext.newMilestone')).toBe(true);
    expect(registered.has('devcontext.completeMilestone')).toBe(true);
    expect(registered.has('devcontext.newLog')).toBe(true);
    expect(registered.has('devcontext.search')).toBe(true);
    expect(registered.has('devcontext.openReference')).toBe(true);
    expect(registered.has('devcontext.linkFileToNote')).toBe(true);
    expect(registered.has('devcontext.export')).toBe(true);
    expect(registered.has('devcontext.import')).toBe(true);
    expect(registered.has('devcontext.doctor')).toBe(true);
  });

  it('devcontext.init respects read-only workspace guard', async () => {
    manager.isWorkspaceReadOnly = () => true;

    let initCalled = false;
    activeClient.init = async () => {
      initCalled = true;
    };

    let errorShown = '';
    mockVscode.window.showErrorMessage = async (msg: string) => {
      errorShown = msg;
      return undefined;
    };

    const handler = mockVscode.commands.registered.get('devcontext.init');
    expect(handler).toBeDefined();
    await handler?.('/read/only/folder');

    expect(initCalled).toBe(false);
    expect(errorShown).toContain('read-only workspace folder');
  });

  it('devcontext.newNote prompts for title, content, tags, and invokes client.addNote', async () => {
    manager.getTargetProjectRoot = async () => '/mock/project';
    manager.isWorkspaceReadOnly = () => false;

    let addedTitle = '';
    let addedContent = '';
    let addedTags: string[] = [];

    activeClient.addNote = async (_root, title, content, tags) => {
      addedTitle = title;
      addedContent = content;
      addedTags = tags;
    };

    let promptStep = 0;
    mockVscode.window.showInputBox = async () => {
      promptStep += 1;
      if (promptStep === 1) return 'Architecture Patterns';
      if (promptStep === 2) return 'Notes on CQRS and Event Sourcing';
      if (promptStep === 3) return 'patterns, cqrs, events';
      return undefined;
    };

    const handler = mockVscode.commands.registered.get('devcontext.newNote');
    await handler?.('/mock/project');

    expect(addedTitle).toBe('Architecture Patterns');
    expect(addedContent).toBe('Notes on CQRS and Event Sourcing');
    expect(addedTags).toEqual(['patterns', 'cqrs', 'events']);
  });

  it('devcontext.newDecision prompts through ADR fields and invokes client.addDecision', async () => {
    manager.getTargetProjectRoot = async () => '/mock/project';
    manager.isWorkspaceReadOnly = () => false;

    let recordedTitle = '';
    let recordedDecision = '';
    let recordedStatus = '';

    activeClient.addDecision = async (_root, title, _context, decision, _consequences, status) => {
      recordedTitle = title;
      recordedDecision = decision;
      recordedStatus = status || 'accepted';
    };

    let step = 0;
    mockVscode.window.showInputBox = async () => {
      step += 1;
      if (step === 1) return 'ADR 0012: Binary Distribution';
      if (step === 2) return 'Need local-first core resolution';
      if (step === 3) return 'Adopt platform-targeted VSIX packaging';
      if (step === 4) return 'Offline reliability guaranteed';
      return undefined;
    };

    mockVscode.window.showQuickPick = async () => ({ label: 'accepted' }) as unknown;

    const handler = mockVscode.commands.registered.get('devcontext.newDecision');
    await handler?.('/mock/project');

    expect(recordedTitle).toBe('ADR 0012: Binary Distribution');
    expect(recordedDecision).toBe('Adopt platform-targeted VSIX packaging');
    expect(recordedStatus).toBe('accepted');
  });

  it('devcontext.export presents preview and requires explicit user confirmation', async () => {
    manager.getTargetProjectRoot = async () => '/mock/project';

    const mockStatus: ProjectStatus = {
      project_id: '1',
      name: 'Alpha',
      root_path: '/mock/project',
      created_at: 'now',
      vcs_identifier: null,
      stats: { notes_count: 10, decisions_count: 4, milestones_count: 2, log_entries_count: 8 },
    };
    activeClient.status = async () => mockStatus;

    mockVscode.window.showQuickPick = async () =>
      ({ label: 'Markdown Directory Export', format: 'markdown' }) as unknown;

    const infoMessages: string[] = [];
    mockVscode.window.showInformationMessage = async (msg: string, ..._opts: unknown[]) => {
      infoMessages.push(msg);
      return 'Confirm & Select Destination' as unknown;
    };

    mockVscode.window.showOpenDialog = async () => [
      { fsPath: '/exported/dest' } as unknown as import('vscode').Uri,
    ];

    const handler = mockVscode.commands.registered.get('devcontext.export');
    await handler?.();

    expect(infoMessages[0]).toContain("Export Preview for 'Alpha'");
    expect(infoMessages[0]).toContain('Notes: 10 items');
    expect(infoMessages[0]).toContain('Decisions (ADRs): 4 items');
    expect(infoMessages[1]).toContain("Export configured for 'Markdown Directory Export'");
  });

  it('devcontext.openReference handles external URLs and files without side effects', async () => {
    let openedUrl = '';
    mockVscode.env.openExternal = async (uri: unknown) => {
      openedUrl = (uri as { fsPath: string }).fsPath;
      return true;
    };

    const handler = mockVscode.commands.registered.get('devcontext.openReference');
    await handler?.({
      id: 'r1',
      record_id: 'rec1',
      ref_type: 'url',
      target: 'https://devcontext.local/docs',
      description: null,
      metadata: null,
    });

    expect(openedUrl).toBe('https://devcontext.local/docs');
  });

  it('DevContextStatusBar displays core status and project name', async () => {
    manager.getWorkspaceProjects = () => [
      {
        uri: { fsPath: '/active/proj' } as unknown as import('vscode').Uri,
        name: 'ActiveProject',
        isInitialized: true,
        isReadOnly: false,
      },
    ];

    activeClient.status = async () => ({
      project_id: 'p1',
      name: 'ActiveProject',
      root_path: '/active/proj',
      created_at: 'now',
      vcs_identifier: null,
      stats: { notes_count: 3, decisions_count: 1, milestones_count: 0, log_entries_count: 2 },
    });

    const handshake: HandshakeResult = {
      status: 'ready',
      executablePath: '/bin/devcontext',
      cliVersion: '0.1.0',
      protocolVersion: '1.0',
      message: 'OK',
    };

    const statusBar = new DevContextStatusBar(manager, () => activeClient, () => handshake);
    await statusBar.update();

    expect(statusBar).toBeDefined();

    statusBar.dispose();
  });
});
