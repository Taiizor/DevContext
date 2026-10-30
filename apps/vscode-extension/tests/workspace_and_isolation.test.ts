import { describe, expect, it } from 'bun:test';
import * as fs from 'node:fs';
import * as os from 'node:os';
import * as path from 'node:path';
import { mockVscode } from './mockVscode';
import { WorkspaceManager } from '../src/workspace/workspaceManager';

describe('WorkspaceManager & Multi-Root Isolation', () => {
  it('detects single-root workspace without merging', () => {
    const tempDir = fs.mkdtempSync(path.join(os.tmpdir(), 'dc-test-single-'));
    try {
      mockVscode.workspace.workspaceFolders = [
        { uri: { fsPath: tempDir }, name: 'SingleApp' },
      ];

      const manager = new WorkspaceManager();
      const projects = manager.getWorkspaceProjects();

      expect(projects.length).toBe(1);
      const proj = projects[0];
      expect(proj).toBeDefined();
      expect(proj?.name).toBe('SingleApp');
      expect(proj?.isInitialized).toBe(false);

      manager.dispose();
    } finally {
      fs.rmSync(tempDir, { recursive: true, force: true });
    }
  });

  it('detects initialized state when .devcontext/devcontext.db exists', () => {
    const tempDir = fs.mkdtempSync(path.join(os.tmpdir(), 'dc-test-init-'));
    try {
      const dcDir = path.join(tempDir, '.devcontext');
      fs.mkdirSync(dcDir);
      fs.writeFileSync(path.join(dcDir, 'devcontext.db'), 'mock-db-bytes');

      mockVscode.workspace.workspaceFolders = [
        { uri: { fsPath: tempDir }, name: 'InitializedApp' },
      ];

      const manager = new WorkspaceManager();
      const projects = manager.getWorkspaceProjects();

      expect(projects.length).toBe(1);
      const proj = projects[0];
      expect(proj?.isInitialized).toBe(true);

      manager.dispose();
    } finally {
      fs.rmSync(tempDir, { recursive: true, force: true });
    }
  });

  it('preserves multi-root workspace isolation with distinct project objects', () => {
    const dirA = fs.mkdtempSync(path.join(os.tmpdir(), 'dc-test-multi-a-'));
    const dirB = fs.mkdtempSync(path.join(os.tmpdir(), 'dc-test-multi-b-'));

    try {
      mockVscode.workspace.workspaceFolders = [
        { uri: { fsPath: dirA }, name: 'ProjectA' },
        { uri: { fsPath: dirB }, name: 'ProjectB' },
      ];

      const manager = new WorkspaceManager();
      const projects = manager.getWorkspaceProjects();

      expect(projects.length).toBe(2);
      expect(projects[0]?.name).toBe('ProjectA');
      expect(projects[1]?.name).toBe('ProjectB');
      expect(projects[0]?.uri.fsPath).not.toBe(projects[1]?.uri.fsPath);

      manager.dispose();
    } finally {
      fs.rmSync(dirA, { recursive: true, force: true });
      fs.rmSync(dirB, { recursive: true, force: true });
    }
  });

  it('respects untrusted workspace as read-only', () => {
    const tempDir = fs.mkdtempSync(path.join(os.tmpdir(), 'dc-test-ro-'));
    try {
      mockVscode.workspace.isTrusted = false;

      const manager = new WorkspaceManager();
      const isRo = manager.isWorkspaceReadOnly(tempDir);
      expect(isRo).toBe(true);

      mockVscode.workspace.isTrusted = true;
      const isWritable = manager.isWorkspaceReadOnly(tempDir);
      expect(isWritable).toBe(false);

      manager.dispose();
    } finally {
      fs.rmSync(tempDir, { recursive: true, force: true });
    }
  });

  it('routes target project root directly when single workspace exists', async () => {
    const tempDir = fs.mkdtempSync(path.join(os.tmpdir(), 'dc-test-target-'));
    try {
      mockVscode.workspace.workspaceFolders = [
        { uri: { fsPath: tempDir }, name: 'Single' },
      ];

      const manager = new WorkspaceManager();
      const target = await manager.getTargetProjectRoot();
      expect(target).toBe(tempDir);

      manager.dispose();
    } finally {
      fs.rmSync(tempDir, { recursive: true, force: true });
    }
  });
});
