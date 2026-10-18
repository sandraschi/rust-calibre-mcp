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

## 2. Page-by-Page Parity Map (All 16 Views in 5 Grouped Sections)

### Group 1: CONTENT
1. **Libraries**:
   - Hero KPIs (`Discovered Libraries`, `Total Books Indexed`, `Active Library`).
   - Horizontal bar chart of books per library across all discovered Calibre libraries.
   - Per-library cards showing `metadata.db` size in MB, `FTS5 READY` badge, book count, and `Activate & Browse` button.
2. **Books & Inspector**:
   - Instant filter bar (`Search`, `Author`, `Tag`, `Format`, `Search All Libraries`, `Clear`).
   - Split view: left column book cards with formats & synopsis snippet; right column **Book Inspector** with live `cover.jpg` texture rendering (`egui::Image`), `Read Excerpt` paginated reader (`full-text-search.db:books_text`), `Speak Synopsis (speech-mcp)`, `Ask Gemma 4`, and disk format file sizes.
3. **Authors**:
   - Full A-Z initial-letter filter bar + name search text box.
   - Author cards displaying total book volume counts.
   - 1-click `Browse Books` button switching immediately to Books view with author pre-filtered.
4. **Series**:
   - Series name search filter.
   - Series cards with volume counts and canon coverage status (`· Complete` in emerald or missing volume indices in amber).
   - 1-click `Browse Series Books` button switching immediately to Books view.
5. **Tags**:
   - Tags filter search box.
   - Responsive wrapped badge pills displaying tag name and book counts.
   - Clicking any tag pill navigates to Books view with the tag applied.
6. **Publishers**:
   - Top publishing houses ranked by book volume from reading statistics.
   - 1-click `Search Books` button navigating to Books view.

### Group 2: DISCOVER
7. **Full-Text Search**:
   - Direct text search inside book bodies via `full-text-search.db` (`books_text` + `books_fts` FTS5) with snippet extraction and one-click jump into Book Inspector.
8. **Reading & Priorities**:
   - Hero KPIs: Collection size, read volumes, and unread queue size.
   - Publication decade bar chart (`calibre.reading_stats`).
   - Unread Priority Queue (`calibre.unread_priority`) with priority scores and scoring reasons, plus instant `Inspect Book` button.
9. **Duplicates**:
   - Scans normalized titles and authors for duplicate groups.
   - Multi-format breakdown (EPUB, PDF, AZW3, MOBI) for each duplicate entry.
   - Direct 1-click inspection for individual duplicate volumes.

### Group 3: MANAGE
10. **Library Health**:
    - Hero KPIs (`Total Books`, `Cover Coverage %`, `Synopsis Coverage %`, `Duplicate Title Groups`).
    - Integrity breakdowns: Missing cover counts, missing comment/synopsis counts, and untagged volume counts.
    - SQLite contention safety guarantees note.

### Group 4: AI & FLEET
11. **AI Librarian**:
    - Local Ollama (`gemma4:12b` default), LM Studio, or Cloud LLM chat interface.
    - Automatic injection of active library context (book titles, authors, formats).
    - Model picker dynamically populated from available models.
12. **Crossconnect**:
    - `speech-mcp` (`:10909`) voice synthesis for aloud reading of synopses.
    - `aiwatcher` (`:10946`) heartbeat and event telemetry dispatch.

### Group 5: SYSTEM
13. **System Logs**:
    - Real-time timestamped engine and desktop event logs.
    - Case-insensitive live text filter box.
    - 1-click log clear and reset action.
14. **Settings & Vault (Including Fleet Standard LLM Card)**:
    - **Calibre Library Registry**: Pin active libraries to `%LOCALAPPDATA%\rust-calibre-mcp\libraries.json`, add external directory paths, rescan discovered libraries.
    - **Fleet Standard LLM Settings Card**:
      - Provider selection: `Ollama (Local :11434)`, `LM Studio (Local :1234)`, `OpenAI (Cloud)`, `Anthropic (Cloud)`, `Custom`.
      - Editable Base URL and model presets.
      - Windows Credential Manager OS vaulting via `keyring` (API keys never touch disk).
      - Live dynamic model elicitation button (queries `/api/tags` or `/v1/models` into a dropdown picker).
      - Live non-streaming connectivity test button with latency status pill.
    - **Windows Credential Manager Vault (Calibre Server)**: Store/clear Content Server password.
    - **Fleet Crossconnect Endpoints**: Configure TTS and Webhook URLs with ping buttons.
    - **Display Preferences**: UI zoom reset, theme toggle (Harsh Dark / Crisp Light), and runtime architecture info.
15. **Help**:
    - Keyboard and mouse navigation shortcuts (`Ctrl+Wheel` zoom), theme switching guide, and SQLite read-only safety guarantees.
16. **Raw JSON**:
    - Monospace payload inspector displaying the raw output of the most recent L1 backend command.
