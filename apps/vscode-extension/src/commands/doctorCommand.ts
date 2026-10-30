import * as vscode from 'vscode';
import type { CoreClient } from '../core/client';
import type { HandshakeResult } from '../types';
import type { WorkspaceManager } from '../workspace/workspaceManager';

export function registerDoctorCommand(
  context: vscode.ExtensionContext,
  workspaceManager: WorkspaceManager,
  getClient: () => CoreClient | null,
  getHandshake: () => HandshakeResult | null
): void {
  const outputChannel = vscode.window.createOutputChannel('DevContext Diagnostic');
  context.subscriptions.push(outputChannel);

  context.subscriptions.push(
    vscode.commands.registerCommand('devcontext.doctor', async () => {
      outputChannel.clear();
      outputChannel.show(true);
      outputChannel.appendLine('=== DevContext Diagnostic Doctor ===\n');

      const handshake = getHandshake();
      outputChannel.appendLine(`Handshake Status: ${handshake?.status || 'Not Executed'}`);
      outputChannel.appendLine(`Executable Path: ${handshake?.executablePath || 'Not Found'}`);
      outputChannel.appendLine(`CLI Version: ${handshake?.cliVersion || 'N/A'}`);
      outputChannel.appendLine(`Protocol Version: ${handshake?.protocolVersion || 'N/A'}`);
      outputChannel.appendLine(`Diagnostic: ${handshake?.message || 'N/A'}\n`);

      const client = getClient();
      if (!client) {
        outputChannel.appendLine('ERROR: Native core client could not be instantiated.');
        vscode.window.showErrorMessage(
          'DevContext core binary is missing or cannot be executed.'
        );
        return;
      }

      const projectRoot = await workspaceManager.getTargetProjectRoot();
      if (!projectRoot) {
        outputChannel.appendLine('Notice: No workspace folder is currently open.');
        return;
      }

      outputChannel.appendLine(`Target Workspace: ${projectRoot}`);

      try {
        const doctorData = await client.doctor(projectRoot);
        outputChannel.appendLine(`Project Detected: ${doctorData.project_detected}`);

        if (doctorData.project_info) {
          const info = doctorData.project_info;
          outputChannel.appendLine(`Schema Version: ${info.schema_version}`);
          outputChannel.appendLine(`Journal Mode: ${info.journal_mode}`);
          outputChannel.appendLine(`Foreign Keys Valid: ${info.foreign_keys_valid}`);
          outputChannel.appendLine(`Integrity Status: ${info.integrity_status}`);
        } else {
          outputChannel.appendLine('Project Info: .devcontext database is not yet initialized.');
        }

        outputChannel.appendLine('\nDiagnostic completed successfully with 0 errors.');
        vscode.window.showInformationMessage('DevContext doctor diagnostic completed.');
      } catch (err) {
        outputChannel.appendLine(`\nFAILED: ${(err as Error).message}`);
        vscode.window.showErrorMessage(
          `DevContext doctor diagnostic failed: ${(err as Error).message}`
        );
      }
    })
  );
}
