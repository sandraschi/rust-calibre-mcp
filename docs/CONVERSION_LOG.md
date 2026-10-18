# Conversion Audit Log: `rust-calibre-mcp`

## [2026-10-18T00:55:49+02:00] Phase 0 & Phase 1 (Automated `New-RustArtcraftRepo.ps1` Stamp)
- Verified legacy repo `D:\Dev\repos\calibre-mcp` at `master@5abaad6` (kept 100% read-only).
- Copied reusable kernel modules from `D:\Dev\repos\rust-devices-mcp`:
  - `crates/calibre-core/src/{bridge.rs, llm.rs (gemma4:12b default), error.rs, creds.rs}`
  - `crates/calibre-app/src/{vault.rs, registry.rs}`
- Generated and completed `docs/CONVERSION_DOSSIER.md` with all 29 legacy `@mcp.tool` definitions, 11 live Calibre libraries on `L:\Multimedia Files\Written Word\`, and the custom FTS5 tokenizer bypass strategy.
- Committed Phase 0 & Phase 1 baseline (`e444627`).

## [2026-10-18T01:40:00+02:00] Phase 2 (`crates/calibre-core` L1 Pure Library)
- Implemented `crates/calibre-core/src/import.rs` (auto-discovers libraries from explicit arg, `CALIBRE_LIBRARY_PATH`, `../calibre-mcp/config.json`, and `L:\Multimedia Files\Written Word\*`).
- Implemented `crates/calibre-core/src/calibre_db.rs`:
  - Strict read-only SQLite helper (`OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX` + `busy_timeout(5000 ms)`).
  - Full-text search on `full-text-search.db:books_text` bypassing Calibre's custom `calibre remove_diacritics` FTS5 C tokenizer.
  - 11 domain operations (`libraries_list`, `library_stats`, `query_books`, `book_get`, `search_fulltext`, `bookcase_catalog`, `authors_list`, `tags_list`, `series_analysis`, `find_duplicates`, `library_health`) + deterministic 6-book `Demo` library.
- Implemented `crates/calibre-core/src/lib.rs` (`Command` enum, `dispatch()`, `dispatch_future_is_send` test).

## [2026-10-18T01:45:00+02:00] Phase 3 (`crates/calibre-mcp` L2 CLI + Dual-Aliased MCP Server)
- Implemented `crates/calibre-mcp/src/main.rs`:
  - Forensic panic hook writing to `%LOCALAPPDATA%\rust-calibre-mcp\crash.log` and `stderr` (never `stdout`).
  - `clap` subcommands (`status`, `libraries`, `library-stats`, `query-books`, `book-get`, `search-fulltext` / `fts`, `bookcase`, `authors`, `tags`, `series`, `duplicates`, `health`, `speak`, `chat`, `models`, `mcp` / `mcp-stdio`).
  - Dual-aliased `MCP 2024-11-05` tool router supporting 17 canonical `calibre.*` tools and all 29 legacy Python portmanteau tool names.

## [2026-10-18T01:49:00+02:00] Phase 4 (`crates/calibre-app` L3 Native `egui` 0.37 Desktop App)
- Implemented `crates/calibre-app/src/{main.rs, app.rs, vault.rs, registry.rs}`:
  - 9 pages: `Libraries`, `Books`, `Fulltext`, `Analytics`, `Chat` (`gemma4:12b`), `Crossconnect`, `Settings`, `Help`, `RawJson`.
  - Off-thread async worker queue (`self.spawn(...)`), `Ctrl+Wheel` zoom (`0.5`..`2.0`), custom painted Sun/Moon theme toggle, and `--smoke` headless mode.

## [2026-10-18T02:05:00+02:00] Phase 5 (Verification Gates & Live Multi-Library Oracle Probes)
- `cargo fmt --all --check`: **PASS**
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`: **PASS** (0 warnings)
- `cargo test --workspace`: **PASS** (27/27 tests passing across all 3 crates)
- `cargo run -q -p calibre-app -- --smoke`: **PASS** (`smoke: status + demo query_books (6) + demo library_health OK`)
- Live SQLite Oracle Probes against `L:\Multimedia Files\Written Word\`:
  - Discovered 11 live libraries + `Demo`.
  - Verified `Calibre-Bibliothek` (`10,339` books, `6,309` authors, `3,142` tags, `378` series, `100.0%` covers, `90.2%` comments).
  - Verified `Calibre-Bibliothek IT` `query-books` and `fts` (`full-text-search.db:books_text`).
- MCP JSON-RPC 2.0 `mcp-stdio` round-trip probe: **PASS** (5/5 responses including legacy aliases `manage_libraries` and `search_fulltext`).
- Confirmed legacy `D:\Dev\repos\calibre-mcp` has 0 modified/untracked files (`git status -s` empty).