# HANDOVER -- `rust-calibre-mcp`

**Last Updated:** 2026-10-18T05:15:00+02:00
**Current Phase:** Full Structural Webapp Clone Parity + Fleet Standard LLM Settings Card COMPLETE (23 canonical commands, 16 desktop pages in 5 categories, 29/29 unit tests, 0 clippy warnings, release `.exe` + NSIS installer v0.4.0 published)

## What Was Done
- Executed and deepened the Wave-0 `repconv` conversion pilot from `calibre-mcp` (`master@5abaad6`, untouched read-only) into `rust-calibre-mcp`.
- Built and verified all 3 crates (`calibre-core`, `rust-calibre-mcp`, `calibre-app`) with **23 canonical commands**:
  - `calibre.libraries`, `calibre.library_stats`, `calibre.query_books`, `calibre.book_get`, `calibre.book_cover`, `calibre.book_excerpt`, `calibre.book_update` (write-safety fenced), `calibre.search_all_libraries`, `calibre.search_fulltext`, `calibre.authors`, `calibre.tags`, `calibre.series`, `calibre.publishers`, `calibre.library_health`, `calibre.duplicates`, `calibre.reading_stats`, `calibre.unread_priority`, `calibre.export_json`, `calibre.import_config`, `bridge.speak`, `bridge.event`, `llm.chat` (`gemma4:12b` default), `status`.
- Replaced the initial 9-page dashboard with **100% structural clone parity matching the original webapp (all 16 pages across 5 grouped sections)**:
  - `CONTENT`: `Libraries`, `Books & Inspector`, `Authors` (A-Z initial filter bar + name search + volume counts), `Series` (canon coverage & missing volume gap detector), `Tags` (taxonomy badges with counts), `Publishers` (top publishing houses).
  - `DISCOVER`: `Full-Text Search`, `Reading & Priorities` (publication decade chart + read/unread ratio KPIs + ranked priority queue), `Duplicates` (normalized title duplicate groups with format breakdown).
  - `MANAGE`: `Library Health` (KPI cards: Total Books, Cover Coverage %, Synopsis Coverage %, Duplicate Title Groups, plus missing covers/comments/tags breakdowns).
  - `AI & FLEET`: `AI Librarian` (conversational assistant with active library context), `Crossconnect` (`speech-mcp` voice synthesis + `aiwatcher` telemetry webhooks).
  - `SYSTEM`: `System Logs` (real-time timestamped event log with text filtering and reset), `Settings & Vault` (library path pinning, Calibre server vault, and Fleet Standard LLM Card), `Help` (shortcuts, zoom, SQLite safety guarantees), `Raw JSON` (live L1 command payload inspector).
- Implemented the **Fleet Standard LLM Settings Card**:
  - Provider selector supporting `Ollama (Local :11434)`, `LM Studio (Local :1234)`, `OpenAI (Cloud)`, `Anthropic (Cloud)`, and `Custom / OpenAI-Compatible`.
  - Configurable Base URL and default models (`gemma4:12b`, `gpt-4o-mini`, `claude-sonnet-4-20250514`).
  - Windows Credential Manager OS vaulting via `keyring` (API keys stored securely under `rust-calibre-mcp`, never in plain text on disk).
  - Dynamic model elicitation querying `/api/tags` or `/v1/models` into a live dropdown picker.
  - Live non-streaming connectivity test button with latency status pill.
- Upgraded `calibre-app` (`egui 0.37`) with:
  - Live `cover.jpg` texture rendering (`egui::Image`) in the Book Inspector.
  - Paginated `Read Excerpt` viewer reading from `full-text-search.db:books_text`.
  - One-click `Search All Libraries` federated search button.
  - Embedded window icon (`crates/calibre-app/icons/icon.png`).
- Codified fleet conversion standards in `mcp-central-docs` (`patterns/REPCONV_SOP.md` and `standards/rust-artcraft/EGUI_DESKTOP_UI_AND_PARITY.md`):
  - Mandatory Webapp Navigation Structure Audit in Phase 0.
  - The Full Structural Parity Law ("Clone, Not Inspired By").
  - Mandatory Fleet Standard LLM Settings Card.
- Redacted all personal library counts, folder names, and drive paths from docs and source files.
- All verification gates passed:
  - `cargo fmt --all --check` -> PASS
  - `cargo clippy --workspace --all-targets --all-features -- -D warnings` -> PASS (0 warnings)
  - `cargo test --workspace` -> PASS (29/29 tests including glyph scan and allowlist)
  - `cargo run -q -p calibre-app -- --smoke` -> PASS
  - Live read-only SQLite verification across local Calibre libraries -> PASS
  - Single-file installer `Calibre-Rust-0.4.0-setup.exe` compiled and published to GitHub Release v0.4.0 -> PASS

## What Was Deliberately Not Done
- Did not modify a single file in `calibre-mcp`.
- Did not touch `rust-plex-mcp` (handled by OpenCode).
- Did not mutate any live Calibre SQLite file (`calibre.book_update` defaults to `confirm_write = false` read-only dry-run preview).