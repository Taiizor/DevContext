import { describe, it, expect, beforeAll, afterAll } from 'bun:test';
import * as path from 'node:path';
import * as fs from 'node:fs';
import * as os from 'node:os';
import { CoreClient } from '../src/core/client';
import { CoreLocator } from '../src/core/locator';
import { CoreHandshake } from '../src/core/handshake';
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

suite('Real Rust Core CLI & JSON IPC Integration', () => {
  let tempDir: string;

  beforeAll(() => {
    if (!isBinaryAvailable) return;
    expect(fs.existsSync(realExecutablePath)).toBe(true);
    tempDir = fs.mkdtempSync(path.join(os.tmpdir(), 'dc-real-core-'));
  });

  afterAll(() => {
    if (!tempDir) return;
    try {
      fs.rmSync(tempDir, { recursive: true, force: true });
    } catch {
      // Ignore cleanup error
    }
  });

  describe('1. Executable Discovery Hierarchy', () => {
    it('resolves real executable via configured executablePath setting', () => {
      mockVscode.workspace.getConfiguration = () => ({
        get: (key: string) => {
          if (key === 'core.executablePath') return realExecutablePath;
          return undefined;
        },
      });

      const resolved = CoreLocator.findExecutable(extensionDir);
      expect(resolved).toBe(realExecutablePath);
    });

    it('resolves bundled binary for current platform when configured path is empty', () => {
      mockVscode.workspace.getConfiguration = () => ({
        get: () => '',
      });

      const platformKey = `${process.platform}-${os.arch()}`;
      const bundledDir = path.join(extensionDir, 'bin', platformKey);
      const binaryName = process.platform === 'win32' ? 'devcontext.exe' : 'devcontext';
      const bundledExe = path.join(bundledDir, binaryName);

      fs.mkdirSync(bundledDir, { recursive: true });
      fs.copyFileSync(realExecutablePath, bundledExe);

      try {
        const resolved = CoreLocator.findExecutable(extensionDir);
        expect(resolved).toBe(bundledExe);
      } finally {
        fs.rmSync(path.join(extensionDir, 'bin'), { recursive: true, force: true });
      }
    });

    it('resolves executable via PATH fallback when configured and bundled are absent', () => {
      mockVscode.workspace.getConfiguration = () => ({
        get: () => '',
      });

      const targetDir = path.dirname(realExecutablePath);
      const resolved = CoreLocator.findExecutable(extensionDir, undefined, targetDir);
      expect(resolved).not.toBeNull();
      expect(fs.existsSync(resolved!)).toBe(true);
    });

    it('returns null and does not throw when executable does not exist', () => {
      mockVscode.workspace.getConfiguration = () => ({
        get: () => '/nonexistent/path/devcontext.exe',
      });

      const resolved = CoreLocator.findExecutable(extensionDir, undefined, '');
      expect(resolved).toBeNull();
    });
  });

  describe('2. Version Handshake & Protocol Compatibility', () => {
    it('performs successful handshake against the real compiled Rust CLI', async () => {
      mockVscode.workspace.getConfiguration = () => ({
        get: (key: string) => (key === 'core.executablePath' ? realExecutablePath : ''),
      });

      const result = await CoreHandshake.verify(extensionDir);

      expect(result.status).toBe('ready');
      expect(result.cliVersion).toMatch(/\d+\.\d+\.\d+/);
      expect(result.protocolVersion).toBe('1.0');
      expect(result.executablePath).toBe(realExecutablePath);
      expect(result.message).toContain('DevContext core');
    });

    it('returns binary_missing when executable cannot be resolved', async () => {
      mockVscode.workspace.getConfiguration = () => ({
        get: () => '/nonexistent/devcontext.exe',
      });

      const result = await CoreHandshake.verify(extensionDir, undefined, '');
      expect(result.status).toBe('binary_missing');
      expect(result.executablePath).toBeNull();
      expect(result.message).toContain('DevContext core executable was not found');
    });
  });

  describe('3. Real JSON IPC Requests & Responses', () => {
    let client: CoreClient;

    beforeAll(() => {
      client = new CoreClient(realExecutablePath);
    });

    it('executes real devcontext init and creates on-disk SQLite database', async () => {
      await client.init(tempDir, 'RealTestWorkspace');

      const dbPath = path.join(tempDir, '.devcontext', 'devcontext.db');
      expect(fs.existsSync(dbPath)).toBe(true);
    });

    it('executes real devcontext status returning current database counts', async () => {
      const status = await client.status(tempDir);

      expect(status.name).toBe('RealTestWorkspace');
      expect(status.stats.notes_count).toBe(0);
      expect(status.stats.decisions_count).toBe(0);
      expect(status.stats.milestones_count).toBe(0);
      expect(status.stats.log_entries_count).toBe(0);
    });

    it('executes real devcontext doctor returning storage and schema health', async () => {
      const doctor = await client.doctor(tempDir);

      expect(doctor.cli_version).toBe('0.1.2');
      expect(doctor.protocol_version).toBe('1.0');
      expect(doctor.project_detected).toBe(true);
      expect(doctor.project_info).not.toBeNull();
      expect(doctor.project_info?.root_path).toBe(tempDir);
    });

    it('executes real devcontext note add and creates searchable note record', async () => {
      await client.addNote(
        tempDir,
        'Architecture Overview Note',
        'Detailed notes regarding local-first storage design.',
        ['architecture', 'storage']
      );

      const notes = await client.listNotes(tempDir);
      expect(notes.length).toBe(1);
      const note = notes[0]!;
      expect(note.title).toBe('Architecture Overview Note');
      expect(note.tags).toEqual(['architecture', 'storage']);

      const statusAfter = await client.status(tempDir);
      expect(statusAfter.stats.notes_count).toBe(1);
    });

    it('executes real devcontext decision add and records architectural decision', async () => {
      await client.addDecision(
        tempDir,
        'ADR 0020: Real IPC Verification',
        'Testing real JSON protocol with Rust core',
        'Use structured JSON IPC with --json flag',
        'Ensures offline and safe communication without webviews',
        'accepted'
      );

      const decisions = await client.listDecisions(tempDir);
      expect(decisions.length).toBe(1);
      const decision = decisions[0]!;
      expect(decision.title).toBe('ADR 0020: Real IPC Verification');
      expect(decision.status).toBe('accepted');

      const statusAfter = await client.status(tempDir);
      expect(statusAfter.stats.decisions_count).toBe(1);
    });

    it('executes real devcontext milestone add and milestone complete', async () => {
      await client.addMilestone(
        tempDir,
        'M1: Real Verification',
        'Verify real Rust execution'
      );

      const milestones = await client.listMilestones(tempDir);
      expect(milestones.length).toBe(1);
      const milestone = milestones[0]!;
      expect(milestone.title).toBe('M1: Real Verification');
      expect(milestone.status).toBe('planned');

      await client.completeMilestone(tempDir, milestone.id);

      const milestonesAfter = await client.listMilestones(tempDir);
      expect(milestonesAfter[0]!.status).toBe('completed');
    });

    it('executes real devcontext log add', async () => {
      await client.addLog(
        tempDir,
        'Log: Rust IPC Verified',
        'Verified end-to-end execution of CLI process with JSON envelope.',
        'feature'
      );

      const logs = await client.listLogs(tempDir);
      expect(logs.length).toBe(1);
      expect(logs[0]!.title).toBe('Log: Rust IPC Verified');

      const statusAfter = await client.status(tempDir);
      expect(statusAfter.stats.log_entries_count).toBe(1);
    });

    it('executes real devcontext search querying SQLite FTS5 index', async () => {
      const results = await client.search(tempDir, 'Architecture');

      expect(results.length).toBeGreaterThanOrEqual(1);
      const match = results[0];
      expect(match).toBeDefined();
      expect(match!.title).toContain('Architecture Overview Note');
      expect(match!.snippet).toBeDefined();
      expect(match!.record_type).toBe('note');
    });
  });

  describe('4. Process Termination, Timeout & Cancellation', () => {
    it('cancels real core process via CancellationToken AbortSignal', async () => {
      const client = new CoreClient(realExecutablePath, 15000);

      const cancellationToken: import('vscode').CancellationToken = {
        isCancellationRequested: false,
        onCancellationRequested: (listener) => {
          setTimeout(() => {
            (cancellationToken as { isCancellationRequested: boolean }).isCancellationRequested = true;
            listener(undefined);
          }, 1);
          return { dispose: () => {} };
        },
      };

      try {
        await client.status(tempDir, cancellationToken);
      } catch (err: unknown) {
        const error = err as { code: string };
        expect(error.code).toBe('COMMAND_CANCELLED');
      }
    });

    it('rejects with COMMAND_TIMEOUT when execution exceeds configured timeoutMs', async () => {
      const fastTimeoutClient = new CoreClient('bun.exe', 50);

      try {
        await fastTimeoutClient.execute([
          '-e',
          'const s = Date.now(); while(Date.now() - s < 1000) {} //',
        ]);
        expect.unreachable('Should have timed out');
      } catch (err: unknown) {
        const error = err as { code: string };
        expect(error.code).toBe('COMMAND_TIMEOUT');
      }
    });
  });
});
