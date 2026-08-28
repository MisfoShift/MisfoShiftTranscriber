# Public release procedure

This document covers repeatable checks for a public Windows release. It does not authorize publishing, signing, or uploading artifacts.

## Dependency vulnerability audit

Run the following from PowerShell with network access. Review every finding instead of applying `--force` updates automatically.

```powershell
npm install
npm audit

cargo install cargo-audit --locked
cd src-tauri
cargo audit
cd ..
```

`npm audit` checks the exact dependency tree in `package-lock.json`. `cargo audit` checks `src-tauri/Cargo.lock` against the RustSec advisory database. Record the execution date, tool versions, unresolved advisories, and the reason for any accepted risk in the release notes.

## Build and test

```powershell
npm test
npm run build
cd src-tauri
cargo fmt --check
cargo test
cargo clippy --all-targets -- -D warnings
cd ..
npm run tauri:build:windows
```

The default bundle must contain the app, `LICENSE`, and `THIRD_PARTY_NOTICES.md`. It must not contain downloaded whisper.cpp binaries, Whisper model files, or FFmpeg binaries. `bundle.resources` makes both notice files readable after installation. The en-US WiX database uses code page 1252, so its license screen uses `src-tauri/windows/LICENSE.txt`, an English-only copy of the legally controlling MIT text from the repository `LICENSE`; the installed `LICENSE` retains the Japanese reference translation as well.

The Windows installer Manufacturer/Publisher metadata must be `MisfoShift`, and the application identifier must be `com.misfoshift.transcriber`. Treat changes to either value as release-significant because they can affect installer identity and the application-data directory.

## Content Security Policy

The current UI loads styles from the packaged `src/styles.css` and does not use inline `<style>`, `style=` attributes, or DOM `.style` assignments. Therefore `style-src` is restricted to `'self'` without `'unsafe-inline'`. Recheck the CSP whenever inline styling or a UI framework that injects styles is introduced.

## Release checksums

After a successful Windows bundle build, generate hashes for the application executable, MSI, and NSIS installer:

```powershell
npm run release:checksums
```

The script writes `src-tauri/target/release/SHA256SUMS.txt` by default. Upload that file alongside the matching artifacts in GitHub Releases. To hash another staging directory:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/generate-release-checksums.ps1 -ReleaseDirectory C:\path\to\release
```

Verify a published file independently with:

```powershell
Get-FileHash -Algorithm SHA256 -LiteralPath .\downloaded-installer.exe
```

Complete [RELEASE_CHECKLIST.md](RELEASE_CHECKLIST.md) on clean Windows 10 and Windows 11 environments before publishing. Windows builds are currently unsigned; see [WINDOWS_CODE_SIGNING.md](WINDOWS_CODE_SIGNING.md) before enabling signing.
