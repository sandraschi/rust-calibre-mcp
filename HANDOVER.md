# HANDOVER -- `rust-calibre-mcp`

**Last Updated:** 2026-10-18T01:06:00+02:00  
**Current Phase:** Phase 0 Complete (`docs/CONVERSION_DOSSIER.md` committed) -> Ready for Phase 1/2 (`crates/calibre-core`)

## What Was Done
- Ran `mcp-central-docs/scripts/New-RustArtcraftRepo.ps1 -Domain calibre`.
- Completed the 7-point Belt-and-Suspenders `docs/CONVERSION_DOSSIER.md`:
  - Confirmed `D:\Dev\repos\calibre-mcp` is at `master@5abaad6` (100% untouched).
  - Verified 11 live Calibre libraries on `L:\Multimedia Files\Written Word\` (`Calibre-Bibliothek` 34.36 MB `metadata.db` + `full-text-search.db`).
  - Mapped all 29 `calibre-mcp` tools (including portmanteau `manage_libraries`, `query_books`, `manage_books`, `search_fulltext`, `bookcase_catalog`, `manage_authors`, `manage_tags`, `get_series_analysis`, `find_duplicate_books`, `analyze_library_health`, etc.) to `Command` variants with dual tool-name aliasing.
  - Copied `bridge.rs`, `llm.rs` (`gemma4:12b`), `error.rs`, `creds.rs`, `vault.rs`, and `registry.rs` from `rust-devices-mcp`.

## Next Concrete Action
1. Write workspace `Cargo.toml`, `crates/calibre-core` (`import.rs` reading `../calibre-mcp/config.json` + `L:\Multimedia Files\Written Word\*`, and `calibre_db.rs` querying `metadata.db` + `full-text-search.db` via `rusqlite` `SQLITE_OPEN_READ_ONLY` + `Demo` fallback).
2. Wire `crates/calibre-mcp` CLI + dual-aliased MCP stdio server and `crates/calibre-app` `egui` desktop UI (`Libraries`, `Books`, `Analytics` tabs).