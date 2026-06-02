# Changelog

All notable changes to NotAlterra are documented in this file.

---

## [v0.2.4] — 2026-06-01

### Added
- SECURITY.md with vulnerability disclosure policy
- Release checklist in GOVERNANCE.md
- 32-test integration suite (guard, config, ops, gvas, ini backup/restore)
- Fuzz target for GVAS parser (`parse_gvas`)
- Second fuzz target (`full_metadata`) — exercises IntProperty, DoubleProperty,
  and additional StrProperty/BoolProperty code paths
- `fuzz/Cargo.toml` manifest with both fuzz targets registered
- **`Set save folder` menu option** — manual path entry with clipboard paste
  support (bracketed paste mode), replaces auto-discovery as the primary way to
  set the save location
- cargo clippy, cargo audit, and cargo-deny in CI
- deny(unsafe_code) in library crate
- Build script validates CHANGELOG has current version entry

### Fixed
- Index-out-of-bounds panic in all four GVAS property extractors
  (`extract_str_property`, `extract_bool_property`, `extract_int_property`,
  `extract_double_property`) when a property name appeared too close to the
  end of the buffer — discovered by fuzzing the existing `parse_gvas` target

### Deprecated
- **Auto-scan for save folders** (`Locate save files` menu item / `discovery.rs`
  module). Scans user profiles and system directories, which is a privacy
  concern. Shows a deprecation notice once per session. Scheduled for removal
  in v0.3.0 — use `Set save folder` instead.

### Changed
- Zero compiler warnings
- Example dump_samples compiles and runs
- `fuzz/target/` added to `.gitignore`
- Fuzz targets rewritten from `#[fuzz]` attribute to `libfuzzer_sys::fuzz_target!`
  macro for nightly-toolchain compatibility
- **No auto-scan on startup** — the application no longer scans user profiles
  and system drives for save folders at launch. Only the cached path from
  `config.ini` is loaded.
- **Menu is always 9 items** — `Set save folder` is always visible. `Locate
  save files` is always visible (with deprecation label). No conditional hiding
  or index remapping.
- **`ensure_save_folder()` and `get_ini_path()`** no longer fall back to
  `discover_save_folders()`. They use the cached path or error with a message.
- **`is_cloud_path()` removed** — was only used by the discovery-era cloud
  detection path.

### Notes
- Working copy diverged from remote after signing the previous commit
  on a different clone. Resolved via `git reset --soft origin/master`
  (no content lost, identical tree).

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
