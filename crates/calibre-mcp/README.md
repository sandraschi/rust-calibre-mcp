# `rust-calibre-mcp` (L2 CLI + MCP 2024-11-05 Server)

`rust-calibre-mcp` is the thin L2 binary crate wrapping `calibre-core` (`L1`). It provides both:

1. **Human & Script CLI (`clap 4.6`)**: Subcommands for all 23 canonical operations (`libraries`, `library-stats`, `query-books`, `book-get`, `book-cover`, `book-excerpt`, `book-update`, `search-all-libraries`, `search-fulltext`, `authors`, `tags`, `series`, `publishers`, `library-health`, `duplicates`, `reading-stats`, `unread-priority`, `export-json`, `import-config`, `speak`, `event`, `chat`, `status`).
2. **MCP 2024-11-05 JSON-RPC 2.0 Stdio Server (`mcp-stdio`)**: Exposes all 23 canonical `calibre.*` tools and accepts legacy Python `calibre-mcp` portmanteau tool names (`manage_libraries`, `query_books`, `search_fulltext`, `manage_books`, `inspect_contents`, `manage_viewer`, `manage_authors`, `manage_tags`, `get_series_analysis`, `canon_coverage`, `analyze_library_health`, `find_duplicate_books`, `reading_statistics`, `unread_priority_list`, `cross_library_search_helper`, `manage_comments`, `manage_metadata`, `export_books`, `calibre_metadata_export_json`).
