# Conversion Dossier: `rust-calibre-mcp` (from `calibre-mcp`)

**Generated:** 2026-10-18T01:05:30+02:00  
**Legacy Repo Path (100% READ-ONLY):** `D:\Dev\repos\calibre-mcp`  
**Legacy Git Baseline:** branch `master` @ commit `5abaad6`  
**Target Rust Repo:** `D:\Dev\repos\rust-calibre-mcp`

---

## 1. Bucket & Privilege-Split / Hidden Binary Audit
- **Subdirectories / Native Binaries Detected in `D:\Dev\repos\calibre-mcp`:**
  - `Cargo.toml` (root) + `src/lib.rs` + `extension.wasm`: An 18-line Zed editor extension bridge (`zed_extension_api = "0.2.0"`), **not** an MCP server or desktop app.
  - `native/`: Tauri 2 NSIS wrapper around the Python backend (`10720`/`10721`).
  - Zero elevated `LocalSystem` services or kernel drivers.
- **Triage Classification:** **Bucket A (100% Pure-Rust Local SQLite + Optional Content-Server REST + Optional `calibredb.exe` CLI wrapper).**
  - Calibre's primary data store (`metadata.db`) and full-text search index (`full-text-search.db`) are standard SQLite 3 databases inside each library directory on `L:\Multimedia Files\Written Word\`.
  - Optional LanceDB vector RAG (`rag_index_build`, `rag_retrieve`) in the legacy Python server is replaced/handled in Rust via SQLite FTS5 (`full-text-search.db` + `metadata.db` multi-field query) + optional local Ollama `nomic-embed-text` / `gemma4:12b` semantic synthesis.

---

## 2. Live Service, Port & Database Contention Audit
- **Legacy Running Service / Port Check:**
  - Checked ports `10720` (backend) and `10721` (frontend) via `Get-NetTCPConnection`: neither is currently listening.
  - Checked `L:\Multimedia Files\Written Word\`: **11 live Calibre libraries** are mounted and online right now:
    1. `Calibre-Bibliothek` (`34.36 MB` `metadata.db`, `full-text-search.db` present)
    2. `Calibre-Bibliothek IT` (`4.10 MB` `metadata.db`, `full-text-search.db` present)
    3. `Calibre-Bibliothek Deutsch` (`2.08 MB` `metadata.db`, `full-text-search.db` present)
    4. `Calibre-Bibliothek Bullshit` (`1.60 MB` `metadata.db`, `full-text-search.db` present)
    5. `Calibre-Bibliothek Comics` (`0.94 MB` `metadata.db`)
    6. `Calibre-Bibliothek Japanisch` (`0.75 MB` `metadata.db`)
    7. `Calibre-Bibliothek Manga` (`0.58 MB` `metadata.db`)
    8. `Calibre-Bibliothek Magazines` (`0.52 MB` `metadata.db`)
    9. `Calibre-Bibliothek-Private-2` (`0.52 MB` `metadata.db`)
    10. `Calibre-Bibliothek Private` (`0.46 MB` `metadata.db`)
    11. `Calibre-Bibliothek Test` (`0.39 MB` `metadata.db`)
- **Contention Strategy (Gürtel und Hosenträger):**
  - Every read query against `metadata.db` and `full-text-search.db` opens the SQLite connection with `OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX` + `busy_timeout(Duration::from_millis(5000))`.
  - Even if Calibre Desktop GUI is open on `Calibre-Bibliothek`, `rust-calibre-mcp` can **never** lock or corrupt `metadata.db`.
  - `D:\Dev\repos\calibre-mcp` (`master@5abaad6`) remains 100% untouched.

---

## 3. Complete 1:1 MCP Tool & Portmanteau Mapping Table

All active tools registered by `calibre-mcp` are mapped below so `rust-calibre-mcp` supports **both** clean ArtCraft tool names (`calibre.<action>` / `calibre_<action>`) **and** 1:1 legacy Python tool names (including `operation=` parameters):

| Legacy Python Tool (`@mcp.tool`) | Rust `Command` Variant | CLI Subcommand | Rust MCP Tool + Legacy Alias |
|---|---|---|---|
| `manage_libraries` (`list`, `switch`, `stats`, `discover`) | `Command::LibrariesList`, `Command::LibraryStats` | `libraries`, `library-stats` | `calibre.libraries_list` + alias `manage_libraries` |
| `query_books` (`search`, `by_author`, `by_series`, `by_tag`, `recent`) | `Command::QueryBooks` | `query-books` | `calibre.query_books` + alias `query_books` |
| `manage_books` (`get`, `list`, `details`, `formats`) | `Command::BookGet`, `Command::BooksList` | `book-get`, `books-list` | `calibre.book_get` + alias `manage_books` |
| `search_fulltext` | `Command::SearchFulltext` | `search-fulltext` | `calibre.search_fulltext` + alias `search_fulltext` |
| `bookcase_catalog` | `Command::BookcaseCatalog` | `bookcase-catalog` | `calibre.bookcase_catalog` + alias `bookcase_catalog` |
| `manage_authors` (`list`, `get`, `stats`, `books`) | `Command::AuthorsList` | `authors` | `calibre.authors_list` + alias `manage_authors` |
| `manage_tags` / `get_tag_statistics` | `Command::TagsList`, `Command::TagStats` | `tags`, `tag-stats` | `calibre.tags_list` + aliases `manage_tags`, `get_tag_statistics` |
| `manage_comments` (`get`, `update`) | `Command::BookComments` | `comments` | `calibre.book_comments` + alias `manage_comments` |
| `manage_metadata` / `calibre_metadata_search` / `calibre_metadata_export_json` | `Command::MetadataInspect`, `Command::MetadataExport` | `metadata`, `metadata-export` | `calibre.metadata_inspect` + aliases `manage_metadata`, `calibre_metadata_search`, `calibre_metadata_export_json` |
| `get_series_analysis` / `canon_coverage` | `Command::SeriesAnalysis` | `series` | `calibre.series_analysis` + aliases `get_series_analysis`, `canon_coverage` |
| `find_duplicate_books` | `Command::FindDuplicates` | `duplicates` | `calibre.find_duplicates` + alias `find_duplicate_books` |
| `analyze_library_health` / `manage_analysis` / `reading_statistics` / `unread_priority_list` | `Command::LibraryHealth` | `health` | `calibre.library_health` + aliases `analyze_library_health`, `manage_analysis`, `reading_statistics`, `unread_priority_list` |
| `inspect_contents` / `manage_files` / `export_books` / `manage_viewer` | `Command::BookFiles` | `files` | `calibre.book_files` + aliases `inspect_contents`, `manage_files`, `export_books`, `manage_viewer` |
| `rag_retrieve` / `calibre_rag` / `rag_index_build` / `calibre_metadata_index_build` | `Command::RagSearch` | `rag-search` | `calibre.rag_search` + aliases `rag_retrieve`, `calibre_rag`, `rag_index_build`, `calibre_metadata_index_build` |
| `manage_system` / `show_api_docs` / `help_tool` / `show_book_prefab_card` / `show_libraries_prefab_card` | `Command::Status` | `status` | `calibre.status` + aliases `manage_system`, `help_tool`, `show_api_docs` |
| `agentic_library_workflow` / `manage_bulk_operations` / `manage_content_sync` / `manage_smart_collections` | `Command::LlmChat` / `Command::LibraryHealth` | `chat` | `llm.chat` + alias `agentic_library_workflow` |
| *(Fleet standard crossconnect & LLM)* | `Command::BridgeEvent`, `BridgeSpeak`, `BridgeWebhook`, `LlmChat`, `LlmModels` | `bridge`, `chat`, `models` | `bridge.event`, `bridge.speak`, `bridge.webhook`, `llm.chat`, `llm.models` |

---

## 4. Credential, `.env`, Config & Data-Path Inventory (`import.rs` Spec)
`crates/calibre-core/src/import.rs` will auto-discover libraries and settings in strict priority order:
1. Explicit CLI/MCP parameter (`library_path` / `library_name`)
2. Process environment variables (`CALIBRE_LIBRARY_PATH`, `CALIBRE_BASE_PATH`, `CALIBRE_LIBRARY_NAME`, `CALIBRE_SERVER_URL`, `CALIBRE_USERNAME`, `CALIBRE_PASSWORD`)
3. Sibling `D:\Dev\repos\calibre-mcp\config.json` (`local_library_path` + `discovered_libraries` map of all 11 libraries on `L:\Multimedia Files\Written Word\`) and `D:\Dev\repos\calibre-mcp\.env`
4. OS Calibre config (`%APPDATA%\calibre\global.py.json` / `gui.json`) and directory scan of `L:\Multimedia Files\Written Word\*` for `metadata.db`
5. Deterministic `Vendor::Demo` (built-in 6-book sample library with authors, tags, series, formats, and FTS excerpts so CI and `--smoke` pass on any naked machine without `L:\`).

---

## 5. Write-Safety & Destructive-Action Fence Design
- **Default 100% Read-Only SQLite Access:** All library, book, author, tag, series, duplicate, health, and FTS queries open `metadata.db` and `full-text-search.db` with `SQLITE_OPEN_READ_ONLY`.
- **Mutating Operations Gate:** Any tag/comment/metadata mutation requires explicit `readonly: false` (`--write` on CLI) and operates only on `Calibre-Bibliothek Test` or via safe parameterized transaction when explicitly unlocked.

---

## 6. L3 `egui` Desktop Page & Golden Snapshot Plan
- **Domain Pages in `crates/calibre-app/src/app.rs`:**
  1. `Page::Libraries` -- Overview of all 11 discovered Calibre libraries (`book_count`, `DB size MB`, `FTS5 status`, 1-click active library switcher, and bar chart of books per library).
  2. `Page::Books` -- Instant search + filter bar (title, author, tag, series, format `EPUB`/`PDF`/`CBZ`), book grid/table with cover preview (`cover.jpg`), and right-hand book detail inspector (comments/synopsis, identifiers, file sizes, `Read`/`Speak Synopsis` via `speech-mcp`).
  3. `Page::Analytics` -- Series completion (`canon_coverage`), duplicate detector (`find_duplicate_books`), tag distribution chart, and library health KPIs.
- **Inherited Kernel Pages:** `Page::Crossconnect`, `Page::Chat` (`gemma4:12b` default with `Inject Library Context` button), `Page::Settings` (OS Credential Manager `vault.rs`), `Page::Help`, `Page::RawJson`.
- **Golden `egui_kittest` Snapshots:** `libraries_dark`, `libraries_light`, `books_dark`, `analytics_dark`, `help_dark`.