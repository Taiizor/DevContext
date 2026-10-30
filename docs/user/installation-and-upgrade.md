# Installation and Upgrade Guide

DevContext consists of a standalone native CLI binary (`devcontext`) and a VS Code editor extension.

---

## 1. Prerequisites

- **Supported Operating Systems:**
  - Windows 10 / 11 (x86_64, ARM64)
  - Linux (Ubuntu 20.04+, Debian 11+, Fedora 38+, Arch Linux, x86_64 and aarch64)
  - macOS 12 Monterey or later (Apple Silicon M-series and Intel x86_64)
- **Editor:** VS Code 1.85.0 or later (or compatible editor: VSCodium, Cursor, Windsurf).
- **Git (Optional):** Git 2.30+ is recommended for timeline integration. DevContext functions fully in non-Git folders.

---

## 2. CLI Installation

### Option A: Pre-built Binary
Download the pre-compiled standalone binary for your architecture from the release page:
- `devcontext-windows-x64.zip` -> `devcontext.exe`
- `devcontext-linux-x64.tar.gz` -> `devcontext`
- `devcontext-macos-arm64.tar.gz` -> `devcontext`

Place the executable in a directory included in your system `PATH` (e.g. `C:\tools\bin` or `/usr/local/bin`), and verify:
```bash
devcontext --version
# Output: devcontext 0.1.0
```

### Option B: Build from Source using Cargo
If you have the Rust toolchain installed:
```bash
git clone https://github.com/Taiizor/DevContext.git
cd DevContext
cargo install --path apps/cli
```

---

## 3. VS Code Extension Installation

The canonical Visual Studio Code extension identifier is **`Taiizor.devcontext-memory`** (listed on the Visual Studio Marketplace as **DevContext by Taiizor**).

### Option A: Install from VSIX Package
Download `devcontext-memory-0.1.0.vsix` and install via terminal:
```bash
code --install-extension devcontext-memory-0.1.0.vsix
```
Or in VS Code:
1. Open the **Extensions** view (`Ctrl+Shift+X` / `Cmd+Shift+X`).
2. Click the `...` menu (Views and More Actions) in the top right.
3. Select **Install from VSIX...** and choose the downloaded file.

### Option B: Visual Studio Marketplace (Planned)
- Planned Marketplace Destination: `https://marketplace.visualstudio.com/items?itemName=Taiizor.devcontext-memory` `[PENDING PUBLICATION]`
*(Note: Marketplace listing is pending public release and cannot be installed from Marketplace yet).*

### Option C: Automatic Core Resolution
Once installed, the extension locates the native binary in order:
1. `devcontext.core.executablePath` setting (if set).
2. Bundled platform binary included in the platform VSIX package.
3. System `PATH`.

---

## 4. Upgrading & Migrating DevContext

### Migrating to `Taiizor.devcontext-memory`
The official extension identifier is `Taiizor.devcontext-memory`. Note that `Taiizor.devcontext-memory` is a distinct VS Code extension identity from any previous development package or third-party identifier (`devcontext.devcontext` or `Taiizor.devcontext`). 

VS Code does not automatically migrate extensions across different publisher or extension IDs. To migrate manually:
1. Uninstall any older package:
   ```bash
   code --uninstall-extension devcontext.devcontext
   # or if an earlier preview was installed:
   code --uninstall-extension Taiizor.devcontext
   ```
2. Install the official `Taiizor.devcontext-memory` VSIX:
   ```bash
   code --install-extension devcontext-memory-0.1.0.vsix
   ```
3. Verify that existing DevContext project data remains intact: all project memory is stored directly in your workspace under `.devcontext/` and is completely independent of the extension publisher identity.

### Core Database Migrations
DevContext uses forward-only, transactional SQLite database migrations:
- Upgrades are automatic: When an upgraded CLI or extension opens an existing `.devcontext/devcontext.db`, migrations execute transactionally inside an atomic SQLite transaction.
- If a migration step encounters an error, the transaction rolls back cleanly without modifying previous data.

### Upgrading Export Interchange Bundles
- DevContext supports schema version upgrade paths (e.g. upgrading v0.9.0 bundles to v1.0.0 during import).
- All previous v1 export bundles remain fully compatible.

---

## 5. Clean Uninstall

DevContext stores all project memory in a dedicated `.devcontext/` directory in each project root:
- To uninstall from a project: Delete the `.devcontext/` directory. No global state, registry entries, or hidden user-profile caches remain.
- To uninstall the extension: Run `code --uninstall-extension Taiizor.devcontext-memory`.
