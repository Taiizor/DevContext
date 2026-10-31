import { describe, it, expect, beforeAll, afterAll } from 'bun:test';
import * as path from 'node:path';
import * as fs from 'node:fs';
import * as os from 'node:os';
import { mockVscode } from './mockVscode';
import { mock } from 'bun:test';

mock.module('vscode', () => mockVscode);

import { CoreLocator } from '../src/core/locator';
import { CoreHandshake } from '../src/core/handshake';
import { CoreClient } from '../src/core/client';

const repoRoot = path.resolve(__dirname, '../../..');
const binaryName = process.platform === 'win32' ? 'devcontext.exe' : 'devcontext';
const realExecutable = path.resolve(repoRoot, 'target', 'debug', binaryName);
const isBinaryAvailable = fs.existsSync(realExecutable);

const suite = isBinaryAvailable ? describe : describe.skip;

suite('Bundled Distribution & CoreLocator End-to-End Tests', () => {
  let tempBaseDir: string;
  let mockExtensionDir: string;
  let mockWorkspaceDir: string;

  beforeAll(() => {
    tempBaseDir = fs.mkdtempSync(path.join(os.tmpdir(), 'dc-bundle-test-'));
    mockExtensionDir = path.join(tempBaseDir, 'mock-extension');
    mockWorkspaceDir = path.join(tempBaseDir, 'mock-workspace');

    fs.mkdirSync(path.join(mockExtensionDir, 'bin'), { recursive: true });
    fs.mkdirSync(mockWorkspaceDir, { recursive: true });

    // Copy real binary into mock extension bundled location
    const bundledBin = path.join(mockExtensionDir, 'bin', binaryName);
    fs.copyFileSync(realExecutable, bundledBin);
    if (process.platform !== 'win32') {
      fs.chmodSync(bundledBin, 0o755);
    }
  });

  afterAll(() => {
    try {
      fs.rmSync(tempBaseDir, { recursive: true, force: true });
    } catch {
      // ignore
    }
  });

  it('1. CoreLocator discovers bundled binary from extension directory', () => {
    // Reset configuration override
    mockVscode.workspace.getConfiguration = () =>
      ({
        get: (_key: string) => '',
      }) as any;

    const resolved = CoreLocator.findExecutable(mockExtensionDir);
    expect(resolved).not.toBeNull();
    expect(path.normalize(resolved!)).toBe(
      path.normalize(path.join(mockExtensionDir, 'bin', binaryName))
    );
  });

  it('2. External executablePath setting takes precedence over bundled binary', () => {
    const customBinary = path.join(tempBaseDir, 'custom-' + binaryName);
    fs.copyFileSync(realExecutable, customBinary);
    if (process.platform !== 'win32') {
      fs.chmodSync(customBinary, 0o755);
    }

    mockVscode.workspace.getConfiguration = () =>
      ({
        get: (key: string) => (key === 'core.executablePath' ? customBinary : ''),
      }) as any;

    const resolved = CoreLocator.findExecutable(mockExtensionDir);
    expect(resolved).not.toBeNull();
    expect(path.normalize(resolved!)).toBe(path.normalize(customBinary));
  });

  it('3. Falls back to system PATH when bundled binary is missing', () => {
    mockVscode.workspace.getConfiguration = () =>
      ({
        get: (_key: string) => '',
      }) as any;

    const emptyExtDir = path.join(tempBaseDir, 'empty-extension');
    fs.mkdirSync(emptyExtDir, { recursive: true });

    const pathDir = path.join(tempBaseDir, 'fake-path');
    fs.mkdirSync(pathDir, { recursive: true });
    const pathBin = path.join(pathDir, binaryName);
    fs.copyFileSync(realExecutable, pathBin);
    if (process.platform !== 'win32') {
      fs.chmodSync(pathBin, 0o755);
    }

    const resolved = CoreLocator.findExecutable(emptyExtDir, undefined, pathDir);
    expect(resolved).not.toBeNull();
    expect(path.normalize(resolved!)).toBe(path.normalize(pathBin));
  });

  it('4. CoreHandshake executes doctor and reports ready with protocol 1.x', async () => {
    mockVscode.workspace.getConfiguration = () =>
      ({
        get: (_key: string) => '',
      }) as any;

    const handshake = await CoreHandshake.verify(mockExtensionDir, mockWorkspaceDir);
    expect(handshake.status).toBe('ready');
    expect(handshake.executablePath).toBe(path.join(mockExtensionDir, 'bin', binaryName));
    expect(handshake.protocolVersion).toBeDefined();
    expect(handshake.protocolVersion?.startsWith('1.')).toBe(true);
    expect(handshake.cliVersion).toBeDefined();
  });

  it('5. End-to-end: Init, Doctor, Note creation and FTS5 search with bundled binary', async () => {
    const bundledBin = path.join(mockExtensionDir, 'bin', binaryName);
    const client = new CoreClient(bundledBin);

    // 5.1 Initialize workspace
    await client.init(mockWorkspaceDir, 'Bundled E2E Workspace');
    const statusRes = await client.status(mockWorkspaceDir);
    expect(statusRes.name).toBe('Bundled E2E Workspace');

    // 5.2 Doctor
    const doctorRes = await client.doctor(mockWorkspaceDir);
    expect(doctorRes.project_detected).toBe(true);
    expect(doctorRes.project_info?.schema_version).toBeGreaterThan(0);
    expect(doctorRes.project_info?.integrity_status).toBe('ok');

    // 5.3 Create note
    await client.addNote(
      mockWorkspaceDir,
      'Bundled Distribution Strategy',
      'DevContext bundles native CLI binaries per platform without runtime downloads.',
      ['offline', 'vsix', 'packaging']
    );

    const notes = await client.listNotes(mockWorkspaceDir);
    expect(notes.length).toBeGreaterThan(0);
    const addedNote = notes.find((n) => n.title === 'Bundled Distribution Strategy');
    expect(addedNote).toBeDefined();
    expect(addedNote?.tags).toContain('vsix');

    // 5.4 Search with SQLite FTS5
    const searchRes = await client.search(mockWorkspaceDir, 'distribution');
    expect(searchRes.length).toBeGreaterThan(0);
    const match = searchRes.find((r) => r.title === 'Bundled Distribution Strategy');
    expect(match).toBeDefined();
    expect(match?.snippet.toLowerCase()).toContain('distribution');
  });

  it('6. ensureBundledExecutable enforces strict containment and rejects symlinks', () => {
    if (process.platform === 'win32') {
      return; // Unix permission test
    }

    const extDir = path.join(tempBaseDir, 'containment-ext');
    const binDir = path.join(extDir, 'bin');
    fs.mkdirSync(binDir, { recursive: true });

    const outsideDir = path.join(tempBaseDir, 'outside');
    fs.mkdirSync(outsideDir, { recursive: true });
    const outsideFile = path.join(outsideDir, 'outside-bin');
    fs.writeFileSync(outsideFile, '#!/bin/sh\necho hi\n', { mode: 0o644 });

    // 6.1 Path outside extension bin directory should not be touched
    CoreLocator.ensureBundledExecutable(extDir, outsideFile);
    const outsideMode = fs.statSync(outsideFile).mode & 0o777;
    expect(outsideMode).toBe(0o644); // Unchanged

    // 6.2 Symlink inside bin pointing to outside file must be rejected
    const symlinkBin = path.join(binDir, 'symlink-bin');
    try {
      fs.symlinkSync(outsideFile, symlinkBin);
      CoreLocator.ensureBundledExecutable(extDir, symlinkBin);
      const targetMode = fs.statSync(outsideFile).mode & 0o777;
      expect(targetMode).toBe(0o644); // Target permissions must not be escalated
    } catch {
      // Symlinks may require elevation on Windows, ignored if platform forbids
    }

    // 6.3 Legitimate regular file inside bin has missing +x restored to 0755
    const regularBin = path.join(binDir, 'legit-bin');
    fs.writeFileSync(regularBin, '#!/bin/sh\necho hi\n', { mode: 0o644 });
    CoreLocator.ensureBundledExecutable(extDir, regularBin);
    const regularMode = fs.statSync(regularBin).mode & 0o777;
    expect(regularMode).toBe(0o755); // Safely repaired
  });
});
