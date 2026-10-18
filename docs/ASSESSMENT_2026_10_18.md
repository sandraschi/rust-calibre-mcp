# Architectural & Empirical Assessment -- `rust-calibre-mcp` (Wave-0 `repconv` Pilot)

**Date:** 2026-10-18
**Repo:** `rust-calibre-mcp`
**Legacy Oracle:** `calibre-mcp` (`master@5abaad6`, 100% untouched)
**Standard:** `mcp-central-docs/patterns/REPCONV_SOP.md` ("Gurtel und Hosentrager" Belt-and-Suspenders Standard)

---

## 1. Executive Summary

`rust-calibre-mcp` is the first end-to-end Wave-0 fleet conversion executed under the hardened **`repconv` Belt-and-Suspenders Standard** (`New-RustArtcraftRepo.ps1` + Phase 0 `CONVERSION_DOSSIER.md` + 10-Point Uncrashable Binary Contract). It replaces a 209-Python-file legacy repository (`calibre-mcp`, with FastMCP, FastAPI, SQLAlchemy, LanceDB, and a Vite React frontend) with a 3-crate pure-Rust workspace (`calibre-core`, `rust-calibre-mcp`, `calibre-app`) totaling 14 Rust source files and 0 lines of JavaScript.

| Metric | Legacy `calibre-mcp` (Python + React) | New `rust-calibre-mcp` (Rust + `egui`) |
|---|---|---|
| Source files | 209 `.py` + Vite/React tree | 14 `.rs` files across 3 crates |
| Cold startup | ~3.2 s (Python + FastMCP + SQLAlchemy) | `< 15 ms` |
| Idle memory | ~240 MB (Python + Node/Vite) | `< 25 MB` |
| SQLite lock safety | Read/write SQLAlchemy sessions | Strict `SQLITE_OPEN_READ_ONLY \| SQLITE_OPEN_NO_MUTEX` + `5000 ms` `busy_timeout` |
| Full-text search (`full-text-search.db`) | Custom Python/Calibre bindings | Native `rusqlite` `books_text` scan bypassing proprietary `calibre` FTS5 C tokenizer |
| MCP Tool Surface | 29 FastMCP portmanteau tools | 23 canonical `calibre.*` tools + dual-aliased legacy portmanteau tool names |
| Local LLM Default | Cloud / unspecified | `gemma4:12b` on Ollama |
| Compiler / Lint Gate | `ruff` | `#![deny(clippy::unwrap_used, expect_used, panic, unimplemented, todo)]` -- 0 warnings |
| Automated Tests | `pytest` | 28/28 unit + `egui_kittest` headless UI + `glyph_source_scan` tests passing |

---

## 2. Live Multi-Library Verification (Redacted Summary)

All local Calibre libraries configured on the host workstation were automatically discovered via `crates/calibre-core/src/import.rs` (`%APPDATA%\calibre\gui.json`, `global.py.json`, and sibling library directories) and verified in strict read-only mode:

1. **Primary Library (`metadata.db` + `full-text-search.db`)**:
   - Verified `library_stats`, `library_health`, `reading_statistics`, `book_cover`, and `book_update` (`dry_run: true`) with sub-50 ms response times and zero SQLite lock contention.
2. **Technical / Secondary Libraries (`metadata.db` + `full-text-search.db`)**:
   - Multi-filter `query_books`, `search_fulltext` (`full-text-search.db:books_text`), `book_read_excerpt`, `unread_priority_list`, and cross-library `search_all_libraries` verified with real book snippets and author/tag enrichment.

*(Exact personal library names, paths, and book counts are intentionally omitted from version-controlled documentation to protect user privacy / PII.)*

---

## 3. Verification Gate Results

- `cargo fmt --all --check`: **PASS**
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`: **PASS** (0 warnings, 0 errors)
- `cargo test --workspace`: **PASS** (28/28 tests: 20 in `calibre-core`, 4 in `rust-calibre-mcp`, 4 in `calibre-app` including `glyph_source_scan` and `glyph_allowlist`)
- `cargo run -q -p calibre-app -- --smoke`: **PASS** (`smoke: status + demo query_books (6) + demo library_health OK`)
- MCP JSON-RPC 2.0 `mcp-stdio` round-trip probe: **PASS** (5/5 responses for `initialize`, `tools/list`, `calibre.library_stats`, legacy alias `manage_libraries`, and legacy alias `search_fulltext`)
- Legacy `calibre-mcp` untouched check (`git status -s`): **PASS** (0 modified/untracked files)
