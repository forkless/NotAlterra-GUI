NotAlterra is an unofficial Subnautica 2 save-file manager.
Cross-platform terminal application.  No admin permissions or network access required.

Pre-compiled binaries — no installation, no dependencies.  Just download,
extract, and run.

---

### v0.4.2 — Pinned headers & layout polish

**UI**
• Restore backup picker: pinned column headers, wider Backup column, right-aligned Size
• INI restore picker: matching pinned header with "INI Backup" / "Size" columns  
• All picker highlight colour changed to yellow for better visibility
• Main menu entries shifted 1 left; INI submenu entries aligned with main menu
• Spacer row between list content and status bar on all screens
• Non-blocking info dialog for backup-in-progress (no button, auto-replaced)

**Layout**
• Backup column widened from 30→38 chars; header/data total widths matched
• Size header right-aligned above file-size values in both restore pickers
• INI submenu header changed to cyan, shifted 3 right

**Backup flow**
• No intermediate spinner page — info popup followed by summary dialog

---

### v0.4.1 — Persistent config & split-layout picker

**UI**
• Split-layout backup picker — file list on left, live GVAS metadata on right
• `►` pip highlight replaces background bar on all menus
• Blank separator lines for visual grouping in menus
• "Set Subnautica 2 location" and "Set backup location" menu entries
• Context-aware header path — title bar shows the relevant file location for the current menu/screen

**Configuration**
• Persistent `app.ini` — save folder and backup location survive sessions
• Config stored in platform-standard directory (`AppData/Local` on Windows)
• UE5 `.ini` backups stored in `backups/ue5/` under the backup root
• Logs moved to `data_local_dir/NotAlterra/logs/`

**Security & Documentation**
• Security disclosure pipeline: SECURITY.md safe harbor, CVE template, bug report template, PR template
• GitHub Private Vulnerability Reporting enabled
• All docs updated to accurately describe data persistence

**Maintenance**
• Zero clippy warnings, zero fmt issues
• 24/24 tests passing

---

### What's new in v0.4.0

• tar.gz backup format — one archive per backup event
• `--help` / `-h` flag
• Automatic migration from old `NotAlterra_Backups/` directory-tree format
• Stale `config.ini` auto-removed from prior versions

Builds:  Linux (amd64)  •  Windows x64
