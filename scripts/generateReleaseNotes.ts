import * as fs from 'node:fs';
import * as path from 'node:path';

function parseArgs(): { tag: string; outputFile?: string } {
  const args = process.argv.slice(2);
  let tag: string | undefined;
  let outputFile: string | undefined;

  for (let i = 0; i < args.length; i++) {
    const arg = args[i];
    if ((arg === '--tag' || arg === '-t') && i + 1 < args.length) {
      tag = args[++i];
    } else if ((arg === '--output' || arg === '-o') && i + 1 < args.length) {
      outputFile = path.resolve(args[++i]);
    } else if (!arg.startsWith('--') && !tag) {
      tag = arg;
    }
  }

  if (!tag) {
    const pkgPath = path.resolve(__dirname, '../apps/vscode-extension/package.json');
    if (fs.existsSync(pkgPath)) {
      const pkg = JSON.parse(fs.readFileSync(pkgPath, 'utf8'));
      tag = `v${pkg.version}`;
    } else {
      tag = 'v0.1.0';
    }
  }

  return { tag, outputFile };
}

export function extractChangelogSection(changelogPath: string, version: string): string | undefined {
  if (!fs.existsSync(changelogPath)) {
    return undefined;
  }

  const content = fs.readFileSync(changelogPath, 'utf8');
  // Match ## [version] or ## [v.version] up to the next ## [ or end of file
  const escapedVersion = version.replace(/\./g, '\\.');
  const regex = new RegExp(`##\\s*\\[v?${escapedVersion}\\][^\\n]*\\n([\\s\\S]*?)(?=\\n##\\s*\\[|$)`);
  const match = content.match(regex);

  if (match && match[1]) {
    return match[1].trim();
  }

  return undefined;
}

export function generateReleaseNotes(tag: string, repoRoot: string = path.resolve(__dirname, '..')): string {
  const version = tag.startsWith('v') ? tag.slice(1) : tag;
  const changelogPath = path.join(repoRoot, 'CHANGELOG.md');
  const changelogSection = extractChangelogSection(changelogPath, version);

  const defaultHighlights = [
    '- **Six-Platform VSIX Packaging:** Native pre-compiled CLI bundled for Windows, Linux, and macOS.',
    '- **Layered Fallback Resolution:** Respects explicit settings, bundled engine, workspace builds, and PATH.',
    '- **Local-First & Offline:** 100% private with SQLite WAL storage, BM25 full-text search, and zero telemetry.',
  ].join('\n');

  const highlights = changelogSection || defaultHighlights;

  return [
    `## DevContext ${tag}`,
    '',
    '**DevContext** is a reliable, local-first project memory system for software development. It preserves developer-authored notes, architecture decisions (ADRs), milestones, references, and saved commands with fast SQLite FTS5 search and strict privacy guarantees.',
    '',
    '---',
    '',
    '### 📦 Installation & Distribution',
    '',
    '#### VS Code Extension (Zero-Configuration Out of the Box)',
    'Install **DevContext by Taiizor** (`Taiizor.devcontext-memory`) directly from the [Visual Studio Marketplace](https://marketplace.visualstudio.com/items?itemName=Taiizor.devcontext-memory) or via command line:',
    '',
    '```bash',
    'code --install-extension Taiizor.devcontext-memory',
    '```',
    '*Zero-configuration:* The extension package automatically includes the pre-compiled native `devcontext` engine for your platform. **No separate CLI, Rust, or Cargo installation is required.**',
    '',
    '#### Platform-Specific VSIX Packages (Manual / Offline Installation)',
    'If installing offline without Marketplace access, download the matching VSIX package below:',
    `- **Windows x64:** \`devcontext-memory-${version}-win32-x64.vsix\``,
    `- **Windows ARM64:** \`devcontext-memory-${version}-win32-arm64.vsix\``,
    `- **Linux x64 (glibc 2.35+):** \`devcontext-memory-${version}-linux-x64.vsix\``,
    `- **Linux ARM64 (glibc 2.35+):** \`devcontext-memory-${version}-linux-arm64.vsix\``,
    `- **macOS Apple Silicon (11.0+):** \`devcontext-memory-${version}-darwin-arm64.vsix\``,
    `- **macOS Intel (11.0+):** \`devcontext-memory-${version}-darwin-x64.vsix\``,
    '',
    'Install manually via:',
    '```bash',
    `code --install-extension devcontext-memory-${version}-<platform>.vsix`,
    '```',
    '',
    '#### Standalone Native CLI (Terminal, Scripts & CI)',
    'Standalone CLI binaries for headless or automation environments:',
    `- **Windows:** \`devcontext-${tag}-windows-x64.zip\` | \`devcontext-${tag}-windows-arm64.zip\``,
    `- **Linux:** \`devcontext-${tag}-linux-x64.tar.gz\` | \`devcontext-${tag}-linux-arm64.tar.gz\``,
    `- **macOS:** \`devcontext-${tag}-macos-arm64.tar.gz\` | \`devcontext-${tag}-macos-x64.tar.gz\``,
    '',
    'Via Cargo:',
    '```bash',
    'cargo install devcontext-cli',
    '```',
    '',
    '---',
    '',
    '### 🛡️ Verification & Checksums',
    'All release artifacts can be verified using the attached `SHA256SUMS.txt`:',
    '```bash',
    'sha256sum -c SHA256SUMS.txt',
    '```',
    '',
    '---',
    '',
    `### 🚀 Release Highlights & Changelog (${tag})`,
    '',
    highlights,
    '',
    '---',
    '',
    `**Full Changelog**: https://github.com/Taiizor/DevContext/releases/tag/${tag}`,
    '',
  ].join('\n');
}

if (import.meta.main) {
  const { tag, outputFile } = parseArgs();
  const notes = generateReleaseNotes(tag);

  if (outputFile) {
    fs.writeFileSync(outputFile, notes, 'utf8');
    console.log(`[GenerateReleaseNotes] Generated release notes for ${tag} at ${outputFile}`);
  } else {
    process.stdout.write(notes);
  }
}
