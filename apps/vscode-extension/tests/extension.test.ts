import { describe, expect, it } from 'bun:test';
import type { CliEnvelope, VersionInfo } from '../src/extension';

describe('VS Code Extension Protocol Foundation', () => {
  it('validates protocol envelope structure', () => {
    const sampleEnvelope: CliEnvelope<VersionInfo> = {
      success: true,
      data: {
        version: '0.1.0',
        protocol_version: '1.0',
      },
      error: null,
    };

    expect(sampleEnvelope.success).toBe(true);
    expect(sampleEnvelope.data?.version).toBe('0.1.0');
    expect(sampleEnvelope.data?.protocol_version).toBe('1.0');
    expect(sampleEnvelope.error).toBeNull();
  });

  it('validates error envelope structure', () => {
    const errorEnvelope: CliEnvelope<VersionInfo> = {
      success: false,
      data: null,
      error: {
        code: 'NOT_INITIALIZED',
        message: 'No project found',
      },
    };

    expect(errorEnvelope.success).toBe(false);
    expect(errorEnvelope.data).toBeNull();
    expect(errorEnvelope.error?.code).toBe('NOT_INITIALIZED');
  });
});
