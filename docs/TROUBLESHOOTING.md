# Troubleshooting & FAQ — `rust-calibre-mcp`

## Common Questions & Issues

### 1. "No libraries discovered on startup"
- **Cause**: Calibre is installed in a non-standard location and no `CALIBRE_LIBRARY_PATH` is set.
- **Solution**: Set the environment variable `CALIBRE_LIBRARY_PATH` to the folder containing your `metadata.db`, or pass `--library "C:\path\to\library"` to the CLI.

### 2. "Database is locked" / SQLite concurrency
- **Behavior**: `rust-calibre-mcp` opens all database connections with `SQLITE_OPEN_READ_ONLY` and a 5-second `busy_timeout`. It will wait for any concurrent Calibre GUI transactions to finish without failing immediately or locking the database.

### 3. Windows SmartScreen warning on installer
- **Cause**: `Calibre-Rust-0.4.0-setup.exe` is self-signed/unsigned.
- **Solution**: Click **More info** → **Run anyway**. All source code is open and audited with reproducible build checksums in `SHA256SUMS.txt`.

### 4. Full-text search returns zero hits
- **Cause**: Calibre creates `full-text-search.db` only when full-text indexing is enabled in Calibre GUI preferences.
- **Solution**: Verify that `full-text-search.db` exists in your Calibre library directory. If not present, standard keyword metadata searches via `calibre.query_books` continue to work normally.
