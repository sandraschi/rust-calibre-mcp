# `calibre-app` (L3 Native `egui 0.37` ArtCraft Desktop App)

`calibre-app` is the native `egui 0.37` (`eframe 0.37`) desktop frontend for `rust-calibre-mcp`. It calls `calibre_core::dispatch` (`L1`) directly in-process via a dedicated background `tokio` worker thread, requiring zero web servers, zero background daemons, and zero open TCP ports.

## Key Features

- **9 Pages**: Libraries, Books & Inspector (with live `cover.jpg` texture rendering, paginated `full-text-search.db` excerpt reader, and federated multi-library search), Full-Text Search, Health & Series + Reading Statistics + Unread Priority Queue, AI Librarian (`gemma4:12b`), Crossconnect (`speech-mcp` & `aiwatcher`), Settings & Windows Credential Manager Vault (`keyring`), Help, and Raw JSON.
- **Custom `egui::Painter` Sun/Moon Theme Toggle** (Harsh Dark & Crisp Light palettes).
- **Safe `Ctrl+MouseWheel` Zoom** (`50%` to `200%`).
- **Headless CI Smoke Gate**: `cargo run -p calibre-app -- --smoke`.
