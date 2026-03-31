# Scoria Release Runbook

## 1) Prepare release

1. Ensure `main` is green in CI (`.github/workflows/ci.yml`).
2. Confirm local checks:
   - `cargo fmt --all --check`
   - `cargo clippy --all-targets -- -D warnings`
   - `cargo test --locked`
3. Bump version in `Cargo.toml` if needed and update changelog context.

## 2) Tag and push

1. Create tag in `vX.Y.Z` format.
2. Push tag to origin:

```bash
git tag vX.Y.Z
git push origin vX.Y.Z
```

This triggers `.github/workflows/release.yml`.

## 3) Validate release artifacts

After workflow completion, confirm the GitHub Release contains:

- Unix archives:
  - `scoria-linux-x86_64.tar.gz`
  - `scoria-linux-aarch64.tar.gz`
  - `scoria-macos-x86_64.tar.gz`
  - `scoria-macos-aarch64.tar.gz`
- Windows artifacts:
  - `scoria-windows-x86_64.zip`
  - `scoria-windows-x86_64.msi`
- Checksums:
  - `*.tar.gz.sha256`
  - `*.sha256` for Windows zip/msi

## 4) Post-release verification

1. Install from release assets on each platform (at least smoke):
   - Linux/macOS: `install.sh`
   - Windows: MSI and portable ZIP (`install.ps1`)
2. Verify runtime basics:
   - `scoria --version`
   - tray starts
   - manual update check behavior by platform
3. Verify uninstall paths:
   - Unix: `uninstall.sh`
   - Windows: `uninstall.ps1` (portable), Apps & Features (MSI)
4. Run platform smoke checklist from [docs/smoke-tests.md](docs/smoke-tests.md).

## 5) Optional winget maintenance

Generate manifests using:

```powershell
powershell -File scripts/windows/gen-winget-manifests.ps1 -Version X.Y.Z -Sha256 "<sha256>"
```

Then validate/publish via your normal winget process.
