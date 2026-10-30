import { describe, expect, it } from 'bun:test';
import { CoreClient } from '../src/core/client';
import type { DecisionSummary, HandshakeResult, NoteSummary, ProjectStatus, TimelineView } from '../src/types';
import { DecisionsTreeDataProvider } from '../src/views/decisionsView';
import { NotesTreeDataProvider } from '../src/views/notesView';
import { OverviewTreeDataProvider } from '../src/views/overviewView';
import { TimelineTreeDataProvider } from '../src/views/timelineView';
import { WorkspaceManager } from '../src/workspace/workspaceManager';
import { mockVscode } from './mockVscode';

describe('Tree View Data Providers', () => {
  it('OverviewTreeDataProvider displays core inactive state when handshake fails', async () => {
    mockVscode.workspace.workspaceFolders = [];
    const manager = new WorkspaceManager();

    const inactiveHandshake: HandshakeResult = {
      status: 'binary_missing',
      executablePath: null,
      cliVersion: null,
      protocolVersion: null,
      message: 'Executable not found in PATH',
    };

    const provider = new OverviewTreeDataProvider(
      manager,
      () => null,
      () => inactiveHandshake
    );

    const children = await provider.getChildren();
    expect(children.length).toBe(1);
    expect(children[0]?.label).toBe('DevContext Core Inactive');
    expect(children[0]?.description).toBe('binary_missing');

    manager.dispose();
  });

  it('OverviewTreeDataProvider displays uninitialized workspace item', async () => {
    mockVscode.workspace.workspaceFolders = [
      { uri: { fsPath: '/mock/workspace' }, name: 'MockApp' },
    ];
    const manager = new WorkspaceManager();

    const readyHandshake: HandshakeResult = {
      status: 'ready',
      executablePath: '/usr/bin/devcontext',
      cliVersion: '0.1.0',
      protocolVersion: '1.0',
      message: 'Connected',
    };

    const provider = new OverviewTreeDataProvider(
      manager,
      () => new CoreClient('devcontext'),
      () => readyHandshake
    );

    const children = await provider.getChildren();
    expect(children.length).toBe(1);
    expect(children[0]?.label).toBe('MockApp');
    expect(children[0]?.description).toContain('Not Initialized');

    manager.dispose();
  });

  it('NotesTreeDataProvider renders notes with tags and pinned icons', async () => {
    const manager = new WorkspaceManager();
    // Mock getWorkspaceProjects to return an initialized project
    manager.getWorkspaceProjects = () => [
      {
        uri: { fsPath: '/test/proj' } as unknown as import('vscode').Uri,
        name: 'TestProj',
        isInitialized: true,
        isReadOnly: false,
      },
    ];

    const mockNotes: NoteSummary[] = [
      {
        id: '11111111-1111-1111-1111-111111111111',
        title: 'Architecture Overview',
        category: null,
        tags: ['arch', 'design'],
        is_pinned: true,
        is_archived: false,
        created_at: '2026-10-30T10:00:00Z',
        updated_at: '2026-10-30T10:00:00Z',
      },
      {
        id: '22222222-2222-2222-2222-222222222222',
        title: 'Legacy Notes',
        category: null,
        tags: ['deprecated'],
        is_pinned: false,
        is_archived: true,
        created_at: '2026-10-29T10:00:00Z',
        updated_at: '2026-10-29T10:00:00Z',
      },
    ];

    const mockClient = new CoreClient('devcontext');
    mockClient.listNotes = async () => mockNotes;

    const provider = new NotesTreeDataProvider(manager, () => mockClient);
    const children = await provider.getChildren();

    expect(children.length).toBe(2);
    expect(children[0]?.label).toBe('Architecture Overview');
    expect(children[0]?.description).toBe('#arch #design');
    expect((children[0]?.iconPath as { id: string })?.id).toBe('pin');

    expect(children[1]?.label).toBe('Legacy Notes');
    expect((children[1]?.iconPath as { id: string })?.id).toBe('archive');

    manager.dispose();
  });

  it('DecisionsTreeDataProvider renders decisions with status badges and icons', async () => {
    const manager = new WorkspaceManager();
    manager.getWorkspaceProjects = () => [
      {
        uri: { fsPath: '/test/proj' } as unknown as import('vscode').Uri,
        name: 'TestProj',
        isInitialized: true,
        isReadOnly: false,
      },
    ];

    const mockDecisions: DecisionSummary[] = [
      {
        id: 'd1',
        title: 'ADR 0008: SQLite Storage',
        status: 'accepted',
        superseded_by: null,
        created_at: '2026-10-30T10:00:00Z',
        tags: ['storage'],
      },
      {
        id: 'd2',
        title: 'ADR 0002: Monolith',
        status: 'superseded',
        superseded_by: 'd1',
        created_at: '2026-10-29T10:00:00Z',
        tags: ['arch'],
      },
    ];

    const mockClient = new CoreClient('devcontext');
    mockClient.listDecisions = async () => mockDecisions;

    const provider = new DecisionsTreeDataProvider(manager, () => mockClient);
    const children = await provider.getChildren();

    expect(children.length).toBe(2);
    expect(children[0]?.label).toBe('ADR 0008: SQLite Storage');
    expect(children[0]?.description).toBe('[ACCEPTED]');
    expect((children[0]?.iconPath as { id: string })?.id).toBe('pass');

    expect(children[1]?.label).toBe('ADR 0002: Monolith');
    expect(children[1]?.description).toBe('[SUPERSEDED]');
    expect(children[1]?.tooltip).toContain('Superseded by: d1');
    expect((children[1]?.iconPath as { id: string })?.id).toBe('history');

    manager.dispose();
  });

  it('TimelineTreeDataProvider filters chronological events by event type', async () => {
    const manager = new WorkspaceManager();
    manager.getWorkspaceProjects = () => [
      {
        uri: { fsPath: '/test/proj' } as unknown as import('vscode').Uri,
        name: 'TestProj',
        isInitialized: true,
        isReadOnly: false,
      },
    ];

    const mockTimelineView: TimelineView = {
      is_truncated: false,
      events: [
        {
          timestamp: '2026-10-30T12:00:00Z',
          event_type: 'commit',
          title: 'feat: add FTS5 indexing',
          summary: null,
          author: 'Alice',
          identifier: 'a1b2c3d',
          metadata: {},
        },
        {
          timestamp: '2026-10-30T11:00:00Z',
          event_type: 'note',
          title: 'Investigation Notes',
          summary: 'Indexed 100 records',
          author: null,
          identifier: 'n1',
          metadata: {},
        },
      ],
    };

    let requestedEventType: string | undefined;
    const mockClient = new CoreClient('devcontext');
    mockClient.timeline = async (_root, opts) => {
      requestedEventType = opts?.eventType;
      return mockTimelineView;
    };

    const provider = new TimelineTreeDataProvider(manager, () => mockClient);
    const allChildren = await provider.getChildren();
    expect(allChildren.length).toBe(2);
    expect(requestedEventType).toBeUndefined();

    // Apply commit filter
    provider.setFilterType('commit');
    await provider.getChildren();
    expect(requestedEventType).toBe('commit');

    manager.dispose();
  });
});
