# Conversion Audit Log: `rust-calibre-mcp`

## [2026-10-18T00:55:49+02:00] Phase 0 & Phase 1 (Automated `New-RustArtcraftRepo.ps1` Stamp)
- Verified legacy repo `D:\Dev\repos\calibre-mcp` at `master@5abaad6` (kept 100% read-only).
- Copied reusable kernel modules from `D:\Dev\repos\rust-devices-mcp`:
  - `crates/calibre-core/src/{bridge.rs, llm.rs (gemma4:12b default), error.rs, creds.rs}`
  - `crates/calibre-app/src/{vault.rs, registry.rs}`
- Generated and completed `docs/CONVERSION_DOSSIER.md` with all 29 legacy `@mcp.tool` definitions, 11 live Calibre libraries on `L:\Multimedia Files\Written Word\`, and the custom FTS5 tokenizer bypass strategy.
- Committed Phase 0 & Phase 1 baseline (`e444627`).

## [2026-10-18T01:40:00+02:00] Phase 2 (`crates/calibre-core` L1 Pure Library)
- Implemented `crates/calibre-core/src/import.rs` (auto-discovers libraries from explicit arg, `CALIBRE_LIBRARY_PATH`, `%APPDATA%\calibre\gui.json`, `global.py.json`, and `L:\Multimedia Files\Written Word\*`).
- Implemented `crates/calibre-core/src/calibre_db.rs`:
  - Strict read-only SQLite helper (`OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX` + `busy_timeout(5000 ms)`).
  - Full-text search on `full-text-search.db:books_text` bypassing Calibre's custom `calibre remove_diacritics` FTS5 C tokenizer.
  - 17 domain operations (`libraries_list`, `library_stats`, `query_books`, `book_get`, `book_cover_bytes`, `book_read_excerpt`, `book_update_metadata`, `search_all_libraries`, `search_fulltext`, `authors_list`, `tags_list`, `series_analysis`, `publishers_list`, `library_health`, `find_duplicates`, `reading_statistics`, `unread_priority_list`, `export_json`) + deterministic 6-book `Demo-Library` fallback.
- Implemented `crates/calibre-core/src/lib.rs` (`Command` enum across 23 canonical commands, `dispatch()`, `dispatch_future_is_send` test).

## [2026-10-18T01:45:00+02:00] Phase 3 (`crates/calibre-mcp` L2 CLI + Dual-Aliased MCP Server)
- Implemented `crates/calibre-mcp/src/main.rs`:
  - Forensic panic hook writing to `%LOCALAPPDATA%\rust-calibre-mcp\crash.log` and `stderr` (never `stdout`).
  - `clap 4.6` subcommands for all 23 canonical commands (`status`, `libraries`, `library-stats`, `query-books`, `book-get`, `book-cover`, `book-excerpt`, `book-update`, `search-all-libraries`, `search-fulltext`, `authors`, `tags`, `series`, `publishers`, `library-health`, `duplicates`, `reading-stats`, `unread-priority`, `export-json`, `import-config`, `speak`, `event`, `chat`, `mcp-stdio`).
  - Dual-aliased `MCP 2024-11-05` tool router supporting 23 canonical `calibre.*` tools and legacy Python portmanteau tool names.

## [2026-10-18T01:49:00+02:00] Phase 4 (`crates/calibre-app` L3 Native `egui` 0.37 Desktop App)
- Implemented `crates/calibre-app/src/{main.rs, app.rs, vault.rs, registry.rs}`:
  - 9 pages: `Libraries`, `Books` (with live `cover.jpg` texture rendering, paginated `full-text-search.db` excerpt reader, and federated 11-library search), `Fulltext`, `Analytics` (Health KPIs, Publication Decade bar chart, Series Gap Detector, and Unread Priority Queue), `Chat` (`gemma4:12b`), `Crossconnect`, `Settings`, `Help`, `RawJson`.
  - Off-thread async worker queue (`self.spawn(...)`), `Ctrl+Wheel` zoom (`0.5`..`2.0`), custom painted Sun/Moon theme toggle, window icon (`icons/icon.png`), and `--smoke` headless mode.

## [2026-10-18T03:23:00+02:00] Phase 5 & Deep Parity Upgrade (23 Commands, Packaging, & Live Oracle Probes)
- Expanded from 17 to **23 canonical commands** (`calibre.book_cover`, `calibre.book_excerpt`, `calibre.book_update`, `calibre.search_all_libraries`, `calibre.reading_stats`, `calibre.unread_priority`).
- Added full `rust-devices-mcp` packaging & documentation parity (`installer/installer.nsi`, `INSTALL.md`, `CHANGELOG.md`, `docs/UI_PARITY.md`, `crates/calibre-core/README.md`, `crates/calibre-mcp/README.md`, `crates/calibre-app/README.md`, `crates/calibre-app/icons/icon.png`).
- `cargo fmt --all --check`: **PASS**
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`: **PASS** (0 warnings)
- `cargo test --workspace`: **PASS** (28/28 tests passing across all 3 crates)
- `cargo run -q -p calibre-app -- --smoke`: **PASS** (`smoke: status + demo query_books (6) + demo library_health OK`)
- Live SQLite Oracle Probes against `L:\Multimedia Files\Written Word\`:
  - `book-cover --library "Calibre-Bibliothek" --book-id 1274`: **PASS** (loaded cover bytes for *Men at Arms*)
  - `book-excerpt --library "Calibre-Bibliothek IT" --book-id 4 --max-chars 350`: **PASS** (`full-text-search.db:books_text`)
  - `reading-stats --library "Calibre-Bibliothek"`: **PASS** (languages, top 20 publishers, decades, ratings)
  - `unread-priority --library "Calibre-Bibliothek IT" --limit 3`: **PASS** (ranked priority queue with scores & reasons)
  - `search-all --query "Python" --limit-per-library 2`: **PASS** (federated search across all discovered libraries)
  - `book-update --library "Calibre-Bibliothek" --book-id 1274 --title "Dry Run Preview Title"`: **PASS** (`dry_run: true, committed: false`)
- Confirmed legacy `D:\Dev\repos\calibre-mcp` has 0 modified/untracked files (`git status -s` empty).