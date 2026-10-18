# Developer & Build Guide — `rust-calibre-mcp`

This guide covers building from source, running tests, checking lints, and packaging.

## Prerequisites

- **Rust 1.85+** (`rustup default stable`)
- **NSIS 3.0+** (optional, for building the Windows installer)

## Common Recipes (`justfile`)

```powershell
# Format code and run clippy checks
just check

# Run all workspace unit tests and UI snapshot checks
just test

# Headless smoke test of the egui desktop app
cargo run -p calibre-app -- --smoke

# Build release binaries
cargo build --workspace --release
```

## Packaging the Windows Installer

```powershell
# Build release binaries first
cargo build --workspace --release

# Run makensis from repo root
& "C:\Program Files (x86)\NSIS\makensis.exe" installer\installer.nsi

# The installer is generated at target\release\bundle\Calibre-Rust-0.4.0-setup.exe
```

## Quality Standards

- Zero panics, zero `.unwrap()`, zero `.expect()` enforced across the workspace via `#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]`.
- Zero compiler or clippy warnings under `cargo clippy --workspace --all-targets --all-features -- -D warnings`.
- Strict glyph allowlist enforced by `parity_tests::glyph_source_scan` in `calibre-app`.
