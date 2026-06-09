NotAlterra is an unofficial Subnautica 2 save-file manager.
Cross-platform terminal application.  No admin permissions or network access required.

Pre-compiled binaries — no installation, no dependencies.  Just download,
extract, and run.

### What's new in v0.4.1

• Persistent `app.ini` config — save folder and backup location survive sessions
• Split-layout backup picker with inline GVAS metadata preview
• `►` pip highlight replaces background bar on all menus
• Set backup location menu entry (defaults to `~/NotAlterra/`)
• Security disclosure pipeline: SECURITY.md safe harbor, CVE template, bug report template, PR template
• Config moved to platform-standard directory (`AppData/Local` on Windows)
• UE5 ini backups stored in `backups/ue5/` under the backup root

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
