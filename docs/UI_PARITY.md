# UI Parity & ArtCraft Desktop Specification (`calibre-app`)

`crates/calibre-app` is the L3 native `egui 0.37` (`eframe 0.37`) desktop frontend for `rust-calibre-mcp`. It replaces the legacy Python/React `calibre-mcp/web_sota` browser UI with a zero-daemon, zero-port native desktop executable.

## 1. Visual & Interaction Standards

| Requirement | Implementation in `crates/calibre-app/src/app.rs` | Verification |
|---|---|---|
| **Harsh Dark & Crisp Light Themes** | `Theme::Dark` (`#0B0D12` bg, `#12151E` panel, `#6366F1` indigo accent) & `Theme::Light` (`#F5F6FA` bg, `#FFFFFF` cards) | Topbar Sun/Moon `egui::Painter` toggle |
| **Safe `Ctrl+Wheel` Zoom** | Reads `ctx.input(\|i\| i.zoom_delta())` inside closure, calls `ctx.set_zoom_factor(self.zoom)` **outside** closure (prevents `Context::input` RwLock deadlock) | `50%` – `200%` with one-click reset badge |
| **Strict Glyph Allowlist** | Only ASCII + `['°', '×', '·', '…']` permitted in `app.rs` (zero tofu boxes in `egui` default font) | `parity_tests::glyph_source_scan` + `glyph_allowlist` |
| **Async Background Worker** | Dedicated `tokio` runtime thread + `mpsc::unbounded_channel<Job>` keeps UI rendering at 60 FPS during multi-library SQLite scans | `self.drain_jobs(&ctx)` |
| **Headless CI Smoke Mode** | `calibre-app --smoke` runs `status` + `query_books(demo)` + `library_health(demo)` and exits `0` without opening a window | `cargo run -p calibre-app -- --smoke` |

## 2. Page-by-Page Parity Map

1. **Libraries**:
   - Hero KPIs (`Discovered Libraries`, `Total Books Indexed`, `Active Library`).
   - Horizontal bar chart of books per library across all discovered Calibre libraries.
   - Per-library cards showing `metadata.db` size in MB, `FTS5 READY` badge, book count, and `Activate & Browse` button.
2. **Books & Inspector**:
   - Instant filter bar (`Search`, `Author`, `Tag`, `Format`, `Search All Libraries`, `Clear`).
   - Split view: left column book cards with formats & synopsis snippet; right column **Book Inspector** with live `cover.jpg` texture rendering (`egui::Image`), `Read Excerpt` paginated reader (`full-text-search.db:books_text`), `Speak Synopsis (speech-mcp)`, `Ask Gemma 4`, and disk format file sizes.
3. **Full-Text Search**:
   - Searches inside book bodies via `full-text-search.db` (`books_text` + `books_fts` FTS5) with one-click jump into Book Inspector.
4. **Health, Series, Reading Statistics & Unread Priority**:
   - Hero KPIs (`Total Books`, `Cover Coverage %`, `Synopsis Coverage %`, `Duplicate Title Groups`).
   - Publication decade bar chart (`calibre.reading_stats`).
   - Two-column view: **Series & Canon Coverage (Gap Detector)** on the left, **Unread Priority Queue** (`calibre.unread_priority`) with priority scores and reasons on the right.
5. **AI Librarian (Gemma 4)**:
   - Local Ollama (`gemma4:12b` default) or OpenAI-compatible chat with active library context injection.
6. **Crossconnect**:
   - `speech-mcp` (`:10909`) voice synthesis + `aiwatcher` (`:10946`) heartbeat webhook dispatch.
7. **Settings & Vault**:
   - Windows Credential Manager (`keyring` crate) storage for optional Calibre Content Server credentials + local `libraries.json` pinning.
8. **Help**:
   - Keyboard shortcuts, zoom controls, and SQLite read-only safety documentation.
9. **Raw JSON**:
   - Monospace inspector for the most recent L1 command JSON payload.
