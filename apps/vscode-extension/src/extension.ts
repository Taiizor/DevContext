import * as vscode from 'vscode';
import { registerDecisionCommands } from './commands/decisionCommands';
import { registerDoctorCommand } from './commands/doctorCommand';
import { registerExportImportCommands } from './commands/exportImportCommands';
import { registerInitCommand } from './commands/init';
import { registerLogCommands } from './commands/logCommands';
import { registerMilestoneCommands } from './commands/milestoneCommands';
import { registerNoteCommands } from './commands/noteCommands';
import { registerReferenceCommands } from './commands/referenceCommands';
import { registerSearchCommand } from './commands/searchCommand';
import { CoreClient } from './core/client';
import { CoreHandshake } from './core/handshake';
import { DevContextStatusBar } from './statusbar/statusBar';
import type { HandshakeResult } from './types';
import { DecisionsTreeDataProvider } from './views/decisionsView';
import { NotesTreeDataProvider } from './views/notesView';
import { OverviewTreeDataProvider } from './views/overviewView';
import { TimelineTreeDataProvider } from './views/timelineView';
import { WorkspaceManager } from './workspace/workspaceManager';

export * from './types';

let currentClient: CoreClient | null = null;
let currentHandshake: HandshakeResult | null = null;

export async function activate(context: vscode.ExtensionContext): Promise<void> {
  const workspaceManager = new WorkspaceManager();
  context.subscriptions.push(workspaceManager);

  const getClient = (): CoreClient | null => currentClient;
  const getHandshake = (): HandshakeResult | null => currentHandshake;

  // 1. Initial Handshake (lightweight, asynchronous)
  const firstRoot = vscode.workspace.workspaceFolders?.[0]?.uri.fsPath;
  currentHandshake = await CoreHandshake.verify(context.extensionPath, firstRoot);

  if (currentHandshake.status === 'ready' && currentHandshake.executablePath) {
    currentClient = new CoreClient(currentHandshake.executablePath);
  }

  // 2. Register Views
  const overviewProvider = new OverviewTreeDataProvider(
    workspaceManager,
    getClient,
    getHandshake
  );
  context.subscriptions.push(
    vscode.window.registerTreeDataProvider('devcontext.overviewView', overviewProvider)
  );

  const notesProvider = new NotesTreeDataProvider(workspaceManager, getClient);
  context.subscriptions.push(
    vscode.window.registerTreeDataProvider('devcontext.notesView', notesProvider)
  );

  const decisionsProvider = new DecisionsTreeDataProvider(workspaceManager, getClient);
  context.subscriptions.push(
    vscode.window.registerTreeDataProvider('devcontext.decisionsView', decisionsProvider)
  );

  const timelineProvider = new TimelineTreeDataProvider(workspaceManager, getClient);
  context.subscriptions.push(
    vscode.window.registerTreeDataProvider('devcontext.timelineView', timelineProvider)
  );

  // 3. Register Commands
  registerInitCommand(context, workspaceManager, getClient);
  registerNoteCommands(context, workspaceManager, getClient);
  registerDecisionCommands(context, workspaceManager, getClient);
  registerMilestoneCommands(context, workspaceManager, getClient);
  registerLogCommands(context, workspaceManager, getClient);
  registerSearchCommand(context, workspaceManager, getClient);
  registerReferenceCommands(context, workspaceManager, getClient);
  registerExportImportCommands(context, workspaceManager, getClient);
  registerDoctorCommand(context, workspaceManager, getClient, getHandshake);

  // View refresh command
  context.subscriptions.push(
    vscode.commands.registerCommand('devcontext.refreshViews', () => {
      overviewProvider.refresh();
      notesProvider.refresh();
      decisionsProvider.refresh();
      timelineProvider.refresh();
    })
  );

  // Timeline filter commands
  context.subscriptions.push(
    vscode.commands.registerCommand('devcontext.timelineFilterAll', () => {
      timelineProvider.setFilterType(undefined);
    })
  );
  context.subscriptions.push(
    vscode.commands.registerCommand('devcontext.timelineFilterCommits', () => {
      timelineProvider.setFilterType('commit');
    })
  );
  context.subscriptions.push(
    vscode.commands.registerCommand('devcontext.timelineFilterNotes', () => {
      timelineProvider.setFilterType('note');
    })
  );
  context.subscriptions.push(
    vscode.commands.registerCommand('devcontext.timelineFilterDecisions', () => {
      timelineProvider.setFilterType('decision');
    })
  );

  // 4. Status Bar
  const statusBar = new DevContextStatusBar(workspaceManager, getClient, getHandshake);
  context.subscriptions.push(statusBar);

  // 5. Watch for configuration changes (e.g. core.executablePath changed)
  context.subscriptions.push(
    vscode.workspace.onDidChangeConfiguration(async (e) => {
      if (e.affectsConfiguration('devcontext.core.executablePath')) {
        currentHandshake = await CoreHandshake.verify(
          context.extensionPath,
          vscode.workspace.workspaceFolders?.[0]?.uri.fsPath
        );
        if (currentHandshake.status === 'ready' && currentHandshake.executablePath) {
          currentClient = new CoreClient(currentHandshake.executablePath);
        } else {
          currentClient = null;
        }
        vscode.commands.executeCommand('devcontext.refreshViews');
        statusBar.update();
      }
    })
  );
}

export function deactivate(): void {
  currentClient = null;
  currentHandshake = null;
}
