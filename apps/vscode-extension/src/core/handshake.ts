import { CoreClient } from './client';
import { CoreLocator } from './locator';
import type { HandshakeResult } from '../types';

/**
 * Expected protocol major version supported by this extension build.
 */
export const SUPPORTED_PROTOCOL_MAJOR = '1.';

/**
 * Performs version and health negotiation between the extension and the native core.
 */
export class CoreHandshake {
  /**
   * Executes handshake with the located core executable.
   */
  public static async verify(
    extensionPath?: string,
    workspaceRoot?: string,
    customPathEnv?: string
  ): Promise<HandshakeResult> {
    const executablePath = CoreLocator.findExecutable(
      extensionPath,
      workspaceRoot,
      customPathEnv
    );

    if (!executablePath) {
      return {
        status: 'binary_missing',
        executablePath: null,
        cliVersion: null,
        protocolVersion: null,
        message:
          'DevContext core executable was not found. Please verify PATH or configure "devcontext.core.executablePath" in settings.',
      };
    }

    try {
      const client = new CoreClient(executablePath);
      const doctorData = await client.doctor();

      const protocol = doctorData.protocol_version || 'unknown';
      const cliVersion = doctorData.cli_version || 'unknown';

      if (!protocol.startsWith(SUPPORTED_PROTOCOL_MAJOR)) {
        return {
          status: 'incompatible_version',
          executablePath,
          cliVersion,
          protocolVersion: protocol,
          message: `Incompatible DevContext core protocol '${protocol}'. Expected major version '${SUPPORTED_PROTOCOL_MAJOR}x'. Please upgrade devcontext.`,
        };
      }

      return {
        status: 'ready',
        executablePath,
        cliVersion,
        protocolVersion: protocol,
        message: `DevContext core v${cliVersion} (protocol ${protocol}) connected.`,
      };
    } catch (err) {
      return {
        status: 'execution_failed',
        executablePath,
        cliVersion: null,
        protocolVersion: null,
        message: `Failed to execute DevContext core at '${executablePath}': ${(err as Error).message}`,
      };
    }
  }
}
