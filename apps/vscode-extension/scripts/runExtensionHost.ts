import * as path from 'node:path';
import * as fs from 'node:fs';
import { runTests } from '@vscode/test-electron';

async function main() {
  const extensionDevelopmentPath = path.resolve(__dirname, '..');
  const extensionTestsPath = path.resolve(__dirname, '../tests/suite/index.js');
  const vscodeExecutablePath = 'D:/Program Files/Microsoft VS Code/Code.exe';

  const testUserDataDir = path.resolve(__dirname, '../.vscode-test-user-data');
  const testExtensionsDir = path.resolve(__dirname, '../.vscode-test-extensions');

  fs.mkdirSync(testUserDataDir, { recursive: true });
  fs.mkdirSync(testExtensionsDir, { recursive: true });

  console.log('Launching VS Code Extension Development Host with local Code.exe...');
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
    console.error('Failed to run VS Code Extension Host tests:', err);
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
