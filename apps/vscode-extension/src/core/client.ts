import { spawn } from 'node:child_process';
import type * as vscode from 'vscode';
import type {
  CliEnvelope,
  DecisionDetail,
  DecisionSummary,
  DoctorData,
  LogEntrySummary,
  MilestoneSummary,
  NoteDetail,
  NoteSummary,
  ProjectStatus,
  SearchResult,
  TimelineView,
} from '../types';

/**
 * Custom error thrown when a core operation fails.
 */
export class DevContextCoreError extends Error {
  public readonly code: string;
  public readonly details: unknown;

  constructor(code: string, message: string, details?: unknown) {
    super(`[${code}] ${message}`);
    this.name = 'DevContextCoreError';
    this.code = code;
    this.details = details;
  }
}

/**
 * Manages typed, asynchronous execution of the native devcontext executable.
 */
export class CoreClient {
  private executablePath: string;
  private readonly timeoutMs: number;

  constructor(executablePath: string, timeoutMs: number = 30000) {
    this.executablePath = executablePath;
    this.timeoutMs = timeoutMs;
  }

  public setExecutablePath(newPath: string): void {
    this.executablePath = newPath;
  }

  public getExecutablePath(): string {
    return this.executablePath;
  }

  /**
   * Spawns devcontext with structured arguments and parses the JSON response envelope.
   */
  public async execute<T>(
    args: string[],
    projectDir?: string,
    cancellationToken?: vscode.CancellationToken
  ): Promise<T> {
    const fullArgs: string[] = [];

    if (projectDir) {
      fullArgs.push('--project-dir', projectDir);
    }

    fullArgs.push(...args);

    if (!fullArgs.includes('--json')) {
      fullArgs.push('--json');
    }

    return new Promise<T>((resolve, reject) => {
      let cancelled = false;

      const child = spawn(this.executablePath, fullArgs, {
        shell: false,
        windowsHide: true,
      });

      let timeoutHandle: ReturnType<typeof setTimeout> | null = null;
      if (this.timeoutMs > 0) {
        timeoutHandle = setTimeout(() => {
          cancelled = true;
          try {
            child.kill();
          } catch {
            // ignore kill errors
          }
          reject(
            new DevContextCoreError(
              'COMMAND_TIMEOUT',
              `Command '${this.executablePath}' timed out after ${this.timeoutMs}ms.`
            )
          );
        }, this.timeoutMs);
      }

      if (cancellationToken) {
        cancellationToken.onCancellationRequested(() => {
          cancelled = true;
          if (timeoutHandle) {
            clearTimeout(timeoutHandle);
          }
          try {
            child.kill();
          } catch {
            // ignore kill errors
          }
          reject(new DevContextCoreError('COMMAND_CANCELLED', 'Operation was cancelled'));
        });
      }

      let stdout = '';
      let stderr = '';

      child.stdout.on('data', (chunk) => {
        stdout += chunk.toString();
      });

      child.stderr.on('data', (chunk) => {
        stderr += chunk.toString();
      });

      child.on('error', (err) => {
        if (timeoutHandle) {
          clearTimeout(timeoutHandle);
        }
        if (!cancelled) {
          reject(
            new DevContextCoreError(
              'PROCESS_SPAWN_ERROR',
              `Failed to launch '${this.executablePath}': ${err.message}`
            )
          );
        }
      });

      child.on('close', (_code) => {
        if (timeoutHandle) {
          clearTimeout(timeoutHandle);
        }
        if (cancelled) {
          return;
        }

        const trimmedStdout = stdout.trim();
        if (!trimmedStdout) {
          reject(
            new DevContextCoreError(
              'EMPTY_RESPONSE',
              `Process returned no output. Stderr: ${stderr.trim()}`
            )
          );
          return;
        }

        try {
          const envelope = JSON.parse(trimmedStdout) as CliEnvelope<T>;
          if (envelope.success && envelope.data !== null) {
            resolve(envelope.data);
          } else if (envelope.error) {
            reject(
              new DevContextCoreError(
                envelope.error.code,
                envelope.error.message,
                envelope.error.details
              )
            );
          } else {
            reject(
              new DevContextCoreError(
                'UNKNOWN_CLI_RESPONSE',
                'Response indicated failure with no error object.'
              )
            );
          }
        } catch (parseErr) {
          reject(
            new DevContextCoreError(
              'INVALID_JSON_RESPONSE',
              `Failed to parse CLI JSON output: ${(parseErr as Error).message}\nRaw output: ${trimmedStdout}`
            )
          );
        }
      });
    });
  }

  // --- High-level typed API methods ---

  public async doctor(projectDir?: string): Promise<DoctorData> {
    return this.execute<DoctorData>(['doctor'], projectDir);
  }

  public async status(projectDir: string): Promise<ProjectStatus> {
    return this.execute<ProjectStatus>(['status'], projectDir);
  }

  public async init(projectDir: string, name?: string): Promise<void> {
    const args = ['init'];
    if (name && name.trim()) {
      args.push('--name', name.trim());
    }
    await this.execute<unknown>(args, projectDir);
  }

  public async listNotes(projectDir: string): Promise<NoteSummary[]> {
    return this.execute<NoteSummary[]>(['note', 'list'], projectDir);
  }

  public async getNote(projectDir: string, id: string): Promise<NoteDetail> {
    return this.execute<NoteDetail>(['note', 'show', id], projectDir);
  }

  public async addNote(
    projectDir: string,
    title: string,
    content: string,
    tags: string[] = [],
    linkCommit?: string
  ): Promise<void> {
    const args = ['note', 'add', title, '--content', content];
    for (const tag of tags) {
      if (tag.trim()) {
        args.push('--tag', tag.trim());
      }
    }
    if (linkCommit && linkCommit.trim()) {
      args.push('--link-commit', linkCommit.trim());
    }
    await this.execute<unknown>(args, projectDir);
  }

  public async editNote(
    projectDir: string,
    id: string,
    options: { title?: string; content?: string }
  ): Promise<void> {
    const args = ['note', 'edit', id];
    if (options.title) {
      args.push('--title', options.title);
    }
    if (options.content) {
      args.push('--content', options.content);
    }
    await this.execute<unknown>(args, projectDir);
  }

  public async deleteNote(projectDir: string, id: string): Promise<void> {
    await this.execute<unknown>(['note', 'delete', id], projectDir);
  }

  public async listDecisions(projectDir: string): Promise<DecisionSummary[]> {
    return this.execute<DecisionSummary[]>(['decision', 'list'], projectDir);
  }

  public async getDecision(projectDir: string, id: string): Promise<DecisionDetail> {
    return this.execute<DecisionDetail>(['decision', 'show', id], projectDir);
  }

  public async addDecision(
    projectDir: string,
    title: string,
    context: string,
    decision: string,
    consequences: string,
    status?: string
  ): Promise<void> {
    const args = [
      'decision',
      'add',
      title,
      '--context',
      context,
      '--decision',
      decision,
      '--consequences',
      consequences,
    ];
    if (status) {
      args.push('--status', status);
    }
    await this.execute<unknown>(args, projectDir);
  }

  public async supersedeDecision(
    projectDir: string,
    id: string,
    supersededBy: string
  ): Promise<void> {
    await this.execute<unknown>(
      ['decision', 'update', id, '--superseded-by', supersededBy],
      projectDir
    );
  }

  public async listMilestones(projectDir: string): Promise<MilestoneSummary[]> {
    return this.execute<MilestoneSummary[]>(['milestone', 'list'], projectDir);
  }

  public async addMilestone(
    projectDir: string,
    title: string,
    description?: string,
    targetDate?: string
  ): Promise<void> {
    const args = ['milestone', 'add', title];
    if (description) {
      args.push('--description', description);
    }
    if (targetDate) {
      args.push('--target-date', targetDate);
    }
    await this.execute<unknown>(args, projectDir);
  }

  public async completeMilestone(projectDir: string, id: string): Promise<void> {
    await this.execute<unknown>(['milestone', 'complete', id], projectDir);
  }

  public async listLogs(projectDir: string): Promise<LogEntrySummary[]> {
    return this.execute<LogEntrySummary[]>(['log', 'list'], projectDir);
  }

  public async addLog(
    projectDir: string,
    summary: string,
    details?: string,
    category?: string
  ): Promise<void> {
    const args = ['log', 'add', summary, '--summary', summary];
    if (details) {
      args.push('--details', details);
    }
    if (category) {
      args.push('--tag', category);
    }
    await this.execute<unknown>(args, projectDir);
  }

  public async timeline(
    projectDir: string,
    options: {
      limit?: number;
      eventType?: string;
      since?: string;
      until?: string;
    } = {}
  ): Promise<TimelineView> {
    const args = ['timeline'];
    if (options.limit) {
      args.push('--limit', options.limit.toString());
    }
    if (options.eventType) {
      args.push('--type', options.eventType);
    }
    if (options.since) {
      args.push('--since', options.since);
    }
    if (options.until) {
      args.push('--until', options.until);
    }
    return this.execute<TimelineView>(args, projectDir);
  }

  public async search(
    projectDir: string,
    query: string,
    options: {
      recordType?: string;
      limit?: number;
      reindex?: boolean;
    } = {}
  ): Promise<SearchResult[]> {
    const args = ['search'];
    if (options.reindex) {
      args.push('--reindex');
    }
    if (query) {
      args.push(query);
    }
    if (options.recordType) {
      args.push('--type', options.recordType);
    }
    if (options.limit) {
      args.push('--limit', options.limit.toString());
    }
    return this.execute<SearchResult[]>(args, projectDir);
  }
}
