import * as path from 'node:path';
import * as fs from 'node:fs';
import * as os from 'node:os';
import { runTests } from '@vscode/test-electron';

function resolveVSCodeExecutable(): string | undefined {
  // 1. Respect VSCODE_PATH when explicitly provided
  const envPath = process.env.VSCODE_PATH?.trim();
  if (envPath) {
    if (fs.existsSync(envPath) && fs.statSync(envPath).isFile()) {
      return envPath;
    }
    console.warn(`[ExtensionHost] Provided VSCODE_PATH '${envPath}' was not found or is not a file.`);
  }

  // 2. Detect standard installation locations per platform
  const platform = process.platform;
  const candidates: string[] = [];

  if (platform === 'win32') {
    const localAppData = process.env.LOCALAPPDATA || path.join(os.homedir(), 'AppData', 'Local');
    const programFiles = process.env.ProgramFiles || 'C:\\Program Files';
    const programFilesX86 = process.env['ProgramFiles(x86)'] || 'C:\\Program Files (x86)';

    candidates.push(
      path.join(localAppData, 'Programs', 'Microsoft VS Code', 'Code.exe'),
      path.join(programFiles, 'Microsoft VS Code', 'Code.exe'),
      path.join(programFilesX86, 'Microsoft VS Code', 'Code.exe'),
      'D:\\Program Files\\Microsoft VS Code\\Code.exe'
    );
  } else if (platform === 'darwin') {
    candidates.push(
      '/Applications/Visual Studio Code.app/Contents/MacOS/Electron',
      path.join(os.homedir(), 'Applications', 'Visual Studio Code.app', 'Contents', 'MacOS', 'Electron')
    );
  } else if (platform === 'linux') {
    candidates.push(
      '/usr/bin/code',
      '/usr/share/code/code',
      '/snap/bin/code',
      path.join(os.homedir(), '.local', 'bin', 'code')
    );
  }

  for (const candidate of candidates) {
    try {
      if (fs.existsSync(candidate) && fs.statSync(candidate).isFile()) {
        return candidate;
      }
    } catch {
      // Continue searching
    }
  }

  // 3. Fallback: undefined delegates to @vscode/test-electron automatic download
  return undefined;
}

async function main() {
  const extensionDevelopmentPath = path.resolve(__dirname, '..');
  const extensionTestsPath = path.resolve(__dirname, '../tests/suite/index.js');
  const vscodeExecutablePath = resolveVSCodeExecutable();

  const testUserDataDir = path.resolve(__dirname, '../.vscode-test-user-data');
  const testExtensionsDir = path.resolve(__dirname, '../.vscode-test-extensions');

  fs.mkdirSync(testUserDataDir, { recursive: true });
  fs.mkdirSync(testExtensionsDir, { recursive: true });

  if (vscodeExecutablePath) {
    console.log(`[ExtensionHost] Launching VS Code Extension Development Host using detected executable: ${vscodeExecutablePath}`);
  } else {
    console.log('[ExtensionHost] No local VS Code installation found in standard locations. Utilizing @vscode/test-electron automatic download...');
  }

  try {
    await runTests({
      vscodeExecutablePath,
      extensionDevelopmentPath,
      extensionTestsPath,
      launchArgs: [
        '--disable-extensions',
        '--disable-gpu',
        '--no-sandbox',
        '--user-data-dir=' + testUserDataDir,
        '--extensions-dir=' + testExtensionsDir,
      ],
    });
    console.log('VS Code Extension Host tests completed successfully!');
  } catch (err) {
    console.error('Failed to run VS Code Extension Host tests (all discovery and download methods failed):', err);
    process.exit(1);
  } finally {
    try {
      fs.rmSync(testUserDataDir, { recursive: true, force: true });
      fs.rmSync(testExtensionsDir, { recursive: true, force: true });
    } catch {
      // ignore cleanup errors
    }
  }
}

main();
