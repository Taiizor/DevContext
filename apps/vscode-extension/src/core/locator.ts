import * as fs from 'node:fs';
import * as os from 'node:os';
import * as path from 'node:path';
import * as vscode from 'vscode';

/**
 * Resolves the path to the native DevContext core executable
 * following the layered resolution strategy defined in ADR 0012.
 */
export class CoreLocator {
  /**
   * Resolves the core executable path.
   *
   * @param extensionPath The filesystem path of the installed extension.
   * @param workspaceRoot Optional workspace root directory for development target lookup.
   * @returns Absolute path to a valid executable, or null if not found.
   */
  public static findExecutable(
    extensionPath?: string,
    workspaceRoot?: string,
    customPathEnv?: string
  ): string | null {
    const isWindows = process.platform === 'win32';
    const binaryName = isWindows ? 'devcontext.exe' : 'devcontext';

    // 1. Explicit configuration setting: devcontext.core.executablePath
    const config = vscode.workspace.getConfiguration('devcontext');
    const configuredPath = config.get<string>('core.executablePath')?.trim();
    if (configuredPath && configuredPath.length > 0) {
      const resolvedConfigured = path.isAbsolute(configuredPath)
        ? configuredPath
        : workspaceRoot
          ? path.resolve(workspaceRoot, configuredPath)
          : path.resolve(configuredPath);

      if (this.isExecutableFile(resolvedConfigured)) {
        return resolvedConfigured;
      }
    }

    // 2. Bundled platform-specific binary in extension directory: bin/<platform>-<arch>/devcontext
    if (extensionPath) {
      const platformKey = `${process.platform}-${os.arch()}`;
      const bundledPath = path.join(extensionPath, 'bin', platformKey, binaryName);
      this.ensureBundledExecutable(extensionPath, bundledPath);
      if (this.isExecutableFile(bundledPath)) {
        return bundledPath;
      }

      // Fallback for unarchived or flat bin/ layout
      const flatBundledPath = path.join(extensionPath, 'bin', binaryName);
      this.ensureBundledExecutable(extensionPath, flatBundledPath);
      if (this.isExecutableFile(flatBundledPath)) {
        return flatBundledPath;
      }
    }

    // 3. Development workspace build outputs (target/debug or target/release)
    if (workspaceRoot) {
      const debugPath = path.join(workspaceRoot, 'target', 'debug', binaryName);
      if (this.isExecutableFile(debugPath)) {
        return debugPath;
      }

      const releasePath = path.join(workspaceRoot, 'target', 'release', binaryName);
      if (this.isExecutableFile(releasePath)) {
        return releasePath;
      }
    }

    // 4. System PATH search
    const systemPath = this.searchSystemPath(binaryName, customPathEnv);
    if (systemPath) {
      return systemPath;
    }

    return null;
  }

  /**
   * Safely ensures executable permissions (0755) on a candidate bundled binary.
   *
   * Strict safety constraints:
   * 1. Only applies on non-Windows platforms.
   * 2. Only applies if the candidate file exists and is a regular file.
   * 3. Explicitly rejects symlinks (!lstat.isSymbolicLink()).
   * 4. Enforces strict path containment: canonical candidate path must reside inside
   *    canonical extensionPath/bin directory.
   * 5. Only modifies permissions if execution permission (X_OK) is missing.
   * 6. Never touches user-configured paths or system PATH.
   */
  public static ensureBundledExecutable(extensionPath: string, candidatePath: string): void {
    if (process.platform === 'win32') {
      return;
    }

    try {
      if (!fs.existsSync(candidatePath)) {
        return;
      }

      const lstat = fs.lstatSync(candidatePath);
      if (lstat.isSymbolicLink() || !lstat.isFile()) {
        return;
      }

      const realExt = fs.realpathSync(extensionPath);
      const realCandidate = fs.realpathSync(candidatePath);
      const allowedBinPrefix = path.join(realExt, 'bin') + path.sep;

      if (!realCandidate.startsWith(allowedBinPrefix)) {
        return;
      }

      // Check if already executable
      try {
        fs.accessSync(realCandidate, fs.constants.X_OK);
        return;
      } catch {
        // Missing execute permission: self-heal by applying 0755 exclusively to bundled binary
        fs.chmodSync(realCandidate, 0o755);
      }
    } catch {
      // Ignore errors; resolution will safely proceed or report missing
    }
  }

  /**
   * Searches the directories listed in PATH for the binary.
   */
  private static searchSystemPath(
    binaryName: string,
    envPath: string = process.env.PATH || ''
  ): string | null {
    const pathSeparator = process.platform === 'win32' ? ';' : ':';
    const directories = envPath.split(pathSeparator);

    for (const dir of directories) {
      if (!dir || dir.trim() === '') {
        continue;
      }
      const fullPath = path.join(dir.trim(), binaryName);
      if (this.isExecutableFile(fullPath)) {
        return fullPath;
      }
    }

    return null;
  }

  /**
   * Verifies that the given path exists, is a file, and has executable permissions.
   */
  public static isExecutableFile(filePath: string): boolean {
    try {
      if (!fs.existsSync(filePath)) {
        return false;
      }
      const stat = fs.statSync(filePath);
      if (!stat.isFile()) {
        return false;
      }

      if (process.platform === 'win32') {
        // On Windows, existence as a file is sufficient
        return true;
      }

      // On Unix, check execute permission bits
      fs.accessSync(filePath, fs.constants.X_OK);
      return true;
    } catch {
      return false;
    }
  }
}
