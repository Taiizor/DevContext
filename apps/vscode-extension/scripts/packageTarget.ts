import * as fs from 'node:fs';
import * as os from 'node:os';
import * as path from 'node:path';
import { spawnSync } from 'node:child_process';

const VALID_TARGETS = [
  'win32-x64',
  'win32-arm64',
  'linux-x64',
  'linux-arm64',
  'darwin-x64',
  'darwin-arm64',
] as const;

type TargetPlatform = (typeof VALID_TARGETS)[number];

const TARGET_TO_RUST: Record<TargetPlatform, string> = {
  'win32-x64': 'x86_64-pc-windows-msvc',
  'win32-arm64': 'aarch64-pc-windows-msvc',
  'linux-x64': 'x86_64-unknown-linux-gnu',
  'linux-arm64': 'aarch64-unknown-linux-gnu',
  'darwin-x64': 'x86_64-apple-darwin',
  'darwin-arm64': 'aarch64-apple-darwin',
};

function parseArgs(): { target: TargetPlatform; binaryPath?: string; outDir?: string } {
  const args = process.argv.slice(2);
  let target: TargetPlatform | undefined;
  let binaryPath: string | undefined;
  let outDir: string | undefined;

  for (let i = 0; i < args.length; i++) {
    const arg = args[i];
    if (arg === '--target' && i + 1 < args.length) {
      const val = args[++i];
      if (VALID_TARGETS.includes(val as TargetPlatform)) {
        target = val as TargetPlatform;
      } else {
        throw new Error(
          `Invalid target '${val}'. Supported targets: ${VALID_TARGETS.join(', ')}`
        );
      }
    } else if (arg === '--binary' && i + 1 < args.length) {
      binaryPath = path.resolve(args[++i]);
    } else if (arg === '--outDir' && i + 1 < args.length) {
      outDir = path.resolve(args[++i]);
    }
  }

  if (!target) {
    throw new Error(`Missing required --target argument. Supported: ${VALID_TARGETS.join(', ')}`);
  }

  return { target, binaryPath, outDir };
}

function resolveBinaryPath(
  target: TargetPlatform,
  explicitPath?: string,
  repoRoot: string = path.resolve(__dirname, '../../..')
): string {
  const binaryName = target.startsWith('win32-') ? 'devcontext.exe' : 'devcontext';

  if (explicitPath) {
    if (!fs.existsSync(explicitPath)) {
      throw new Error(`Specified binary path does not exist: ${explicitPath}`);
    }
    const stat = fs.statSync(explicitPath);
    if (!stat.isFile()) {
      throw new Error(`Specified binary path is not a file: ${explicitPath}`);
    }
    return explicitPath;
  }

  const rustTriple = TARGET_TO_RUST[target];
  const candidates = [
    path.join(repoRoot, 'target', rustTriple, 'release', binaryName),
    path.join(repoRoot, 'target', 'release', binaryName),
    path.join(repoRoot, 'target', 'debug', binaryName),
  ];

  for (const c of candidates) {
    if (fs.existsSync(c) && fs.statSync(c).isFile()) {
      return c;
    }
  }

  throw new Error(
    `No binary found for target '${target}'. Looked in:\n  ${candidates.join('\n  ')}\nProvide path explicitly via --binary.`
  );
}

export async function packageTarget(options?: {
  target?: TargetPlatform;
  binaryPath?: string;
  outDir?: string;
}): Promise<string> {
  const parsed = options && options.target ? (options as { target: TargetPlatform; binaryPath?: string; outDir?: string }) : parseArgs();
  const { target } = parsed;

  const extensionDir = path.resolve(__dirname, '..');
  const repoRoot = path.resolve(extensionDir, '../..');

  const pkgJsonPath = path.join(extensionDir, 'package.json');
  const pkgJson = JSON.parse(fs.readFileSync(pkgJsonPath, 'utf8'));
  const version = pkgJson.version;
  const binaryName = target.startsWith('win32-') ? 'devcontext.exe' : 'devcontext';

  const resolvedBinary = resolveBinaryPath(target, parsed.binaryPath, repoRoot);
  console.log(`[PackageTarget] Packaging target: ${target}`);
  console.log(`[PackageTarget] Using binary: ${resolvedBinary}`);

  // 1. Build extension bundle
  console.log('[PackageTarget] Building TypeScript extension bundle...');
  const buildResult = spawnSync('bun', ['run', 'build'], {
    cwd: extensionDir,
    stdio: 'inherit',
    shell: false,
  });
  if (buildResult.status !== 0) {
    throw new Error('Failed to build TypeScript extension bundle.');
  }

  // 2. Prepare isolated temporary staging directory
  const stagingDir = fs.mkdtempSync(path.join(os.tmpdir(), `devcontext-staging-${target}-`));
  console.log(`[PackageTarget] Staging directory: ${stagingDir}`);

  const outDir = parsed.outDir || path.join(repoRoot, 'dist-vsix');
  fs.mkdirSync(outDir, { recursive: true });
  const outVsixFilename = `devcontext-memory-${version}-${target}.vsix`;
  const outVsixPath = path.join(outDir, outVsixFilename);

  try {
    // Copy manifest
    fs.copyFileSync(pkgJsonPath, path.join(stagingDir, 'package.json'));

    // Copy dist/
    const stagedDist = path.join(stagingDir, 'dist');
    fs.mkdirSync(stagedDist, { recursive: true });
    fs.cpSync(path.join(extensionDir, 'dist'), stagedDist, { recursive: true });

    // Copy media/
    const mediaDir = path.join(extensionDir, 'media');
    if (fs.existsSync(mediaDir)) {
      const stagedMedia = path.join(stagingDir, 'media');
      fs.mkdirSync(stagedMedia, { recursive: true });
      fs.cpSync(mediaDir, stagedMedia, { recursive: true });
    }

    // Copy README & Licenses
    const extReadme = path.join(extensionDir, 'README.md');
    if (fs.existsSync(extReadme)) {
      fs.copyFileSync(extReadme, path.join(stagingDir, 'README.md'));
    } else {
      const rootReadme = path.join(repoRoot, 'README.md');
      if (fs.existsSync(rootReadme)) {
        fs.copyFileSync(rootReadme, path.join(stagingDir, 'README.md'));
      }
    }

    const rootLicense = path.join(repoRoot, 'LICENSE');
    if (fs.existsSync(rootLicense)) {
      fs.copyFileSync(rootLicense, path.join(stagingDir, 'LICENSE'));
    }
    const licenseMit = path.join(repoRoot, 'LICENSE-MIT');
    if (fs.existsSync(licenseMit)) {
      fs.copyFileSync(licenseMit, path.join(stagingDir, 'LICENSE-MIT'));
    }
    const licenseApache = path.join(repoRoot, 'LICENSE-APACHE');
    if (fs.existsSync(licenseApache)) {
      fs.copyFileSync(licenseApache, path.join(stagingDir, 'LICENSE-APACHE'));
    }

    // Write minimal strict .vscodeignore into stagingDir
    const vscodeIgnoreContent = [
      '.vscode/**',
      '.vscode-test/**',
      'scripts/**',
      'src/**',
      'tests/**',
      'tsconfig.json',
      'bun.lock',
      'bun.lockb',
      'node_modules/**',
      '*.map',
      '',
    ].join('\n');
    fs.writeFileSync(path.join(stagingDir, '.vscodeignore'), vscodeIgnoreContent, 'utf8');

    // Create bin/ and copy binary
    const stagedBinDir = path.join(stagingDir, 'bin');
    fs.mkdirSync(stagedBinDir, { recursive: true });
    const stagedBinaryPath = path.join(stagedBinDir, binaryName);
    fs.copyFileSync(resolvedBinary, stagedBinaryPath);

    // Apply Unix executable permissions if non-Windows target
    if (!target.startsWith('win32-')) {
      try {
        fs.chmodSync(stagedBinaryPath, 0o755);
      } catch (err) {
        console.warn(`[PackageTarget] Warning setting permissions on ${stagedBinaryPath}:`, err);
      }
    }

    // 3. Invoke vsce package
    console.log(`[PackageTarget] Executing vsce package for target ${target}...`);
    const vsceResult = spawnSync(
      'bunx',
      [
        '@vscode/vsce@4.1.0',
        'package',
        '--target',
        target,
        '--no-dependencies',
        '-o',
        outVsixPath,
      ],
      {
        cwd: stagingDir,
        stdio: 'inherit',
        shell: false,
      }
    );

    if (vsceResult.status !== 0) {
      throw new Error(`vsce package failed with exit code ${vsceResult.status}`);
    }

    const vsixStat = fs.statSync(outVsixPath);
    console.log(
      `[PackageTarget] Successfully created: ${outVsixPath} (${(vsixStat.size / (1024 * 1024)).toFixed(2)} MB)`
    );

    return outVsixPath;
  } finally {
    // Isolated cleanup: Remove staging directory
    try {
      fs.rmSync(stagingDir, { recursive: true, force: true });
      console.log(`[PackageTarget] Cleaned up staging directory: ${stagingDir}`);
    } catch (cleanErr) {
      console.warn('[PackageTarget] Warning cleaning staging directory:', cleanErr);
    }
  }
}

if (import.meta.main) {
  try {
    packageTarget().catch((err) => {
      console.error('[PackageTarget] Error:', err.message);
      process.exit(1);
    });
  } catch (err) {
    console.error('[PackageTarget] Error:', (err as Error).message);
    process.exit(1);
  }
}
