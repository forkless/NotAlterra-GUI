# Changelog

All notable changes to NotAlterra are documented in this file.

---

## [v0.2.4] — 2026-06-01

### Added
- SECURITY.md with vulnerability disclosure policy
- Release checklist in GOVERNANCE.md
- 32-test integration suite (guard, config, ops, gvas, ini backup/restore)
- Fuzz target for GVAS parser
- cargo clippy, cargo audit, and cargo-deny in CI
- deny(unsafe_code) in library crate
- Build script validates CHANGELOG has current version entry

### Changed
- Zero compiler warnings
- Example dump_samples compiles and runs

## [v0.2.3] — 2026-06-01

- Zero compiler warnings (`deny(unsafe_code)`, `allow(dead_code)` where intentional)
- Example `dump_samples` compiles and runs
- 100% function doc coverage with automated check script
- `last_path` → `save_path`, `last_scan` → `save_scan`, `config_path` → `ini_path`
- Windows executable renamed to `NotAlterra.exe`
- Diligence skill added for verification

### Security
- Logged paths truncated at `Subnautica2/` — personal paths removed

## [v0.2.0] — 2026-06-01

- Zero compiler warnings (`deny(unsafe_code)`, `allow(dead_code)` where intentional)
- Example `dump_samples` compiles and runs
- `last_path` → `save_path`, `last_scan` → `save_scan`, `config_path` → `ini_path`
- Windows executable renamed to `NotAlterra.exe`
- Removed unused dependencies (sysinfo, log, simplelog)
- Playtime zero-padded to 2 digits (`03h 05m`)
- Header and blank rows non-navigable in pickers

### Fixed
- Playtime extraction on production saves (fallback byte-scan for DoubleProperty)
- File picker column alignment and spacing tightened

## [v0.1.3] — 2026-06-01

- Zero compiler warnings (`deny(unsafe_code)`, `allow(dead_code)` where intentional)
- Example `dump_samples` compiles and runs
- "Multiplayer" / "Single Player" used consistently throughout picker and inspector
- Release archives moved to `builds/` directory
- Playtime zero-padded to 2 digits (03h 05m)
- Header and blank rows non-navigable in pickers

### Fixed
- Playtime extraction on production saves (fallback byte-scan for DoubleProperty)
- File picker column alignment and spacing tightened

## [v0.1.2] — 2026-06-01

- Zero compiler warnings (`deny(unsafe_code)`, `allow(dead_code)` where intentional)
- Example `dump_samples` compiles and runs
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
- Zero compiler warnings (`deny(unsafe_code)`, `allow(dead_code)` where intentional)
- Example `dump_samples` compiles and runs
- "Recover .sav file from .bak" renamed to "Recover save file" (less technical)
- Dashboard stats now filter by `savegame_*` prefix, matching the file picker
- Save/Backup labels pluralize based on count

- Zero compiler warnings (`deny(unsafe_code)`, `allow(dead_code)` where intentional)
- Example `dump_samples` compiles and runs
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
