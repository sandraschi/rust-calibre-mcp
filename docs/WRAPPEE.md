# About Calibre (Host Application)

`rust-calibre-mcp` is an independent, pure-Rust MCP interface and native desktop companion for **Calibre**, the open-source e-book manager.

## Official Links & Resources

- **Official Website**: [calibre-ebook.com](https://calibre-ebook.com/)
- **Source Repository**: [github.com/kovidgoyal/calibre](https://github.com/kovidgoyal/calibre)
- **User Manual & Documentation**: [manual.calibre-ebook.com](https://manual.calibre-ebook.com/)
- **User Community & Support**: [MobileRead Calibre Forum](https://www.mobileread.com/forums/forumdisplay.php?f=166)

## Relationship & Data Safety

- `rust-calibre-mcp` reads Calibre's native SQLite databases (`metadata.db` and `full-text-search.db`) directly.
- All connections are opened in strict **read-only mode** (`SQLITE_OPEN_READ_ONLY`) with a 5000 ms busy timeout, ensuring zero database corruption or lock contention even while the official Calibre desktop GUI is actively running.
- Calibre itself is developed and maintained independently by Kovid Goyal and contributors.
