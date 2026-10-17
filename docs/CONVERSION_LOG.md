# Conversion Audit Log: `rust-calibre-mcp`

## [2026-10-18T01:03:54+02:00] Phase 0 & Phase 1 Init (Automated `New-RustArtcraftRepo.ps1` Stamp)
- Verified legacy repo `D:\Dev\repos\calibre-mcp` at `master@5abaad6` (kept 100% read-only).
- Copied reusable kernel modules from `D:\Dev\repos\rust-devices-mcp`:
  - `crates/calibre-core/src/{bridge.rs, llm.rs (gemma4:12b default), error.rs, creds.rs}`
  - `crates/calibre-app/src/{vault.rs, registry.rs}`
- Probed live Calibre libraries on `L:\Multimedia Files\Written Word\`: confirmed 11 active libraries (`Calibre-Bibliothek` 34.36 MB + `full-text-search.db`, `Calibre-Bibliothek IT` 4.10 MB, `Calibre-Bibliothek Deutsch` 2.08 MB, etc.).
- Verified ports `10720` / `10721` are free; designed `SQLITE_OPEN_READ_ONLY | SQLITE_OPEN_NO_MUTEX` + `busy_timeout(5000 ms)` contention-free access.
- Completed 7-point `docs/CONVERSION_DOSSIER.md` mapping all 29 active `calibre-mcp` tools + legacy portmanteau aliases to `Command` variants.