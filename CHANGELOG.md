# Changelog

All notable changes to `rust-calibre-mcp` are documented in this file.

## [0.4.0] - 2026-10-18

### Added
- **3-Crate ArtCraft Architecture (`calibre-core` L1, `rust-calibre-mcp` L2, `calibre-app` L3)** converted from `D:\Dev\repos\calibre-mcp` (`master@5abaad6`) under the `repconv` v2.1 standard.
- **23 Canonical Commands (`calibre-core::ALL_COMMANDS`)** covering:
  - Multi-library discovery (`calibre.libraries`, `calibre.library_stats`, `calibre.import_config`) across `%APPDATA%\calibre\gui.json`, `global.py.json`, and sibling Calibre folders (11 live libraries, 14,193 books on `L:\Multimedia Files\Written Word\`).
  - Book search & inspection (`calibre.query_books`, `calibre.book_get`, `calibre.book_cover`, `calibre.book_excerpt`, `calibre.book_update`, `calibre.search_all_libraries`).
  - Full-text search (`calibre.search_fulltext`) against `full-text-search.db` (`books_text` + `books_fts` FTS5).
  - Taxonomy & analytics (`calibre.authors`, `calibre.tags`, `calibre.series`, `calibre.publishers`, `calibre.library_health`, `calibre.duplicates`, `calibre.reading_stats`, `calibre.unread_priority`, `calibre.export_json`).
  - Fleet crossconnect & local LLM (`bridge.speak`, `bridge.event`, `llm.chat` defaulting to `gemma4:12b`, `status`).
- **Dual-Aliased MCP 2024-11-05 Server (`rust-calibre-mcp mcp-stdio`)**:
  - Exposes all 23 canonical tool names plus legacy Python `calibre-mcp` portmanteau aliases (`manage_libraries`, `query_books`, `search_fulltext`, `manage_books`, `inspect_contents`, `manage_viewer`, `manage_authors`, `manage_tags`, `get_series_analysis`, `canon_coverage`, `analyze_library_health`, `find_duplicate_books`, `reading_statistics`, `unread_priority_list`, `cross_library_search_helper`, `manage_comments`, `manage_metadata`, `export_books`, `calibre_metadata_export_json`).
- **Native `egui 0.37` (`eframe 0.37`) Desktop App (`calibre-app`)**:
  - 9 pages (Libraries, Books & Inspector with live cover art + paginated full-text excerpt reader + federated 11-library search, Full-Text Search, Health & Series + Reading Statistics + Unread Priority Queue, AI Librarian, Crossconnect, Settings & Windows Credential Manager Vault, Help, Raw JSON).
  - Custom `egui::Painter` Sun/Moon theme toggle, safe `Ctrl+MouseWheel` zoom, and `egui_kittest` glyph & parity tests.
- **SQLite Contention Safety**:
  - All read operations open `metadata.db` and `full-text-search.db` strictly with `SQLITE_OPEN_READ_ONLY | SQLITE_OPEN_NO_MUTEX` + `5000 ms` `busy_timeout`.
  - Metadata updates (`calibre.book_update`) are guarded by an explicit write-safety fence (`confirm_write = false` returns a read-only dry-run preview).
