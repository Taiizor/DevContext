import { describe, it, expect, beforeAll, afterAll } from 'bun:test';
import * as path from 'node:path';
import * as fs from 'node:fs';
import * as os from 'node:os';
import { execSync } from 'node:child_process';
import { WorkspaceManager } from '../src/workspace/workspaceManager';
import { registerReferenceCommands } from '../src/commands/referenceCommands';
import type { Reference } from '../src/types';
import { mockVscode } from './mockVscode';

describe('Git Commit Reference Actions (Phase 6 Verification)', () => {
  let gitRepoDir: string;
  let commit1Sha: string;
  let commit2Sha: string;
  let workspaceManager: WorkspaceManager;
  const mockContext: import('vscode').ExtensionContext = {
    subscriptions: [],
  } as unknown as import('vscode').ExtensionContext;

  beforeAll(() => {
    // 1. Create an isolated temporary Git repository with at least two real commits
    gitRepoDir = fs.mkdtempSync(path.join(os.tmpdir(), 'dc-git-commit-ref-'));
    
    // Initialize git repo
    execSync('git init -b main', { cwd: gitRepoDir });
    execSync('git config user.name "DevContext Tester"', { cwd: gitRepoDir });
    execSync('git config user.email "tester@devcontext.local"', { cwd: gitRepoDir });

    // Commit 1: Initial repository setup
    fs.writeFileSync(path.join(gitRepoDir, 'README.md'), '# DevContext Project\nInitial commit.\n', 'utf8');
    execSync('git add README.md', { cwd: gitRepoDir });
    execSync('git commit -m "feat: initial project structure"', { cwd: gitRepoDir });
    commit1Sha = execSync('git rev-parse HEAD', { cwd: gitRepoDir }).toString().trim();

    // Commit 2: Meaningful change with diff
    fs.appendFileSync(path.join(gitRepoDir, 'README.md'), '\nSecond line added in commit 2.\n', 'utf8');
    fs.writeFileSync(path.join(gitRepoDir, 'config.json'), '{\n  "version": 1\n}\n', 'utf8');
    execSync('git add README.md config.json', { cwd: gitRepoDir });
    execSync('git commit -m "feat: add config and update readme"', { cwd: gitRepoDir });
    commit2Sha = execSync('git rev-parse HEAD', { cwd: gitRepoDir }).toString().trim();

    expect(commit1Sha.length).toBe(40);
    expect(commit2Sha.length).toBe(40);
    expect(commit1Sha).not.toBe(commit2Sha);

    workspaceManager = new WorkspaceManager();
    workspaceManager.getTargetProjectRoot = async () => gitRepoDir;

    registerReferenceCommands(mockContext, workspaceManager, () => null);
  });

  afterAll(() => {
    try {
      fs.rmSync(gitRepoDir, { recursive: true, force: true });
    } catch {
      // ignore cleanup error
    }
  });

  it('opens a meaningful diff document in the editor for a valid Git commit reference', async () => {
    let openedDocOptions: { content: string; language?: string } | null = null;
    let showedDocument = false;

    mockVscode.workspace.openTextDocument = async (options: unknown) => {
      openedDocOptions = options as { content: string; language?: string };
      return {} as import('vscode').TextDocument;
    };

    mockVscode.window.showTextDocument = async () => {
      showedDocument = true;
      return {} as import('vscode').TextEditor;
    };

    const commitRef: Reference = {
      id: 'ref-c2',
      record_id: 'rec-1',
      ref_type: 'git_commit',
      target: commit2Sha,
      created_at: new Date().toISOString(),
    };

    const openRefHandler = mockVscode.commands.registered.get('devcontext.openReference');
    expect(openRefHandler).toBeDefined();

    await openRefHandler?.(commitRef, gitRepoDir);

    // Verify virtual document creation with diff language
    expect(openedDocOptions).not.toBeNull();
    expect(openedDocOptions?.language).toBe('diff');
    expect(showedDocument).toBe(true);

    // Verify meaningful commit content in the diff document:
    const diffContent = openedDocOptions?.content ?? '';
    expect(diffContent).toContain(commit2Sha);
    expect(diffContent).toContain('DevContext Tester');
    expect(diffContent).toContain('feat: add config and update readme');
    expect(diffContent).toContain('diff --git a/README.md b/README.md');
    expect(diffContent).toContain('+Second line added in commit 2.');
    expect(diffContent).toContain('diff --git a/config.json b/config.json');
  });

  it('rejects an invalid (non-hex) commit SHA with actionable error without crashing', async () => {
    let errorMessage = '';
    mockVscode.window.showErrorMessage = async (msg: string) => {
      errorMessage = msg;
      return undefined;
    };

    let openedDoc = false;
    mockVscode.workspace.openTextDocument = async () => {
      openedDoc = true;
      return {} as import('vscode').TextDocument;
    };

    const invalidRef: Reference = {
      id: 'ref-invalid',
      record_id: 'rec-1',
      ref_type: 'git_commit',
      target: 'not-a-valid-sha-xyz!',
      created_at: new Date().toISOString(),
    };

    const openRefHandler = mockVscode.commands.registered.get('devcontext.openReference');
    await openRefHandler?.(invalidRef, gitRepoDir);

    expect(errorMessage).toContain("Invalid Git commit hash 'not-a-valid-sha-xyz!'");
    expect(errorMessage).toContain('Expected 7-40 hexadecimal characters');
    expect(openedDoc).toBe(false);
  });

  it('reports unavailable commit SHA when hash does not exist in local repository', async () => {
    let errorMessage = '';
    mockVscode.window.showErrorMessage = async (msg: string) => {
      errorMessage = msg;
      return undefined;
    };

    let openedDoc = false;
    mockVscode.workspace.openTextDocument = async () => {
      openedDoc = true;
      return {} as import('vscode').TextDocument;
    };

    const missingSha = '0000000000000000000000000000000000000000';
    const missingRef: Reference = {
      id: 'ref-missing',
      record_id: 'rec-1',
      ref_type: 'git_commit',
      target: missingSha,
      created_at: new Date().toISOString(),
    };

    const openRefHandler = mockVscode.commands.registered.get('devcontext.openReference');
    await openRefHandler?.(missingRef, gitRepoDir);

    expect(errorMessage).toContain(`Commit '${missingSha}' was not found in the local Git repository.`);
    expect(openedDoc).toBe(false);
  });

  it('reports descriptive error when workspace is not a Git repository', async () => {
    const nonGitDir = fs.mkdtempSync(path.join(os.tmpdir(), 'dc-non-git-'));

    try {
      let errorMessage = '';
      mockVscode.window.showErrorMessage = async (msg: string) => {
        errorMessage = msg;
        return undefined;
      };

      let openedDoc = false;
      mockVscode.workspace.openTextDocument = async () => {
        openedDoc = true;
        return {} as import('vscode').TextDocument;
      };

      const ref: Reference = {
        id: 'ref-non-git',
        record_id: 'rec-1',
        ref_type: 'git_commit',
        target: commit1Sha,
        created_at: new Date().toISOString(),
      };

      const openRefHandler = mockVscode.commands.registered.get('devcontext.openReference');
      await openRefHandler?.(ref, nonGitDir);

      expect(errorMessage).toContain('is not a Git repository');
      expect(openedDoc).toBe(false);
    } finally {
      fs.rmSync(nonGitDir, { recursive: true, force: true });
    }
  });
});
