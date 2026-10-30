import { describe, it, expect, beforeAll, afterAll, beforeEach } from 'bun:test';
import * as path from 'node:path';
import * as fs from 'node:fs';
import * as os from 'node:os';
import { Database } from 'bun:sqlite';
import { CoreClient } from '../src/core/client';
import { WorkspaceManager } from '../src/workspace/workspaceManager';
import { registerInitCommand } from '../src/commands/init';
import { registerNoteCommands } from '../src/commands/noteCommands';
import { registerDecisionCommands } from '../src/commands/decisionCommands';
import { registerMilestoneCommands } from '../src/commands/milestoneCommands';
import { registerLogCommands } from '../src/commands/logCommands';
import { registerSearchCommand } from '../src/commands/searchCommand';
import { registerReferenceCommands } from '../src/commands/referenceCommands';
import { OverviewTreeDataProvider } from '../src/views/overviewView';
import { NotesTreeDataProvider } from '../src/views/notesView';
import { DecisionsTreeDataProvider } from '../src/views/decisionsView';
import { TimelineTreeDataProvider } from '../src/views/timelineView';
import { mockVscode } from './mockVscode';

const extensionDir = path.resolve(__dirname, '..');
const projectRoot = path.resolve(extensionDir, '../..');
const realExecutablePath = path.resolve(
  projectRoot,
  'target/debug',
  process.platform === 'win32' ? 'devcontext.exe' : 'devcontext'
);
const isBinaryAvailable = fs.existsSync(realExecutablePath);
const suite = isBinaryAvailable ? describe : describe.skip;

suite('Real End-to-End Workflows with Native Rust Engine', () => {
  let tempWorkspace: string;
  let workspaceManager: WorkspaceManager;
  let realClient: CoreClient;
  const mockContext: import('vscode').ExtensionContext = {
    subscriptions: [],
  } as unknown as import('vscode').ExtensionContext;

  beforeAll(() => {
    if (!isBinaryAvailable) return;
    expect(fs.existsSync(realExecutablePath)).toBe(true);
    tempWorkspace = fs.mkdtempSync(path.join(os.tmpdir(), 'dc-e2e-ws-'));
    realClient = new CoreClient(realExecutablePath);
    workspaceManager = new WorkspaceManager();

    // Register all real commands bound to realClient
    registerInitCommand(mockContext, workspaceManager, () => realClient);
    registerNoteCommands(mockContext, workspaceManager, () => realClient);
    registerDecisionCommands(mockContext, workspaceManager, () => realClient);
    registerMilestoneCommands(mockContext, workspaceManager, () => realClient);
    registerLogCommands(mockContext, workspaceManager, () => realClient);
    registerSearchCommand(mockContext, workspaceManager, () => realClient);
    registerReferenceCommands(mockContext, workspaceManager, () => realClient);
  });

  afterAll(() => {
    if (!tempWorkspace) return;
    try {
      fs.rmSync(tempWorkspace, { recursive: true, force: true });
    } catch {
      // Ignore cleanup error
    }
  });

  beforeEach(() => {
    // Default workspace to tempWorkspace
    mockVscode.workspace.workspaceFolders = [
      { uri: { fsPath: tempWorkspace } as import('vscode').Uri, name: 'E2E-Workspace', index: 0 },
    ];
  });

  it('Workflow 1: Initializes project and creates Rust SQLite schema', async () => {
    let promptTitle = '';
    mockVscode.window.showInputBox = async (opts) => {
      promptTitle = opts?.title ?? '';
      return 'E2E Test Project';
    };

    const handler = mockVscode.commands.registered.get('devcontext.init');
    expect(handler).toBeDefined();
    await handler?.(tempWorkspace);

    expect(promptTitle).toContain('Initialize DevContext');
    const dbPath = path.join(tempWorkspace, '.devcontext', 'devcontext.db');
    expect(fs.existsSync(dbPath)).toBe(true);

    // Verify status via real client
    const status = await realClient.status(tempWorkspace);
    expect(status.name).toBe('E2E Test Project');
    expect(status.stats.notes_count).toBe(0);
  });

  it('Workflow 2: Full Note CRUD and display lifecycle', async () => {
    // 2.1 Add Note
    const inputs = ['Authentication Design', 'Use OAuth2 with PKCE flow', 'auth,security'];
    let inputIdx = 0;
    mockVscode.window.showInputBox = async () => inputs[inputIdx++];

    const addHandler = mockVscode.commands.registered.get('devcontext.newNote');
    await addHandler?.(tempWorkspace);

    const notes = await realClient.listNotes(tempWorkspace);
    expect(notes.length).toBe(1);
    const noteId = notes[0]!.id;
    expect(notes[0]!.title).toBe('Authentication Design');
    expect(notes[0]!.tags).toEqual(['auth', 'security']);

    // 2.2 Show Note
    let openedDocContent = '';
    mockVscode.workspace.openTextDocument = async (options: unknown) => {
      openedDocContent = (options as { content: string }).content;
      return {} as import('vscode').TextDocument;
    };
    mockVscode.window.showTextDocument = async () => ({} as import('vscode').TextEditor);

    const showHandler = mockVscode.commands.registered.get('devcontext.showNote');
    await showHandler?.(noteId, tempWorkspace);
    expect(openedDocContent).toContain('# Authentication Design');
    expect(openedDocContent).toContain('Use OAuth2 with PKCE flow');

    // 2.3 Edit Note
    const editInputs = ['Authentication Design v2', 'Updated PKCE flow with refresh tokens'];
    let editIdx = 0;
    mockVscode.window.showInputBox = async () => editInputs[editIdx++];

    const editHandler = mockVscode.commands.registered.get('devcontext.editNote');
    await editHandler?.({ id: noteId, title: 'Authentication Design' });

    const noteAfterEdit = await realClient.getNote(tempWorkspace, noteId);
    expect(noteAfterEdit.record.title).toBe('Authentication Design v2');
    expect(noteAfterEdit.content_markdown).toBe('Updated PKCE flow with refresh tokens');

    // 2.4 Delete Note
    mockVscode.window.showWarningMessage = async () => 'Delete Note' as unknown;
    const deleteHandler = mockVscode.commands.registered.get('devcontext.deleteNote');
    await deleteHandler?.({ id: noteId, title: 'Authentication Design v2' });

    const notesAfterDelete = await realClient.listNotes(tempWorkspace);
    expect(notesAfterDelete.length).toBe(0);
  });

  it('Workflow 3: Architecture Decision (ADR) creation, show, and supersede', async () => {
    // 3.1 Create ADR 0001
    const adrInputs = [
      'ADR 0001: SQLite Storage Engine',
      'Need reliable local persistence',
      'Use SQLite with WAL mode',
      'Local-first, safe single-file storage',
    ];
    let adrIdx = 0;
    mockVscode.window.showInputBox = async () => adrInputs[adrIdx++];
    mockVscode.window.showQuickPick = async () => ({ label: 'accepted' }) as unknown;

    const newDecisionHandler = mockVscode.commands.registered.get('devcontext.newDecision');
    await newDecisionHandler?.(tempWorkspace);

    const decisions = await realClient.listDecisions(tempWorkspace);
    expect(decisions.length).toBe(1);
    const adr1 = decisions[0]!;
    expect(adr1.title).toBe('ADR 0001: SQLite Storage Engine');
    expect(adr1.status).toBe('accepted');

    // 3.2 Show Decision
    let adrDocContent = '';
    mockVscode.workspace.openTextDocument = async (options: unknown) => {
      adrDocContent = (options as { content: string }).content;
      return {} as import('vscode').TextDocument;
    };
    const showDecisionHandler = mockVscode.commands.registered.get('devcontext.showDecision');
    await showDecisionHandler?.(adr1.id, tempWorkspace);
    expect(adrDocContent).toContain('# ADR 0001: SQLite Storage Engine');
    expect(adrDocContent).toContain('Use SQLite with WAL mode');

    // 3.3 Create ADR 0002 superseding ADR 0001
    const adr2Inputs = [
      'ADR 0002: Dual Storage Engine',
      'Scaling needs require dual backend',
      'Support SQLite and DuckDB',
      'Higher complexity',
    ];
    let adr2Idx = 0;
    mockVscode.window.showInputBox = async () => adr2Inputs[adr2Idx++];
    mockVscode.window.showQuickPick = async () => ({ label: 'accepted' }) as unknown;

    await newDecisionHandler?.(tempWorkspace);

    const decisionsAfter = await realClient.listDecisions(tempWorkspace);
    expect(decisionsAfter.length).toBe(2);
    const adr2 = decisionsAfter.find((d) => d.title.includes('ADR 0002'))!;

    // Supersede ADR 0001 with ADR 0002
    mockVscode.window.showQuickPick = async () =>
      ({ label: adr2.title, id: adr2.id }) as unknown;

    const supersedeHandler = mockVscode.commands.registered.get('devcontext.supersedeDecision');
    await supersedeHandler?.({ id: adr1.id, title: adr1.title });

    const adr1Updated = await realClient.getDecision(tempWorkspace, adr1.id);
    expect(adr1Updated.status).toBe('superseded');
    expect(adr1Updated.superseded_by).toBe(adr2.id);
  });

  it('Workflow 4: Milestone and Log Entry Lifecycle', async () => {
    // 4.1 Milestone add
    const milestoneInputs = ['Milestone v1.0 Release', 'Production release candidate', '2026-12-31'];
    let mIdx = 0;
    mockVscode.window.showInputBox = async () => milestoneInputs[mIdx++];

    const newMilestoneHandler = mockVscode.commands.registered.get('devcontext.newMilestone');
    await newMilestoneHandler?.(tempWorkspace);

    const milestones = await realClient.listMilestones(tempWorkspace);
    expect(milestones.length).toBe(1);
    const milestone = milestones[0]!;
    expect(milestone.title).toBe('Milestone v1.0 Release');
    expect(milestone.status).toBe('planned');

    // 4.2 Milestone complete
    mockVscode.window.showQuickPick = async () =>
      ({ label: milestone.title, id: milestone.id }) as unknown;

    const completeMilestoneHandler = mockVscode.commands.registered.get('devcontext.completeMilestone');
    await completeMilestoneHandler?.();

    const milestonesAfter = await realClient.listMilestones(tempWorkspace);
    expect(milestonesAfter[0]!.status).toBe('completed');

    // 4.3 Log add
    const logInputs = ['Completed Database Schema Migration', 'All 2 migrations applied successfully'];
    let logIdx = 0;
    mockVscode.window.showInputBox = async () => logInputs[logIdx++];
    mockVscode.window.showQuickPick = async () => ({ label: 'refactor' }) as unknown;

    const newLogHandler = mockVscode.commands.registered.get('devcontext.newLog');
    await newLogHandler?.(tempWorkspace);

    const logs = await realClient.listLogs(tempWorkspace);
    expect(logs.length).toBe(1);
    expect(logs[0]!.title).toBe('Completed Database Schema Migration');
  });

  it('Workflow 5: Full-Text Search QuickPick with BM25 and snippets', async () => {
    // Add searchable note
    await realClient.addNote(
      tempWorkspace,
      'Search Target: Microservice Architecture',
      'Detailed discussion about event-driven architectures and message queues.',
      ['architecture', 'events']
    );

    let searchItems: import('vscode').QuickPickItem[] = [];
    mockVscode.window.createQuickPick = () => {
      let currentItems: import('vscode').QuickPickItem[] = [];
      const qp = {
        title: '',
        placeholder: '',
        get items() {
          return currentItems;
        },
        set items(val: import('vscode').QuickPickItem[]) {
          currentItems = val;
          searchItems = val;
        },
        onDidChangeValue: (cb: (val: string) => void) => {
          setTimeout(() => cb('Microservice'), 20);
          return { dispose: () => {} };
        },
        onDidAccept: (cb: () => void) => {
          setTimeout(() => cb(), 600);
          return { dispose: () => {} };
        },
        onDidHide: () => ({ dispose: () => {} }),
        show: () => {},
        hide: () => {},
        dispose: () => {},
        selectedItems: [] as import('vscode').QuickPickItem[],
      };
      return qp as unknown as import('vscode').QuickPick<import('vscode').QuickPickItem>;
    };

    const searchHandler = mockVscode.commands.registered.get('devcontext.search');
    await searchHandler?.(tempWorkspace);

    // Wait for debounce (150ms) and FTS5 search execution
    await new Promise((r) => setTimeout(r, 800));

    expect(searchItems.length).toBeGreaterThanOrEqual(1);
    const targetMatch = searchItems.find((item) =>
      item.label.includes('Search Target: Microservice Architecture')
    );
    expect(targetMatch).toBeDefined();
    expect(targetMatch?.detail).toContain('Microservice');
  });

  it('Workflow 6: Tree View Data Providers reflect persisted state', async () => {
    // 6.1 Overview Provider
    const overviewProvider = new OverviewTreeDataProvider(
      workspaceManager,
      () => realClient,
      () => ({
        status: 'ready',
        executablePath: realExecutablePath,
        cliVersion: '0.1.0',
        protocolVersion: '1.0',
        message: 'Connected',
      })
    );
    const overviewItems = await overviewProvider.getChildren();
    expect(overviewItems.length).toBeGreaterThanOrEqual(3);
    const projectItem = overviewItems[0]!;
    expect(projectItem.label).toContain('E2E Test Project');

    // 6.2 Notes Provider
    const notesProvider = new NotesTreeDataProvider(workspaceManager, () => realClient);
    const noteItems = await notesProvider.getChildren();
    expect(noteItems.length).toBeGreaterThanOrEqual(1);
    expect(noteItems[0]!.label).toContain('Search Target: Microservice Architecture');

    // 6.3 Decisions Provider
    const decisionsProvider = new DecisionsTreeDataProvider(workspaceManager, () => realClient);
    const decisionItems = await decisionsProvider.getChildren();
    expect(decisionItems.length).toBe(2);
    const adrItem = decisionItems.find((d) => (d.label as string).includes('ADR 0001'));
    expect(adrItem?.description).toContain('[SUPERSEDED]');

    // 6.4 Timeline Provider
    const timelineProvider = new TimelineTreeDataProvider(workspaceManager, () => realClient);
    const timelineItems = await timelineProvider.getChildren();
    expect(timelineItems.length).toBeGreaterThanOrEqual(3);
  });

  it('Workflow 7: Multi-root workspace isolation between independent projects', async () => {
    const tempRootA = fs.mkdtempSync(path.join(os.tmpdir(), 'dc-iso-a-'));
    const tempRootB = fs.mkdtempSync(path.join(os.tmpdir(), 'dc-iso-b-'));

    try {
      // Initialize Project A and Project B
      await realClient.init(tempRootA, 'Project Alpha');
      await realClient.init(tempRootB, 'Project Beta');

      // Add note exclusively to Project A
      await realClient.addNote(tempRootA, 'Alpha Secret Note', 'Only in Alpha', ['alpha']);

      // Query both projects
      const notesA = await realClient.listNotes(tempRootA);
      const notesB = await realClient.listNotes(tempRootB);

      expect(notesA.length).toBe(1);
      expect(notesA[0]!.title).toBe('Alpha Secret Note');
      expect(notesB.length).toBe(0);

      // Verify databases are isolated on disk
      const dbA = new Database(path.join(tempRootA, '.devcontext', 'devcontext.db'), { readonly: true });
      const dbB = new Database(path.join(tempRootB, '.devcontext', 'devcontext.db'), { readonly: true });

      const countA = dbA.query<{ count: number }, []>('SELECT count(*) as count FROM notes;').get()?.count;
      const countB = dbB.query<{ count: number }, []>('SELECT count(*) as count FROM notes;').get()?.count;

      dbA.close();
      dbB.close();

      expect(countA).toBe(1);
      expect(countB).toBe(0);
    } finally {
      fs.rmSync(tempRootA, { recursive: true, force: true });
      fs.rmSync(tempRootB, { recursive: true, force: true });
    }
  });

  it('Workflow 8: Read-only workspace protection blocks mutation commands', async () => {
    const readOnlyDir = fs.mkdtempSync(path.join(os.tmpdir(), 'dc-readonly-'));

    try {
      // Mock workspace trust as untrusted
      mockVscode.workspace.isTrusted = false;

      let errorNotice = '';
      mockVscode.window.showErrorMessage = async (msg: string) => {
        errorNotice = msg;
        return undefined;
      };

      const initHandler = mockVscode.commands.registered.get('devcontext.init');
      await initHandler?.(readOnlyDir);

      expect(errorNotice).toContain('read-only');
      const dbPath = path.join(readOnlyDir, '.devcontext', 'devcontext.db');
      expect(fs.existsSync(dbPath)).toBe(false);
    } finally {
      mockVscode.workspace.isTrusted = true;
      fs.rmSync(readOnlyDir, { recursive: true, force: true });
    }
  });

  it('Workflow 9: Direct SQLite database verification of persisted invariants', () => {
    const dbPath = path.join(tempWorkspace, '.devcontext', 'devcontext.db');
    const db = new Database(dbPath, { readonly: true });

    try {
      // 1. Verify WAL journal mode stored in DB header
      const journalMode = db.query<{ journal_mode: string }, []>('PRAGMA journal_mode;').get()?.journal_mode;
      expect(journalMode?.toLowerCase()).toBe('wal');

      // Verify no foreign key violations exist in the database
      const fkViolations = db.query('PRAGMA foreign_key_check;').all();
      expect(fkViolations.length).toBe(0);

      // 2. Verify schema migrations
      const schemaVer = db.query<{ version: number }, []>('SELECT max(version) as version FROM schema_migrations;').get()?.version;
      expect(schemaVer).toBeGreaterThanOrEqual(2);

      // 3. Verify records table integrity
      const recordsCount = db.query<{ count: number }, []>('SELECT count(*) as count FROM records;').get()?.count;
      expect(recordsCount).toBeGreaterThanOrEqual(4);

      // 4. Verify FTS5 virtual table synchronization
      const ftsMatches = db.query<{ count: number }, [string]>(
        'SELECT count(*) as count FROM records_fts WHERE records_fts MATCH ?;'
      ).get('Architecture')?.count;
      expect(ftsMatches).toBeGreaterThanOrEqual(1);
    } finally {
      db.close();
    }
  });
});
