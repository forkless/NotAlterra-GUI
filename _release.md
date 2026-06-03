NotAlterra is an unofficial Subnautica 2 save-file manager.
Cross-platform terminal application.  No admin permissions or network access required.

Pre-compiled binaries — no installation, no dependencies.  Just download,
extract, and run.

### What's new in v0.4.1

• Log migration — existing `transaction.log` is moved into `logs/` on first launch
• Backup directory structure scaffolded at startup (`backups/saves/`, `backups/config/`, `logs/`)
• `ensure_dir()` helper for consistent directory creation
• All migration paths integrated into startup (config.ini, backups, log)

### What's new in v0.4.0

• tar.gz backup format — one archive per backup event, standard `tar -xzf` recovers data without the tool
• File restructure: `backups/saves/`, `backups/config/`, `logs/`
• Automatic migration from old `NotAlterra_Backups/` directory-tree format
• `--help` / `-h` flag
• Stale `config.ini` auto-removed from prior versions
• Bus factor mitigation plan documented in GOVERNANCE.md
• 6 migration tests, full round-trip tar.gz integration tests
• `tar` + `flate2` dependencies added (pure Rust, ~150KB increase)

Builds:  Linux (amd64)  •  Windows x64
