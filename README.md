# rust-calibre-mcp

Native Rust Artcraft conversion of [`calibre-mcp`](../calibre-mcp) -- a single-workspace 3-crate Calibre library engine, CLI + MCP 2024-11-05 server, and `eframe`/`egui` 0.31 native desktop application.

## Why `rust-calibre-mcp`?

- **Zero-Corruption Read-Only SQLite Guarantee**: Every connection to Calibre's `metadata.db` and `full-text-search.db` uses `OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX` + `busy_timeout(5000 ms)`, safe to run concurrently while Calibre GUI is open on the same library.
- **Custom Tokenizer Bypass on `full-text-search.db`**: Queries `full-text-search.db:books_text` directly with snippet extraction (`substr` + `instr`), avoiding Calibre's proprietary `calibre remove_diacritics` FTS5 C tokenizer so standalone `rusqlite` works with zero external C plugins.
- **Dual Tool-Dispatcher Aliasing**: Exposes 17 canonical `calibre.*` MCP tools (`calibre.libraries_list`, `calibre.query_books`, `calibre.search_fulltext`, `calibre.library_health`, etc.) **and** routes all 29 legacy FastMCP 3.2 portmanteau tool names (`manage_libraries`, `query_books`, `search_fulltext`, `bookcase_catalog`, `manage_books`, `analyze_library_health`, etc.) to the exact same `calibre_core::dispatch()` entry point.
- **Auto-Discovery Across Fleet & `L:\`**: Automatically discovers Calibre libraries from `--library`, `CALIBRE_LIBRARY_PATH`, `../calibre-mcp/config.json`, and `L:\Multimedia Files\Written Word\*` (11 live libraries on `Goliath`), plus a deterministic 6-book `Demo` library for offline tests.
- **Local LLM Book Copilot**: Built-in `llm.rs` client defaults to `gemma4:12b` (`7.47 GB` on Ollama, comfortable fit on RTX 4090 24 GB VRAM alongside `wgpu`).

## Workspace Layout (L1 / L2 / L3)

| Layer | Crate | Role |
|---|---|---|
| **L1 Core** | [`crates/calibre-core`](crates/calibre-core) | Pure library (`calibre_db`, `import`, `llm`, `bridge`, `creds`, `error`). Zero stdout/stderr, typed `Command` + `dispatch()`. |
| **L2 CLI + MCP** | [`crates/calibre-mcp`](crates/calibre-mcp) | `rust-calibre-mcp` binary: `clap` subcommands (`--json`) + `mcp-stdio` JSON-RPC 2.0 server (`2024-11-05`). |
| **L3 GUI** | [`crates/calibre-app`](crates/calibre-app) | `calibre-app` binary: `eframe`/`egui` 0.31 native desktop app (`Libraries`, `Books`, `Fulltext`, `Analytics`, `Chat`, `Crossconnect`, `Settings`, `Help`, `RawJson`) + `--smoke` headless probe. |

## Quick Start

```powershell
# List all discovered Calibre libraries (plus Demo)
cargo run -q -p rust-calibre-mcp -- libraries

# Library statistics for your main Calibre library
cargo run -q -p rust-calibre-mcp -- library-stats --library "Calibre-Bibliothek"

# Multi-filter book search
cargo run -q -p rust-calibre-mcp -- query-books --library "Calibre-Bibliothek IT" --query "Rust" --limit 10

# Full-text search inside full-text-search.db
cargo run -q -p rust-calibre-mcp -- fts --library "Calibre-Bibliothek IT" --query "tokio" --limit 5

# Library health audit (missing formats, covers, ISBNs, comments, series gaps)
cargo run -q -p rust-calibre-mcp -- health --library "Calibre-Bibliothek"

# Headless GUI smoke test
cargo run -q -p calibre-app -- --smoke
```

## Verification Gates

```powershell
just check
```
