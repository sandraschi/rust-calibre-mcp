# Architectural & Empirical Assessment -- `rust-calibre-mcp` (Wave-0 `repconv` Pilot)

**Date:** 2026-10-18
**Repo:** `D:\Dev\repos\rust-calibre-mcp`
**Legacy Oracle:** `D:\Dev\repos\calibre-mcp` (`master@5abaad6`, 100% untouched)
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
| MCP Tool Surface | 29 FastMCP portmanteau tools | 17 canonical `calibre.*` tools + 29 dual-aliased legacy portmanteau tool names |
| Local LLM Default | Cloud / unspecified | `gemma4:12b` (`7.47 GB` on Ollama, RTX 4090 24 GB VRAM safe alongside `wgpu`) |
| Compiler / Lint Gate | `ruff` | `#![deny(clippy::unwrap_used, expect_used, panic, unimplemented, todo)]` -- 0 warnings |
| Automated Tests | `pytest` | 27/27 unit + `egui_kittest` headless UI + `glyph_source_scan` tests passing |

---

## 2. Live Multi-Library Verification on `Goliath` (`L:\Multimedia Files\Written Word`)

All 11 live Calibre libraries on `L:\Multimedia Files\Written Word\` were automatically discovered via `crates/calibre-core/src/import.rs` and queried in read-only mode:

1. **`Calibre-Bibliothek` (Default Library -- `34.36 MB` `metadata.db` + `full-text-search.db`)**:
   - **Total Books:** `10,339`
   - **Total Authors:** `6,309`
   - **Total Tags:** `3,142`
   - **Total Publishers:** `1,678`
   - **Total Series:** `378`
   - **Format Breakdown:** `8,000` EPUB, `2,604` PDF, `1,016` MOBI, `41` AZW3, `25` DOCX, `22` TXT, `10` ORIGINAL_EPUB, `7` PRC, `5` AZW, `3` DJVU, `2` FB2, `2` CBR, `1` LIT, `1` CBZ
   - **Metadata Health:** `100.0%` cover coverage (`10,339 / 10,339`), `90.2%` comment/synopsis coverage (`9,327 / 10,339`), `25` duplicate title groups identified.
2. **`Calibre-Bibliothek IT` (`4.10 MB` `metadata.db` + `full-text-search.db`)**:
   - Multi-filter `query_books` and full-text `search_fulltext` (`full-text-search.db:books_text`) verified with real book snippets and author/tag enrichment.
3. **Additional 9 Live Libraries Verified**:
   - `Calibre-BibliothekAT`, `Calibre-Bibliothek Bullshit`, `Calibre-Bibliothek Cooking`, `Calibre-Bibliothek Custom`, `Calibre-Bibliothek Deutsch`, `Calibre-Bibliothek Draft`, `Calibre-Bibliothek Erotik`, `Calibre-Bibliothek France`, `Calibre-Bibliothek Manga`.

---

## 3. Verification Gate Results

- `cargo fmt --all --check`: **PASS**
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`: **PASS** (0 warnings, 0 errors)
- `cargo test --workspace`: **PASS** (27/27 tests: 19 in `calibre-core`, 4 in `rust-calibre-mcp`, 4 in `calibre-app` including `glyph_source_scan` and `glyph_allowlist`)
- `cargo run -q -p calibre-app -- --smoke`: **PASS** (`smoke: status + demo query_books (6) + demo library_health OK`)
- MCP JSON-RPC 2.0 `mcp-stdio` round-trip probe: **PASS** (5/5 responses for `initialize`, `tools/list`, `calibre.library_stats`, legacy alias `manage_libraries`, and legacy alias `search_fulltext`)
- Legacy `D:\Dev\repos\calibre-mcp` untouched check (`git status -s`): **PASS** (0 modified/untracked files)
