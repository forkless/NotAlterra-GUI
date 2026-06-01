# Changelog

All notable changes to NotAlterra are documented in this file.

---

## [v0.2.0] — 2026-06-01

### Added
- GitHub Actions CI pipeline — builds, packages, and uploads on signed tag push
- SLSA L3 provenance generation for build artifacts
- Fully automated release pipeline — `./build.sh` handles build, package, sign, push, and CI release
- Column headers in file pickers (Slot, Description, Game Type, Playtime, Size, Date)
- Navigation prompt alongside item description line
- Unsigned executable notice in README

### Changed
- `last_path` → `save_path`, `last_scan` → `save_scan`, `config_path` → `ini_path`
- Windows executable renamed to `NotAlterra.exe`
- Removed unused dependencies (sysinfo, log, simplelog)
- Playtime zero-padded to 2 digits (`03h 05m`)
- Header and blank rows non-navigable in pickers

### Fixed
- Playtime extraction on production saves (fallback byte-scan for DoubleProperty)
- File picker column alignment and spacing tightened

## [v0.1.3] — 2026-06-01

### Added
- Column headers in file pickers (Slot, Description, Game Type, Playtime, Size, Date)
- Navigation prompt moved to same line as item description

### Changed
- "Multiplayer" / "Single Player" used consistently throughout picker and inspector
- Release archives moved to `builds/` directory
- Playtime zero-padded to 2 digits (03h 05m)
- Header and blank rows non-navigable in pickers

### Fixed
- Playtime extraction on production saves (fallback byte-scan for DoubleProperty)
- File picker column alignment and spacing tightened

## [v0.1.2] — 2026-06-01

### Added
- Playtime extraction from GVAS `ElapsedTimeDouble` property — displays as `Xh Ym`
  in file picker and inspector
- Filesystem checks for .ini delete guard — requires actual `.ini` files in backup
- Non-blocking event poll across all screens

### Changed
- Replaced process detection (`tasklist`/`pgrep`) with startup warning modal
  to avoid Windows Defender false-positive (Trojan:Win32/Wacatac.C!ml)
- "Online" / "Local" renamed to "Multiplayer" / "Single Player" throughout
- Date format changed to `YYYY-Mon-DD HH:MM`
- Status bar text removed — whale-only row
- "Back" option added to .ini submenu
- All save/ini operations now show file counts

### Fixed
- .ini backup now shows OK confirmation dialog
- .ini restore returns file count
- Fullbackup restore returns file count
- Dashboard stats filter by `savegame_*` prefix
- OK dialog width padding prevents text clipping
- File picker column alignment tightened

## [v0.1.1] — 2026-05-31

### Changed
- "Recover .sav file from .bak" renamed to "Recover save file" (less technical)
- Dashboard stats now filter by `savegame_*` prefix, matching the file picker
- Save/Backup labels pluralize based on count

### Added
- 🐋 animated leviathan patrols the status bar
- Non-blocking event poll across all screens — animates everywhere
- Whale rendered on popups, dialogs, and file pickers
- "Back" option in .ini management submenu

### Fixed
- Game-running guard shows informative OK dialog instead of silent exit
- OK dialog width padding prevents text clipping on long lines

## [v0.1.0] — 2026-05-31

### Added
- Cross-platform terminal UI (ratatui + crossterm) with keyboard-driven menus, pickers, popups, and status bar
- GVAS save-file binary parser: extracts SlotName, DisplayName, bIsMultiplayerSave, GameMode, LevelName, BuildNumber, and more
- **Save-folder discovery** with Windows fast-path (`%LOCALAPPDATA%`) and Linux Proton/Steam Deck paths
- **.bak to .sav recovery** with filename-derived slot grouping, `.sav.old` rollback, and size sanity checks
- **Full backup / restore** (only `savegame_*` files) with pre-restore snapshots and verification
- **UE5 Config (.ini) management** — backup, restore, and guarded delete
- **Save inspector** with full GVAS metadata displayed in a color-coded popup
- **Slot grouping** in the recovery picker — first entry per slot gets a numbered label, subsequent entries blank
- **Online / Local detection** via `bIsMultiplayerSave` BoolProperty
- **Mode-change warning** (Online ↔ Local) and **name-change warning** on recovery
- **Game-running guard** — exits or warns if Subnautica 2 is active
- **Transaction log** (`transaction.log`) with `MANUAL_BAK`, `AUTO_BAK`, `RESTORE`, `RECOVER`, `CONFIG_BAK`, `CONFIG_RESTORE`, `CONFIG_DEL`, and `LICENSE` actions
- **Disclaimer popup** with Accept / Decline buttons — Esc returns to menu without revoking
- **OK dialogs** for backup results, no-backup warnings, and ini action outcomes
- **Confirmation popups** with `[ Yes ]` / `[ No ]` buttons and detailed metadata comparison
- **No-backup guard** — warns if `NotAlterra_Backups` is empty before destructive actions
- **Empty directory cleanup** — failed backup directories are removed
- **Auto-size popups** — GVAS inspector and OK dialogs scale to fit content
- **Terminal resize** — main loop and all sub-loops respond to terminal size changes
- **Arrow-key debounce** — `KeyRelease` events are filtered; no double-stepping
- **Elapsed timer** during save-folder scan with background threading
- **Release build script** (`build.sh`) producing `notalterra-v{version}-linux-amd64.tar.gz` and `notalterra-v{version}-windows-amd64.zip`
- `--version` / `-v` CLI flag
- MIT License and README with build, usage, and unpack instructions

### Changed
- Improved game-running exit message — explains why save files are at risk

### Fixed
- Background scan thread panic no longer hangs the application
- Arrow-key navigation no longer overshoots — `KeyEventKind::Release` events filtered
- GVAS metadata popup renders all 11 fields (was clipped to 2 because of `Paragraph::new(Span)`)
- Header path now shows tail of path (e.g. `…\Subnautica2\Saved\SaveGames`) instead of truncated prefix
- Confirmation popup always appears — hard `require_backup` block replaced with soft warning

### Security
- Game-running detection at launch and before each destructive operation
- `.ini` delete guarded by requiring at least one `ini_backup_*` directory
