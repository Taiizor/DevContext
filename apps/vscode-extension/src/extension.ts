import type * as vscode from 'vscode';

/**
 * Standard typed JSON response envelope received from devcontext CLI.
 */
export interface CliEnvelope<T> {
  success: boolean;
  data: T | null;
  error: {
    code: string;
    message: string;
    details?: unknown;
  } | null;
}

/**
 * Version handshake metadata payload.
 */
export interface VersionInfo {
  version: string;
  protocol_version: string;
}

/**
 * Extension activation entry point.
 */
export function activate(context: vscode.ExtensionContext): void {
  // Phase 1 foundation: Register diagnostic command
  const doctorDisposable = {
    dispose: () => {
      // Cleanup placeholder
    },
  };

  context.subscriptions.push(doctorDisposable);
}

/**
 * Extension deactivation cleanup.
 */
export function deactivate(): void {
  // Cleanup resources
}
