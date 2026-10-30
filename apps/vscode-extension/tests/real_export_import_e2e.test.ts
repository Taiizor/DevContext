import { describe, it, expect, beforeAll, afterAll } from 'bun:test';
import * as fs from 'node:fs';
import * as os from 'node:os';
import * as path from 'node:path';
import { CoreClient } from '../src/core/client';

describe('Real Core Export, Import, Inspect, and Data Safety (Phase 7)', () => {
  let cliBinaryPath: string;
  let tempWorkspace1: string;
  let tempWorkspace2: string;
  let client: CoreClient;

  beforeAll(() => {
    const isWindows = process.platform === 'win32';
    const binaryName = isWindows ? 'devcontext.exe' : 'devcontext';
    cliBinaryPath = path.resolve(
      __dirname,
      '../../../target/debug',
      binaryName
    );

    if (!fs.existsSync(cliBinaryPath)) {
      throw new Error(`Real core binary not found at '${cliBinaryPath}'. Build target/debug first.`);
    }

    client = new CoreClient(cliBinaryPath, 15000);

    tempWorkspace1 = fs.mkdtempSync(path.join(os.tmpdir(), 'dc-p7-src-'));
    tempWorkspace2 = fs.mkdtempSync(path.join(os.tmpdir(), 'dc-p7-dest-'));
  });

  afterAll(() => {
    try {
      fs.rmSync(tempWorkspace1, { recursive: true, force: true });
      fs.rmSync(tempWorkspace2, { recursive: true, force: true });
    } catch {
      // ignore
    }
  });

  it('1. Initializes project and seeds multiple record types with Turkish characters', async () => {
    await client.init(tempWorkspace1, 'Phase7SourceProject');
    const status = await client.status(tempWorkspace1);
    expect(status.name).toBe('Phase7SourceProject');

    // Add Note with Turkish characters and emoji
    await client.addNote(
      tempWorkspace1,
      'Türkçe Not Başlığı 🚀',
      'Veri güvenliği ve taşınabilirlik testi: ç, ğ, ı, ö, ş, ü.',
      ['türkçe', 'export-test']
    );

    // Add Architecture Decision
    await client.addDecision(tempWorkspace1, {
      title: 'ADR-0014: Portable Archive Format',
      status: 'accepted',
      context: 'Users require full export durability without vendor lock-in.',
      decision: 'Standard ZIP archive with root export-v1.json and Markdown files.',
      consequences: 'Zero proprietary lock-in; human-readable standalone docs.',
      alternatives: 'Single SQLite file, rejected as difficult to inspect in text editors.',
    });

    // Add Milestone
    await client.addMilestone(
      tempWorkspace1,
      'Milestone Beta 1.0',
      'Target for Phase 7 complete release.',
      '2026-11-01T00:00:00Z'
    );

    // Add Log Entry
    await client.addLog(
      tempWorkspace1,
      'Export Pipeline Implemented',
      'Verified ZIP, JSON, and Markdown generation with path redaction.',
      'milestone'
    );

    const postSeedStatus = await client.status(tempWorkspace1);
    expect(postSeedStatus.stats.notes_count).toBe(1);
    expect(postSeedStatus.stats.decisions_count).toBe(1);
    expect(postSeedStatus.stats.milestones_count).toBe(1);
    expect(postSeedStatus.stats.log_entries_count).toBe(1);
  });

  it('2. Previews export summary with accurate record counts and path redaction', async () => {
    const preview = await client.previewExport(tempWorkspace1, true);
    expect(preview.project_name).toBe('Phase7SourceProject');
    expect(preview.total_records).toBe(4);
    expect(preview.notes_count).toBe(1);
    expect(preview.decisions_count).toBe(1);
    expect(preview.milestones_count).toBe(1);
    expect(preview.logs_count).toBe(1);
    expect(preview.redact_paths).toBe(true);
  });

  it('3. Exports to versioned JSON and portable ZIP archive', async () => {
    const jsonOut = path.join(tempWorkspace1, 'bundle-v1.json');
    const zipOut = path.join(tempWorkspace1, 'bundle-v1.zip');

    const jsonRes = await client.export(tempWorkspace1, {
      format: 'json',
      outputPath: jsonOut,
      redactPaths: true,
    });
    expect(jsonRes.format).toBe('json');
    expect(jsonRes.total_records).toBe(4);
    expect(fs.existsSync(jsonOut)).toBe(true);

    const zipRes = await client.export(tempWorkspace1, {
      format: 'zip',
      outputPath: zipOut,
      redactPaths: true,
    });
    expect(zipRes.format).toBe('zip');
    expect(zipRes.total_records).toBe(4);
    expect(fs.existsSync(zipOut)).toBe(true);
  });

  it('4. Inspects active database and external export bundle before import', async () => {
    // Inspect active database
    const dbReport = await client.inspect(tempWorkspace1);
    expect(dbReport.source_type).toBe('database');
    expect(dbReport.total_records).toBe(4);
    expect(dbReport.project_name).toBe('Phase7SourceProject');

    // Inspect exported ZIP file
    const zipOut = path.join(tempWorkspace1, 'bundle-v1.zip');
    const zipReport = await client.inspect(tempWorkspace1, zipOut);
    expect(zipReport.source_type).toBe('zip_archive');
    expect(zipReport.schema_version).toBe('1.0.0');
    expect(zipReport.total_records).toBe(4);
    expect(zipReport.notes_count).toBe(1);
    expect(zipReport.decisions_count).toBe(1);
  });

  it('5. Imports export bundle into fresh workspace with dry-run and conflict handling', async () => {
    await client.init(tempWorkspace2, 'Phase7TargetProject');
    const zipOut = path.join(tempWorkspace1, 'bundle-v1.zip');

    // Dry-run import
    const dryRunRes = await client.import(tempWorkspace2, zipOut, {
      dryRun: true,
      onConflict: 'skip',
    });
    expect(dryRunRes.dry_run).toBe(true);
    expect(dryRunRes.total_incoming).toBe(4);
    expect(dryRunRes.imported_count).toBe(4);
    expect(dryRunRes.backup_path).toBeFalsy();

    // Actual confirmed import
    const importRes = await client.import(tempWorkspace2, zipOut, {
      dryRun: false,
      onConflict: 'skip',
    });
    expect(importRes.dry_run).toBe(false);
    expect(importRes.imported_count).toBe(4);
    expect(importRes.backup_path).toBeDefined();
    expect(fs.existsSync(importRes.backup_path!)).toBe(true);

    // Verify imported records in target project
    const targetStatus = await client.status(tempWorkspace2);
    expect(targetStatus.stats.notes_count).toBe(1);
    expect(targetStatus.stats.decisions_count).toBe(1);
    expect(targetStatus.stats.milestones_count).toBe(1);
    expect(targetStatus.stats.log_entries_count).toBe(1);

    // Search query matches imported Turkish content
    const searchRes = await client.search(tempWorkspace2, 'taşınabilirlik');
    expect(searchRes.length).toBeGreaterThanOrEqual(1);
    expect(searchRes[0]?.title).toContain('Türkçe Not Başlığı 🚀');
  });

  it('6. Deletes all project memory safely with --force, preserving backup snapshot', async () => {
    const deleteRes = await client.deleteData(tempWorkspace2, true);
    expect(deleteRes.deleted_records_count).toBe(4);
    expect(deleteRes.backup_path).toBeDefined();
    expect(fs.existsSync(deleteRes.backup_path!)).toBe(true);

    // Verify active database is now empty
    const clearedStatus = await client.status(tempWorkspace2);
    expect(clearedStatus.stats.notes_count).toBe(0);
    expect(clearedStatus.stats.decisions_count).toBe(0);
    expect(clearedStatus.stats.milestones_count).toBe(0);
    expect(clearedStatus.stats.log_entries_count).toBe(0);
  });
});
