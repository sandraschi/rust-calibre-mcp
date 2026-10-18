# Product Requirements Document (PRD) -- `rust-calibre-mcp`

**Date**: 2026-10-18
**Status**: Wave-0 Reference Pilot (`repconv` Belt-and-Suspenders Standard)
**Legacy Oracle**: `D:\Dev\repos\calibre-mcp` (`master@5abaad6`, untouched read-only)

## 1. Objective

Replace the 209-Python-file `calibre-mcp` stack (FastMCP + FastAPI + SQLAlchemy + LanceDB + Vite React webapp) with a single native Rust workspace (`rust-calibre-mcp`) that starts in `< 15 ms`, consumes `< 25 MB` RAM idle, never mutates or locks Calibre's SQLite files, and provides:

1. **`calibre-core` (L1)**: Typed async `Command` / `dispatch()` core over `rusqlite` (`bundled`).
2. **`rust-calibre-mcp` (L2)**: CLI (`--json`) + `MCP 2024-11-05` stdio server with 17 canonical `calibre.*` tools and 29 legacy portmanteau aliases.
3. **`calibre-app` (L3)**: Native `eframe`/`egui` 0.31 desktop UI with 9 tabs, `Ctrl+Wheel` zoom (`0.75`..`2.0`), dark/light theme persistence, and Ollama (`gemma4:12b`) chat.

## 2. Hard Invariants (10-Point Uncrashable Binary Contract)

1. **Zero Panic Surface**: `#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unimplemented, clippy::todo)]` in every crate.
2. **Read-Only SQLite Guard**: `OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX` + `busy_timeout(5000 ms)`.
3. **Custom FTS Tokenizer Bypass**: Direct `books_text` scan in `full-text-search.db` so Calibre's custom `calibre` FTS5 tokenizer never causes a runtime error.
4. **Dual Tool-Dispatcher Aliasing**: Both canonical `calibre.*` tools and all 29 legacy FastMCP 3.2 portmanteau tool names resolve cleanly.
5. **Glyph Budget**: Only `['°', '×', '·', '…']` non-ASCII glyphs in `crates/calibre-app/src/app.rs`, verified by `glyph_source_scan`.
6. **Zero Legacy Mutation**: `D:\Dev\repos\calibre-mcp` remains 100% untouched.
