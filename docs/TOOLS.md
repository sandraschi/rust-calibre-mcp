# Tool Reference — `rust-calibre-mcp`

`rust-calibre-mcp` exposes 23 canonical MCP tools via `mcp-stdio` (MCP protocol version `2024-11-05`). It also accepts legacy Python `calibre-mcp` portmanteau tool names as transparent aliases.

## Canonical Tools

| Tool Name | Parameters | Description |
|---|---|---|
| `calibre.libraries` | `include_counts: bool` | List all discovered Calibre libraries with metadata size and book counts. |
| `calibre.library_stats` | `library: Option<String>` | Comprehensive statistics (book count, author count, tag count, series, formats). |
| `calibre.query_books` | `library, query, author, tag, series, format, limit, offset` | Multi-filter book search with rich metadata joins. |
| `calibre.book_get` | `library, book_id` | Get complete book details, identifiers, format files, and synopsis. |
| `calibre.book_cover` | `library, book_id` | Retrieve `cover.jpg` file path and image metadata. |
| `calibre.book_excerpt` | `library, book_id, format, offset_chars, max_chars` | Read a character-safe text slice from `full-text-search.db:books_text`. |
| `calibre.book_update` | `library, book_id, title, comment_html, confirm_write` | Write-safety fenced metadata update (dry-run preview by default). |
| `calibre.search_all_libraries` | `query, limit_per_library` | Federated cross-library search across all discovered libraries. |
| `calibre.search_fulltext` | `library, query, limit` | Full-text keyword search inside book contents via `full-text-search.db`. |
| `calibre.authors` | `library, limit` | List top authors sorted by book count. |
| `calibre.tags` | `library, limit` | List top tags/genres sorted by book count. |
| `calibre.series` | `library, limit` | List series and detect missing volume indices. |
| `calibre.publishers` | `library, limit` | List top publishers with book counts. |
| `calibre.library_health` | `library` | Health audit (cover coverage %, synopsis coverage %, duplicate titles). |
| `calibre.duplicates` | `library` | Identify books sharing identical titles across the library. |
| `calibre.reading_stats` | `library` | Language breakdown, publication decades, and rating distributions. |
| `calibre.unread_priority` | `library, limit` | Ranked reading queue based on rating, series opener status, and format readiness. |
| `calibre.export_json` | `library, limit` | Export library catalog metadata as structured JSON. |
| `calibre.import_config` | — | Inspect discovered library paths and legacy configuration status. |
| `bridge.speak` | `text, voice, url` | Dispatch voice synthesis request to `speech-mcp` (`:10909`). |
| `bridge.event` | `kind, device, payload, url` | Dispatch event webhook to `aiwatcher` (`:10946`). |
| `llm.chat` | `provider, model, prompt, system, context, base_url, api_key` | Query local or cloud LLM with optional library context. |
| `status` | — | Health check returning tool catalog and engine status. |

## Legacy Portmanteau Aliases

Existing prompts, agents, and IDE extensions using legacy Python tool names work without modification:
- `manage_libraries` → `calibre.libraries` / `calibre.library_stats`
- `query_books` → `calibre.query_books`
- `manage_books` → `calibre.book_get`
- `search_fulltext` → `calibre.search_fulltext`
- `inspect_contents` / `manage_viewer` → `calibre.book_cover` / `calibre.book_excerpt`
- `manage_comments` / `manage_metadata` → `calibre.book_update`
- `cross_library_search_helper` → `calibre.search_all_libraries`
- `manage_authors` → `calibre.authors`
- `manage_tags` / `get_tag_statistics` → `calibre.tags`
- `get_series_analysis` / `canon_coverage` → `calibre.series`
- `find_duplicate_books` → `calibre.duplicates`
- `analyze_library_health` / `manage_analysis` → `calibre.library_health`
- `reading_statistics` → `calibre.reading_stats`
- `unread_priority_list` → `calibre.unread_priority`
- `calibre_metadata_export_json` / `export_books` → `calibre.export_json`
- `manage_system` / `show_api_docs` / `help_tool` → `status`
