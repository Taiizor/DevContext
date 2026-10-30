import { describe, expect, it } from 'bun:test';
import { CoreClient, DevContextCoreError } from '../src/core/client';
import type { ProjectStatus } from '../src/types';

describe('CoreClient & JSON IPC Protocol', () => {
  it('instantiates client with executable path', () => {
    const client = new CoreClient('devcontext');
    expect(client.getExecutablePath()).toBe('devcontext');
    client.setExecutablePath('custom/path/devcontext');
    expect(client.getExecutablePath()).toBe('custom/path/devcontext');
  });

  it('rejects with PROCESS_SPAWN_ERROR when binary does not exist', async () => {
    const client = new CoreClient('/definitely/nonexistent/binary_xyz_123');
    try {
      await client.doctor();
      expect(true).toBe(false); // should not reach here
    } catch (err) {
      expect(err).toBeInstanceOf(DevContextCoreError);
      const coreErr = err as DevContextCoreError;
      expect(coreErr.code).toBe('PROCESS_SPAWN_ERROR');
      expect(coreErr.message).toContain('Failed to launch');
    }
  });

  it('correctly constructs DevContextCoreError with code and details', () => {
    const err = new DevContextCoreError('NOT_FOUND', 'Record 123 was not found', { id: 123 });
    expect(err.code).toBe('NOT_FOUND');
    expect(err.message).toBe('[NOT_FOUND] Record 123 was not found');
    expect(err.details).toEqual({ id: 123 });
    expect(err.name).toBe('DevContextCoreError');
  });

  it('parses valid JSON response envelope into typed data', () => {
    const rawEnvelope = JSON.stringify({
      success: true,
      data: {
        project_id: '11111111-1111-1111-1111-111111111111',
        name: 'Test Project',
        root_path: '/workspace',
        created_at: '2026-10-30T12:00:00Z',
        vcs_identifier: 'git:main:abc1234',
        stats: {
          notes_count: 5,
          decisions_count: 2,
          milestones_count: 1,
          log_entries_count: 3,
        },
      },
      error: null,
    });

    const parsed = JSON.parse(rawEnvelope) as { success: boolean; data: ProjectStatus };
    expect(parsed.success).toBe(true);
    expect(parsed.data.name).toBe('Test Project');
    expect(parsed.data.stats.notes_count).toBe(5);
    expect(parsed.data.stats.decisions_count).toBe(2);
  });

  it('parses error envelope into typed error code and message', () => {
    const rawErrorEnvelope = JSON.stringify({
      success: false,
      data: null,
      error: {
        code: 'VALIDATION_ERROR',
        message: 'Note title cannot be empty',
        details: { field: 'title' },
      },
    });

    const parsed = JSON.parse(rawErrorEnvelope) as {
      success: boolean;
      error: { code: string; message: string; details: unknown };
    };

    expect(parsed.success).toBe(false);
    expect(parsed.error.code).toBe('VALIDATION_ERROR');
    expect(parsed.error.message).toBe('Note title cannot be empty');
    expect(parsed.error.details).toEqual({ field: 'title' });
  });
});
