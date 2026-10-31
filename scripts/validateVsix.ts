import * as fs from 'node:fs';
import * as path from 'node:path';
import * as zlib from 'node:zlib';

export const EXPECTED_ARCHITECTURES = {
  'win32-x64': { format: 'PE', expectedValue: 0x8664, label: 'PE x64' },
  'win32-arm64': { format: 'PE', expectedValue: 0xaa64, label: 'PE ARM64' },
  'linux-x64': { format: 'ELF', expectedValue: 0x3e, label: 'ELF x64' },
  'linux-arm64': { format: 'ELF', expectedValue: 0xb7, label: 'ELF ARM64' },
  'darwin-x64': { format: 'Mach-O', expectedValue: 0x01000007, label: 'Mach-O x64' },
  'darwin-arm64': { format: 'Mach-O', expectedValue: 0x0100000c, label: 'Mach-O ARM64' },
} as const;

export type SupportedTarget = keyof typeof EXPECTED_ARCHITECTURES;

interface ZipEntry {
  fileName: string;
  compressionMethod: number;
  compressedSize: number;
  uncompressedSize: number;
  externalFileAttributes: number;
  localHeaderOffset: number;
}

export class VsixArchive {
  private buffer: Buffer;
  public entries: ZipEntry[] = [];

  constructor(filePath: string) {
    this.buffer = fs.readFileSync(filePath);
    this.parseCentralDirectory();
  }

  private parseCentralDirectory(): void {
    const buf = this.buffer;
    let eocdOffset = -1;

    for (let i = buf.length - 22; i >= Math.max(0, buf.length - 65557); i--) {
      if (buf.readUInt32LE(i) === 0x06054b50) {
        eocdOffset = i;
        break;
      }
    }

    if (eocdOffset === -1) {
      throw new Error('Not a valid ZIP/VSIX archive (End of Central Directory not found).');
    }

    const totalEntries = buf.readUInt16LE(eocdOffset + 10);
    const cdOffset = buf.readUInt32LE(eocdOffset + 16);

    let offset = cdOffset;
    for (let i = 0; i < totalEntries; i++) {
      if (buf.readUInt32LE(offset) !== 0x02014b50) {
        throw new Error(`Corrupted ZIP: Invalid central directory signature at offset ${offset}`);
      }

      const compressionMethod = buf.readUInt16LE(offset + 10);
      const compressedSize = buf.readUInt32LE(offset + 20);
      const uncompressedSize = buf.readUInt32LE(offset + 24);
      const fileNameLength = buf.readUInt16LE(offset + 28);
      const extraFieldLength = buf.readUInt16LE(offset + 30);
      const commentLength = buf.readUInt16LE(offset + 32);
      const externalFileAttributes = buf.readUInt32LE(offset + 38);
      const localHeaderOffset = buf.readUInt32LE(offset + 42);

      const fileName = buf.toString('utf8', offset + 46, offset + 46 + fileNameLength);

      this.entries.push({
        fileName,
        compressionMethod,
        compressedSize,
        uncompressedSize,
        externalFileAttributes,
        localHeaderOffset,
      });

      offset += 46 + fileNameLength + extraFieldLength + commentLength;
    }
  }

  public getEntry(name: string): ZipEntry | undefined {
    return this.entries.find((e) => e.fileName === name);
  }

  public readEntry(entry: ZipEntry): Buffer {
    const buf = this.buffer;
    const localOffset = entry.localHeaderOffset;

    if (buf.readUInt32LE(localOffset) !== 0x04034b50) {
      throw new Error(`Corrupted ZIP: Invalid local file header for ${entry.fileName}`);
    }

    const localFileNameLength = buf.readUInt16LE(localOffset + 26);
    const localExtraFieldLength = buf.readUInt16LE(localOffset + 28);
    const dataStart = localOffset + 30 + localFileNameLength + localExtraFieldLength;

    const compressedSlice = buf.subarray(dataStart, dataStart + entry.compressedSize);

    if (entry.compressionMethod === 0) {
      return Buffer.from(compressedSlice);
    } else if (entry.compressionMethod === 8) {
      const decompressed = zlib.inflateRawSync(compressedSlice);
      if (decompressed.length !== entry.uncompressedSize) {
        throw new Error(
          `Decompressed size mismatch for ${entry.fileName}: expected ${entry.uncompressedSize}, got ${decompressed.length}`
        );
      }
      return decompressed;
    } else {
      throw new Error(`Unsupported compression method ${entry.compressionMethod} in ${entry.fileName}`);
    }
  }
}

export function detectBinaryFormatAndArch(data: Buffer): {
  format: 'PE' | 'ELF' | 'Mach-O';
  archValue: number;
  archLabel: string;
} {
  // 1. Check ELF
  if (data.length >= 20 && data.toString('ascii', 0, 4) === '\x7fELF') {
    const archValue = data.readUInt16LE(18);
    let archLabel = `ELF 0x${archValue.toString(16)}`;
    if (archValue === 0x3e) archLabel = 'ELF x64 (EM_X86_64)';
    if (archValue === 0xb7) archLabel = 'ELF ARM64 (EM_AARCH64)';
    return { format: 'ELF', archValue, archLabel };
  }

  // 2. Check PE
  if (data.length >= 64 && data.toString('ascii', 0, 2) === 'MZ') {
    const peOffset = data.readUInt32LE(0x3c);
    if (data.length >= peOffset + 6 && data.toString('ascii', peOffset, peOffset + 4) === 'PE\0\0') {
      const archValue = data.readUInt16LE(peOffset + 4);
      let archLabel = `PE 0x${archValue.toString(16)}`;
      if (archValue === 0x8664) archLabel = 'PE x64 (AMD64)';
      if (archValue === 0xaa64) archLabel = 'PE ARM64 (AA64)';
      return { format: 'PE', archValue, archLabel };
    }
  }

  // 3. Check Mach-O
  if (data.length >= 8) {
    const magic = data.readUInt32BE(0);
    const isLittleEndian = magic === 0xcffaedfe || magic === 0xcefaedfe;
    const isBigEndian = magic === 0xfeedfacf || magic === 0xfeedface;

    if (isLittleEndian || isBigEndian) {
      const archValue = isLittleEndian ? data.readUInt32LE(4) : data.readUInt32BE(4);
      let archLabel = `Mach-O 0x${archValue.toString(16)}`;
      if (archValue === 0x01000007) archLabel = 'Mach-O x64 (CPU_TYPE_X86_64)';
      if (archValue === 0x0100000c) archLabel = 'Mach-O ARM64 (CPU_TYPE_ARM64)';
      return { format: 'Mach-O', archValue, archLabel };
    }
  }

  throw new Error('Unrecognized binary executable format (not PE, ELF, or Mach-O).');
}

export interface ValidationReport {
  vsixPath: string;
  targetPlatform: string;
  publisher: string;
  extensionId: string;
  version: string;
  bundledBinaryPath: string;
  binaryFormat: string;
  binaryArch: string;
  binaryArchMatched: boolean;
  unixPermissionsValid: boolean;
  platformIsolationPassed: boolean;
  runtimeFilesCheckPassed: boolean;
  errors: string[];
}

export function validateVsix(vsixPath: string, expectedTarget?: SupportedTarget): ValidationReport {
  if (!fs.existsSync(vsixPath)) {
    throw new Error(`VSIX file not found: ${vsixPath}`);
  }

  const archive = new VsixArchive(vsixPath);
  const errors: string[] = [];

  // 1. Required runtime files
  const requiredFiles = [
    '[Content_Types].xml',
    'extension.vsixmanifest',
    'extension/package.json',
    'extension/dist/extension.js',
    'extension/media/icon.png',
    'extension/LICENSE-MIT',
    'extension/LICENSE-APACHE',
  ];

  for (const rf of requiredFiles) {
    if (!archive.getEntry(rf)) {
      errors.push(`Missing required file in VSIX: '${rf}'`);
    }
  }

  // Check root dual-license descriptor (vsce normalizes LICENSE to extension/LICENSE.txt)
  if (!archive.getEntry('extension/LICENSE') && !archive.getEntry('extension/LICENSE.txt')) {
    errors.push("Missing required root dual-license file in VSIX ('extension/LICENSE' or 'extension/LICENSE.txt')");
  }

  // 2. Validate extension.vsixmanifest
  const manifestEntry = archive.getEntry('extension.vsixmanifest');
  let targetPlatform = '';
  let publisher = '';
  let extensionId = '';
  let version = '';

  if (manifestEntry) {
    const manifestXml = archive.readEntry(manifestEntry).toString('utf8');

    // Extract Identity attributes
    const identityMatch = manifestXml.match(
      /<Identity\s+([^>]*Id=["']([^"']+)["'][^>]*)>/i
    );

    if (identityMatch) {
      const attrs = identityMatch[0];
      const idMatch = attrs.match(/Id=["']([^"']+)["']/i);
      const pubMatch = attrs.match(/Publisher=["']([^"']+)["']/i);
      const verMatch = attrs.match(/Version=["']([^"']+)["']/i);
      const targetMatch = attrs.match(/TargetPlatform=["']([^"']+)["']/i);

      extensionId = idMatch ? idMatch[1] : '';
      publisher = pubMatch ? pubMatch[1] : '';
      version = verMatch ? verMatch[1] : '';
      targetPlatform = targetMatch ? targetMatch[1] : '';
    }

    if (publisher !== 'Taiizor') {
      errors.push(`Invalid publisher in manifest: '${publisher}' (expected 'Taiizor')`);
    }
    if (extensionId !== 'devcontext-memory') {
      errors.push(`Invalid extension ID in manifest: '${extensionId}' (expected 'devcontext-memory')`);
    }
    if (!targetPlatform) {
      errors.push('Manifest does not declare a TargetPlatform (universal packages not permitted in 6-platform architecture).');
    }
  } else {
    errors.push('Manifest file extension.vsixmanifest is missing.');
  }

  const effectiveTarget = (expectedTarget || targetPlatform) as SupportedTarget;
  if (!EXPECTED_ARCHITECTURES[effectiveTarget]) {
    errors.push(`Unsupported or unknown TargetPlatform: '${effectiveTarget}'`);
  }

  // 3. Inspect bundled binary in extension/bin/
  const binEntries = archive.entries.filter(
    (e) => e.fileName.startsWith('extension/bin/') && !e.fileName.endsWith('/')
  );

  if (binEntries.length === 0) {
    errors.push('No bundled CLI binary found in extension/bin/');
  } else if (binEntries.length > 1) {
    errors.push(
      `Platform isolation failure: extension/bin/ contains ${binEntries.length} files (${binEntries.map((e) => e.fileName).join(', ')}). Exactly one is required.`
    );
  }

  let bundledBinaryPath = '';
  let binaryFormat = 'Unknown';
  let binaryArch = 'Unknown';
  let binaryArchMatched = false;
  let unixPermissionsValid = true;

  if (binEntries.length > 0) {
    const binEntry = binEntries[0];
    bundledBinaryPath = binEntry.fileName;

    const isWindows = effectiveTarget && effectiveTarget.startsWith('win32-');
    const expectedBinaryName = isWindows ? 'extension/bin/devcontext.exe' : 'extension/bin/devcontext';

    if (binEntry.fileName !== expectedBinaryName) {
      errors.push(
        `Invalid binary filename for ${effectiveTarget}: '${binEntry.fileName}' (expected '${expectedBinaryName}')`
      );
    }

    // Unix permissions in ZIP header (top 16 bits of externalFileAttributes)
    if (!isWindows) {
      const unixMode = (binEntry.externalFileAttributes >>> 16) & 0o777;
      const isExecutable = (unixMode & 0o111) !== 0;
      if (!isExecutable) {
        unixPermissionsValid = false;
        errors.push(
          `Unix binary permission failure in ZIP header: mode is 0o${unixMode.toString(8)}, executable bits missing.`
        );
      }
    }

    // Read and parse binary data
    try {
      const binData = archive.readEntry(binEntry);
      if (binData.length === 0) {
        errors.push(`Bundled binary '${binEntry.fileName}' is empty (0 bytes).`);
      } else {
        const detected = detectBinaryFormatAndArch(binData);
        binaryFormat = detected.format;
        binaryArch = detected.archLabel;

        if (effectiveTarget && EXPECTED_ARCHITECTURES[effectiveTarget]) {
          const expected = EXPECTED_ARCHITECTURES[effectiveTarget];
          if (detected.format !== expected.format) {
            errors.push(
              `Binary format mismatch for ${effectiveTarget}: got '${detected.format}', expected '${expected.format}'`
            );
          }
          if (detected.archValue !== expected.expectedValue) {
            errors.push(
              `Architecture signature mismatch for ${effectiveTarget}: got 0x${detected.archValue.toString(16)} (${detected.archLabel}), expected 0x${expected.expectedValue.toString(16)} (${expected.label})`
            );
          } else {
            binaryArchMatched = true;
          }
        }
      }
    } catch (parseErr) {
      errors.push(`Failed to parse binary format of ${binEntry.fileName}: ${(parseErr as Error).message}`);
    }
  }

  // 4. Ensure no foreign binaries in other locations
  const foreignBinaries = archive.entries.filter(
    (e) =>
      !e.fileName.startsWith('extension/bin/') &&
      (e.fileName.endsWith('.exe') ||
        e.fileName.endsWith('.node') ||
        e.fileName.endsWith('.dll') ||
        e.fileName.endsWith('.so') ||
        e.fileName.endsWith('.dylib'))
  );

  if (foreignBinaries.length > 0) {
    errors.push(
      `Foreign binaries detected outside extension/bin/: ${foreignBinaries.map((e) => e.fileName).join(', ')}`
    );
  }

  const report: ValidationReport = {
    vsixPath,
    targetPlatform,
    publisher,
    extensionId,
    version,
    bundledBinaryPath,
    binaryFormat,
    binaryArch,
    binaryArchMatched,
    unixPermissionsValid,
    platformIsolationPassed: binEntries.length === 1 && foreignBinaries.length === 0,
    runtimeFilesCheckPassed: errors.filter((e) => e.includes('Missing required file')).length === 0,
    errors,
  };

  return report;
}

export function runCli(): void {
  const args = process.argv.slice(2);
  let target: SupportedTarget | undefined;
  let targetPath: string | undefined;
  let dirPath: string | undefined;

  for (let i = 0; i < args.length; i++) {
    const arg = args[i];
    if (arg === '--target' && i + 1 < args.length) {
      target = args[++i] as SupportedTarget;
    } else if (arg === '--dir' && i + 1 < args.length) {
      dirPath = path.resolve(args[++i]);
    } else if (!arg.startsWith('--')) {
      targetPath = path.resolve(arg);
    }
  }

  if (targetPath && fs.existsSync(targetPath) && fs.statSync(targetPath).isDirectory()) {
    dirPath = targetPath;
    targetPath = undefined;
  }

  if (dirPath || !targetPath) {
    // If no path given or --dir specified, check target directory
    const searchDir = dirPath || path.resolve(process.cwd(), 'dist-vsix');
    if (fs.existsSync(searchDir)) {
      const vsixFiles = fs.readdirSync(searchDir).filter((f) => f.endsWith('.vsix'));
      if (vsixFiles.length === 0) {
        console.error(`No .vsix files found in ${searchDir}. Provide a path to a .vsix file.`);
        process.exit(1);
      }

      let allPassed = true;
      console.log(`[ValidateVsix] Validating ${vsixFiles.length} VSIX package(s) in ${searchDir}...\n`);

      for (const f of vsixFiles) {
        const fullPath = path.join(searchDir, f);
        const report = validateVsix(fullPath);
        printReport(report);
        if (report.errors.length > 0) allPassed = false;
      }

      if (!allPassed) {
        console.error('\n[ValidateVsix] VALIDATION FAILED on one or more packages.');
        process.exit(1);
      }
      console.log('\n[ValidateVsix] ALL PACKAGES VALIDATED SUCCESSFULLY.');
      return;
    }

    console.error(`Directory not found: ${searchDir}`);
    process.exit(1);
  }

  const report = validateVsix(targetPath, target);
  printReport(report);

  if (report.errors.length > 0) {
    console.error('\n[ValidateVsix] VALIDATION FAILED.');
    process.exit(1);
  }

  console.log('\n[ValidateVsix] VALIDATION PASSED.');
}

function printReport(report: ValidationReport): void {
  console.log('='.repeat(70));
  console.log(`Package: ${path.basename(report.vsixPath)}`);
  console.log(`TargetPlatform: ${report.targetPlatform}`);
  console.log(`Identity: ${report.publisher}.${report.extensionId} v${report.version}`);
  console.log(`Bundled Binary: ${report.bundledBinaryPath}`);
  console.log(`Detected Architecture: ${report.binaryFormat} (${report.binaryArch})`);
  console.log(`Architecture Matched: ${report.binaryArchMatched ? 'YES' : 'NO'}`);
  console.log(`Unix Permissions: ${report.unixPermissionsValid ? 'VALID' : 'INVALID'}`);
  console.log(`Platform Isolation: ${report.platformIsolationPassed ? 'PASSED' : 'FAILED'}`);
  console.log(`Runtime Files: ${report.runtimeFilesCheckPassed ? 'PASSED' : 'FAILED'}`);

  if (report.errors.length > 0) {
    console.log('\nErrors:');
    for (const err of report.errors) {
      console.log(`  - [ERROR] ${err}`);
    }
  }
  console.log('='.repeat(70));
}

if (import.meta.main) {
  runCli();
}
