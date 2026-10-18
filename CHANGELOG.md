# Changelog

All notable changes to `rust-calibre-mcp` are documented in this file.

## [0.4.0] - 2026-10-18

### Added
- **3-Crate ArtCraft Architecture (`calibre-core` L1, `rust-calibre-mcp` L2, `calibre-app` L3)** converted from `calibre-mcp` (`master@5abaad6`) under the `repconv` v2.1 standard.
- **Full Structural Webapp Clone Parity (All 16 Pages in 5 Grouped Categories)**:
  - Replaced the initial monolithic dashboard with complete 1:1 page parity mirroring the legacy Next.js webapp:
    - **`CONTENT`**: `Libraries` (multi-collection explorer, FTS5 readiness, sizes), `Books & Inspector` (live cover image texture, metadata viewer, paginated full-text excerpt reader), `Authors` (A-Z initial letter filter bar, search box, book counts, 1-click book filter), `Series` (canon coverage, gap detector with missing volume indices), `Tags` (full taxonomy badge browser), `Publishers` (top publishing houses by volume).
    - **`DISCOVER`**: `Full-Text Search` (direct `full-text-search.db` text scan), `Reading & Priorities` (publication decade bar chart, read/unread ratio KPIs, ranked unread priority queue), `Duplicates` (normalized title duplicate group scanner with multi-format inspection).
    - **`MANAGE`**: `Library Health` (KPI cards: Total Books, Cover Coverage %, Synopsis Coverage %, Duplicate Title Groups, plus missing covers/comments/tags breakdowns).
    - **`AI & FLEET`**: `AI Librarian` (conversational assistant with active library context), `Crossconnect` (`speech-mcp` voice synthesis + `aiwatcher` telemetry webhooks).
    - **`SYSTEM`**: `System Logs` (real-time timestamped event log with text filtering and reset), `Settings & Vault` (library path pinning, Calibre server vault, and Fleet Standard LLM Card), `Help` (shortcuts, zoom, SQLite safety guarantees), `Raw JSON` (live L1 command payload inspector).
- **Fleet Standard LLM Settings Card (`crates/calibre-app/src/app.rs`)**:
  - Provider selector supporting `Ollama (Local :11434)`, `LM Studio (Local :1234)`, `OpenAI (Cloud)`, `Anthropic (Cloud)`, and `Custom / OpenAI-Compatible`.
  - Configurable Base URL and default models (`gemma4:12b`, `gpt-4o-mini`, `claude-sonnet-4-20250514`).
  - OS Credential Manager integration via `keyring` storing API keys securely in Windows Credential Manager under `rust-calibre-mcp` (never written in plain text to disk).
  - Dynamic model elicitation button querying `/api/tags` or `/v1/models` to automatically populate a live `egui::ComboBox` dropdown model picker.
  - Live connection test button executing a lightweight non-streaming probe and rendering an interactive status pill with latency metrics.
- **23 Canonical Commands (`calibre-core::ALL_COMMANDS`)** covering:
  - Multi-library discovery (`calibre.libraries`, `calibre.library_stats`, `calibre.import_config`) across `%APPDATA%\calibre\gui.json`, `global.py.json`, and sibling Calibre library folders.
  - Book search & inspection (`calibre.query_books`, `calibre.book_get`, `calibre.book_cover`, `calibre.book_excerpt`, `calibre.book_update`, `calibre.search_all_libraries`).
  - Full-text search (`calibre.search_fulltext`) against `full-text-search.db` (`books_text` + `books_fts` FTS5).
  - Taxonomy & analytics (`calibre.authors`, `calibre.tags`, `calibre.series`, `calibre.publishers`, `calibre.library_health`, `calibre.duplicates`, `calibre.reading_stats`, `calibre.unread_priority`, `calibre.export_json`).
  - Fleet crossconnect & local LLM (`bridge.speak`, `bridge.event`, `llm.chat` defaulting to `gemma4:12b`, `status`).
- **Dual-Aliased MCP 2024-11-05 Server (`rust-calibre-mcp mcp-stdio`)**:
  - Exposes all 23 canonical tool names plus legacy Python `calibre-mcp` portmanteau aliases (`manage_libraries`, `query_books`, `search_fulltext`, `manage_books`, `inspect_contents`, `manage_viewer`, `manage_authors`, `manage_tags`, `get_series_analysis`, `canon_coverage`, `analyze_library_health`, `find_duplicate_books`, `reading_statistics`, `unread_priority_list`, `cross_library_search_helper`, `manage_comments`, `manage_metadata`, `export_books`, `calibre_metadata_export_json`).
- **SQLite Contention Safety**:
  - All read operations open `metadata.db` and `full-text-search.db` strictly with `SQLITE_OPEN_READ_ONLY | SQLITE_OPEN_NO_MUTEX` + `5000 ms` `busy_timeout`.
  - Metadata updates (`calibre.book_update`) are guarded by an explicit write-safety fence (`confirm_write = false` returns a read-only dry-run preview).
- **Single-File Windows Installer**:
  - Current-user (no admin, no UAC) NSIS installer `Calibre-Rust-0.4.0-setup.exe` (~13.2 MB) packaging both `calibre-app.exe` and `rust-calibre-mcp.exe`.
