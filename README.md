# rust-calibre-mcp

Native Rust Artcraft conversion of [`calibre-mcp`](../calibre-mcp) — a single-workspace 3-crate Calibre library engine (`calibre-core`), CLI + MCP 2024-11-05 server (`rust-calibre-mcp`), and `eframe`/`egui 0.37` native desktop application (`calibre-app`).

## Why `rust-calibre-mcp`?

- **Zero-Corruption Read-Only SQLite Guarantee**: Every read connection to Calibre's `metadata.db` and `full-text-search.db` uses `OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX` + `busy_timeout(5000 ms)`, safe to run concurrently while the Calibre desktop GUI is open on the same library. Metadata updates (`calibre.book_update`) are guarded by an explicit write-safety fence (`confirm_write = false` returns a read-only dry-run preview).
- **Custom Tokenizer Bypass on `full-text-search.db`**: Queries `full-text-search.db:books_text` directly with snippet extraction (`substr` + `instr`) and paginated character-safe excerpt reading (`calibre.book_excerpt`), avoiding Calibre's proprietary `calibre remove_diacritics` FTS5 C tokenizer so standalone `rusqlite` works with zero external C plugins.
- **23 Canonical Commands + Dual Tool-Dispatcher Aliasing**: Exposes 23 canonical `calibre.*` / `bridge.*` / `llm.*` MCP tools **and** routes legacy FastMCP portmanteau tool names (`manage_libraries`, `query_books`, `search_fulltext`, `bookcase_catalog`, `manage_books`, `inspect_contents`, `manage_viewer`, `manage_authors`, `manage_tags`, `get_series_analysis`, `canon_coverage`, `analyze_library_health`, `find_duplicate_books`, `reading_statistics`, `unread_priority_list`, `cross_library_search_helper`, `manage_comments`, `manage_metadata`, `export_books`, `calibre_metadata_export_json`) to `calibre_core::dispatch()`.
- **Auto-Discovery Across Fleet & `L:\`**: Automatically discovers all 11 Calibre libraries (`14,193` books) from `%APPDATA%\calibre\gui.json`, `global.py.json`, and sibling directories on `L:\Multimedia Files\Written Word\*`, plus a deterministic 6-book `Demo-Library` fallback for offline/CI tests.
- **Local LLM Book Copilot**: Built-in `llm.rs` client defaults to `gemma4:12b` on Ollama (comfortable fit on RTX 4090 24 GB VRAM alongside `wgpu`).

## Workspace Layout (L1 / L2 / L3)

| Layer | Crate | Role |
|---|---|---|
| **L1 Core** | [`crates/calibre-core`](crates/calibre-core/README.md) | Pure library (`calibre_db`, `import`, `llm`, `bridge`, `creds`, `error`). Zero stdout/stderr, typed `Command` + `dispatch()` across 23 canonical commands. |
| **L2 CLI + MCP** | [`crates/calibre-mcp`](crates/calibre-mcp/README.md) | `rust-calibre-mcp` binary: `clap 4.6` subcommands + `mcp-stdio` JSON-RPC 2.0 server (`2024-11-05`). |
| **L3 GUI** | [`crates/calibre-app`](crates/calibre-app/README.md) | `calibre-app` binary: `eframe`/`egui 0.37` native desktop app (9 pages, live `cover.jpg` texture rendering, paginated book excerpt reader, federated 11-library search, reading stats & unread priority queue) + `--smoke` headless probe. |

## Quick Start

```powershell
# List all 11 discovered Calibre libraries (plus Demo fallback)
cargo run -q -p rust-calibre-mcp -- libraries

# Library statistics for your main Calibre library
cargo run -q -p rust-calibre-mcp -- library-stats --library "Calibre-Bibliothek"

# Multi-filter book search
cargo run -q -p rust-calibre-mcp -- query-books --library "Calibre-Bibliothek IT" --query "Rust" --limit 10

# Federated search across all 11 discovered Calibre libraries
cargo run -q -p rust-calibre-mcp -- search-all --query "Python" --limit-per-library 3

# Read a book excerpt from full-text-search.db:books_text
cargo run -q -p rust-calibre-mcp -- book-excerpt --library "Calibre-Bibliothek IT" --book-id 4 --max-chars 500

# Reading statistics (languages, publishers, decades, ratings) & Unread Priority Queue
cargo run -q -p rust-calibre-mcp -- reading-stats --library "Calibre-Bibliothek"
cargo run -q -p rust-calibre-mcp -- unread-priority --library "Calibre-Bibliothek IT" --limit 5

# Library health audit (missing formats, covers, ISBNs, comments, series gaps)
cargo run -q -p rust-calibre-mcp -- library-health --library "Calibre-Bibliothek"

# Headless GUI smoke test
cargo run -q -p calibre-app -- --smoke
```

## Documentation

- [`INSTALL.md`](INSTALL.md) — Build, install, and MCP client registration
- [`CHANGELOG.md`](CHANGELOG.md) — Release history (`v0.4.0`)
- [`docs/UI_PARITY.md`](docs/UI_PARITY.md) — `egui 0.37` ArtCraft desktop UI specification
- [`docs/CONVERSION_DOSSIER.md`](docs/CONVERSION_DOSSIER.md) — Phase 0 due-diligence dossier
- [`docs/ASSESSMENT_2026_10_18.md`](docs/ASSESSMENT_2026_10_18.md) — `repconv` 100-point rubric assessment
- [`docs/CONVERSION_LOG.md`](docs/CONVERSION_LOG.md) — Phase-by-phase telemetry & oracle verification

## Verification Gates

```powershell
just check
```
