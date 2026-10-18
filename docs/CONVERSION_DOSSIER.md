# Conversion Dossier: `rust-calibre-mcp` (from `calibre-mcp`)

**Generated:** 2026-10-18T01:05:30+02:00  
**Legacy Repo Path (100% READ-ONLY):** `../calibre-mcp`  
**Legacy Git Baseline:** branch `master` @ commit `5abaad6`  
**Target Rust Repo:** `rust-calibre-mcp`

---

## 1. Bucket & Privilege-Split / Hidden Binary Audit
- **Subdirectories / Native Binaries Detected in `calibre-mcp`:**
  - `Cargo.toml` (root) + `src/lib.rs` + `extension.wasm`: An 18-line Zed editor extension bridge (`zed_extension_api = "0.2.0"`), **not** an MCP server or desktop app.
  - `native/`: Tauri 2 NSIS wrapper around the Python backend (`10720`/`10721`).
  - Zero elevated `LocalSystem` services or kernel drivers.
- **Triage Classification:** **Bucket A (100% Pure-Rust Local SQLite + Optional Content-Server REST).**
  - Calibre's primary data store (`metadata.db`) and full-text search index (`full-text-search.db`) are standard SQLite 3 databases inside each Calibre library directory.
  - Optional LanceDB vector RAG (`rag_index_build`, `rag_retrieve`) in the legacy Python server is replaced/handled in Rust via SQLite FTS5 (`full-text-search.db` + `metadata.db` multi-field query) + optional local Ollama `gemma4:12b` semantic synthesis.

---

## 2. Live Service, Port & Database Contention Audit
- **Legacy Running Service / Port Check:**
  - Checked ports `10720` (backend) and `10721` (frontend) via `Get-NetTCPConnection`: neither is currently listening.
  - Verified local Calibre libraries discovered via `%APPDATA%\calibre\gui.json` and `global.py.json` (both `metadata.db` and `full-text-search.db` present). *(Personal library folder names, paths, and book counts redacted for privacy.)*
- **Contention Strategy (Gürtel und Hosenträger):**
  - Every read query against `metadata.db` and `full-text-search.db` opens the SQLite connection with `OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX` + `busy_timeout(Duration::from_millis(5000))`.
  - Even if Calibre Desktop GUI is open on the active library, `rust-calibre-mcp` can **never** lock or corrupt `metadata.db`.
  - Legacy `calibre-mcp` (`master@5abaad6`) remains 100% untouched.

---

## 3. Complete 1:1 MCP Tool & Portmanteau Mapping Table

All active tools registered by `calibre-mcp` are mapped below so `rust-calibre-mcp` supports **both** clean ArtCraft tool names (`calibre.<action>` / `calibre_<action>`) **and** 1:1 legacy Python tool names (including `operation=` parameters):

| Legacy Python Tool (`@mcp.tool`) | Rust `Command` Variant | CLI Subcommand | Rust MCP Tool + Legacy Alias |
|---|---|---|---|
| `manage_libraries` (`list`, `switch`, `stats`, `discover`) | `Command::LibrariesList`, `Command::LibraryStats` | `libraries`, `library-stats` | `calibre.libraries` + alias `manage_libraries` |
| `query_books` (`search`, `by_author`, `by_series`, `by_tag`, `recent`) | `Command::QueryBooks` | `query-books` | `calibre.query_books` + alias `query_books` |
| `manage_books` (`get`, `list`, `details`, `formats`) | `Command::BookGet` | `book-get` | `calibre.book_get` + alias `manage_books` |
| `search_fulltext` | `Command::SearchFulltext` | `search-fulltext` | `calibre.search_fulltext` + alias `search_fulltext` |
| `cross_library_search_helper` | `Command::SearchAllLibraries` | `search-all-libraries` | `calibre.search_all_libraries` + alias `cross_library_search_helper` |
| `manage_authors` (`list`, `get`, `stats`, `books`) | `Command::AuthorsList` | `authors` | `calibre.authors` + alias `manage_authors` |
| `manage_tags` / `get_tag_statistics` | `Command::TagsList` | `tags` | `calibre.tags` + aliases `manage_tags`, `get_tag_statistics` |
| `manage_comments` / `manage_metadata` | `Command::BookUpdate` | `book-update` | `calibre.book_update` + aliases `manage_comments`, `manage_metadata` |
| `calibre_metadata_export_json` / `export_books` | `Command::ExportJson` | `export-json` | `calibre.export_json` + aliases `calibre_metadata_export_json`, `export_books` |
| `get_series_analysis` / `canon_coverage` | `Command::SeriesList` | `series` | `calibre.series` + aliases `get_series_analysis`, `canon_coverage` |
| `find_duplicate_books` | `Command::Duplicates` | `duplicates` | `calibre.duplicates` + alias `find_duplicate_books` |
| `analyze_library_health` / `manage_analysis` | `Command::LibraryHealth` | `library-health` | `calibre.library_health` + aliases `analyze_library_health`, `manage_analysis` |
| `reading_statistics` | `Command::ReadingStats` | `reading-stats` | `calibre.reading_stats` + alias `reading_statistics` |
| `unread_priority_list` | `Command::UnreadPriority` | `unread-priority` | `calibre.unread_priority` + alias `unread_priority_list` |
| `inspect_contents` / `manage_viewer` | `Command::BookCover`, `Command::BookExcerpt` | `book-cover`, `book-excerpt` | `calibre.book_cover`, `calibre.book_excerpt` + aliases `inspect_contents`, `manage_viewer` |
| `manage_system` / `show_api_docs` / `help_tool` | `Command::Status` | `status` | `status` + aliases `manage_system`, `help_tool`, `show_api_docs` |
| *(Fleet standard crossconnect & LLM)* | `Command::BridgeEvent`, `BridgeSpeak`, `LlmChat` | `event`, `speak`, `chat` | `bridge.event`, `bridge.speak`, `llm.chat` |

---

## 4. Credential, `.env`, Config & Data-Path Inventory (`import.rs` Spec)
`crates/calibre-core/src/import.rs` auto-discovers libraries and settings in strict priority order:
1. Explicit CLI/MCP parameter (`library`)
2. Process environment variables (`CALIBRE_LIBRARY_PATH`, `CALIBRE_BASE_PATH`, `CALIBRE_SERVER_URL`, `CALIBRE_USERNAME`, `CALIBRE_PASSWORD`)
3. OS Calibre config (`%APPDATA%\calibre\gui.json` and `global.py.json`)
4. Sibling `../calibre-mcp/config.json` (`local_library_path` + `discovered_libraries`) and `../calibre-mcp/.env`
5. Sibling directory scan inside the parent folder of any discovered library root
6. Deterministic `Demo` fallback (built-in 6-book sample library with authors, tags, series, formats, and FTS excerpts so CI and `--smoke` pass on any headless machine).

---

## 5. Write-Safety & Destructive-Action Fence Design
- **Default 100% Read-Only SQLite Access:** All library, book, author, tag, series, duplicate, health, reading stats, priority, and FTS queries open `metadata.db` and `full-text-search.db` with `SQLITE_OPEN_READ_ONLY`.
- **Mutating Operations Gate:** Any title/comment/metadata mutation (`calibre.book_update`) defaults to `confirm_write = false` (returning a read-only dry-run preview) and only writes inside a parameterized transaction when `confirm_write = true` is explicitly passed.

---

## 6. L3 `egui` Desktop Page & Golden Snapshot Plan
- **Domain Pages in `crates/calibre-app/src/app.rs`:**
  1. `Page::Libraries` -- Overview of all discovered Calibre libraries (`book_count`, `DB size MB`, `FTS5 status`, 1-click active library switcher, and bar chart of books per library).
  2. `Page::Books` -- Instant search + filter bar (title, author, tag, series, format `EPUB`/`PDF`), federated multi-library search, book list, and right-hand book detail inspector (cover art preview, paginated full-text excerpt reader, comments/synopsis, file sizes, `Speak Synopsis` via `speech-mcp`).
  3. `Page::Fulltext` -- Full-text search across `full-text-search.db`.
  4. `Page::Analytics` -- Library health KPIs, publication decade chart, series completion (`canon_coverage`), and unread priority queue.
- **Inherited Kernel Pages:** `Page::Crossconnect`, `Page::Chat` (`gemma4:12b` default), `Page::Settings` (OS Credential Manager `vault.rs`), `Page::Help`, `Page::RawJson`.