# Changelog

All notable changes to NotAlterra are documented in this file.

---

## [v0.4.1] — 2026-06-09

### Added
- **Split-layout backup picker** — pip (`►`) highlight replaces background bar,
  right pane shows live GVAS metadata for the selected backup (loads on
  highlight, cached per file)
- **Persistent config** (`app.ini`) — save-folder and backup-root paths now
  survive sessions, stored under `data_local_dir/NotAlterra/config/`
- **Set backup location** menu entry — choose where save and UE5 backup
  archives are stored; defaults to `~/NotAlterra`
- **Blank separator lines** in menus — visual grouping, auto-skipped on
  navigation
- **`docs/CVE_TEMPLATE.md`** — structured vulnerability disclosure template
- **`docs/BUG_REPORT_TEMPLATE.md`** — user-facing bug report reference with
  privacy warnings
- **`.github/PULL_REQUEST_TEMPLATE.md`** — PR checklist matching CI gates
- **Safe harbor clause** in `SECURITY.md` — legal protection for good-faith
  security researchers

### Changed
- **File picker layout** — horizontal 60/40 split: file list on left,
  metadata preview on right. Slot, Description, Date columns only
- **Main menu pip highlight** — `►` replaces full-row cyan background
  (legacy highlight code retained for other pickers)
- **Config directory** — `app.ini` and sentinel moved from `exe_dir/` to
  `data_local_dir/NotAlterra/config/` (platform-standard)
- **Logs directory** — `transaction.log` moved from `exe_dir/logs/` to
  `data_local_dir/NotAlterra/logs/`
- **UE5 ini backups** — stored in `backups/ue5/` under the backup root
  instead of `backups/config/`
- **Menu labels** — "Set save folder" renamed to "Set Subnautica 2 location",
  descriptions updated
- **Local builds** — compiled with `cargo build` for both Linux and Windows
  targets
- **Context-aware header path** — the right side of the title bar shows
  the relevant file location for the current menu or submenu item (save
  folder, backup root, or ini Config\Windows path)
- **Human-readable backup labels** — archive filenames are now displayed
  as `Full Backup — <date>` and `Pre-restore — <date>` in the restore
  picker instead of raw `.tar.gz` filenames
- **Backup Types section** in README — explains Full Backup vs
  Pre-restore in plain language so users understand the safety flow
- **Archive integrity check** — `check_tar_gz_integrity()` verifies gzip
  magic bytes before restore; corrupt files show a warning description
  and Enter is blocked
- **Context-aware restore descriptions** — restore picker shows
  different descriptions for full backup, pre-restore, migrated, and
  corrupted archives
- **Consistent warning spacing** — all `⚠` messages use double-space
  formatting

### Removed
- **"Inspect save files" menu entry** — metadata is now visible inline in
  the backup picker right pane. `action_inspect_saves()` code retained.
- **`i` key handler** in backup picker — redundant with live metadata pane
- **Outdated privacy claims** — docs no longer state "no data stored" or
  "session-only"; now accurately describe `app.ini` persistence

### Security
- **GitHub Private Vulnerability Reporting** as primary disclosure channel,
  email as fallback
- **Safe harbor** — explicit no-litigation commitment for good-faith
  reporters following the disclosure policy
- **CVSS v4 scoring** documented in Dependabot advisory for `lru`
  transitive dependency (low severity, not actionable)
- All paths in `app.ini` documented as potentially containing the system
  username — plain text, never transmitted

## [v0.4.0] — 2026-06-03

### Added
- **`--help` / `-h` flag** — displays usage information
- **tar.gz backup format** — one archive per backup event, all slots in one
  file. Standard `tar -xzf` recovers data without the tool (no vendor lock-in).
  Safeguards: atomic write (`.tmp` → rename), per-file SHA256 manifest,
  per-entry restore without full decompress.
- **Migration path** — old `NotAlterra_Backups/` directory-tree backups are
  automatically detected and imported into the new tar.gz format
- **Bus factor mitigation** — documented in GOVERNANCE.md: emergency signing
  key stored with a non-technical trusted person, revocable if compromised

### Changed
- **File layout** — backups stored in `backups/saves/` (tar.gz),
  `backups/config/` (.ini archives), logs in `logs/transaction.log`
- **Stale `config.ini` removed** on first launch from prior versions
- **Dependencies** — added `tar` + `flate2` (pure Rust, +~150KB binary)

### Testing
- 6 migration unit tests (basic, empty, nonexistent, integrity, filtering, idempotency)
- Full round-trip integration tests for tar.gz backup/restore
- Fuzz target for backup round-trip (10s: 11,717 runs, zero crashes)
- File permissions fixed in tar headers (`0o644` instead of `0o000`)

## [v0.3.2] — 2026-06-02

### Added
- **Lightweight startup check** — on launch, the app silently checks the
  current user's default save locations (`%LOCALAPPDATA%` on Windows,
  Proton + XDG data on Linux). No scanning of other profiles or system
  drives. If nothing is found, use **Set save folder** as before.
- **Migration notification** — old backups in `NotAlterra_Backups/` are
  migrated silently on first launch with a log entry. The user is informed
  their original data remains untouched.
- **Log migration** — existing `transaction.log` is moved into `logs/`
  on first launch, appended to the new location.

### Removed
- **`config.ini` eliminated entirely** — no save path, disclaimer flag, or
  scan timestamp is written to disk anymore. The save folder is session-only,
  entered via **Set save folder**. The disclaimer acceptance is tracked via a
  0-byte sentinel file (`NotAlterra_LICENSE_ACCEPTED`) alongside the binary.
  - `src/config.rs` reduced to sentinel utilities and `exe_dir()`
  - `AppConfig`, `load_config()`, `save_config()` removed
  - Integration tests for config round-trips removed (replaced by sentinel test)

### Changed
- `get_ini_path()` now derives the Config/Windows path from the save folder
  at runtime — no cached `ini_path` in memory or on disk.

## [v0.3.1] — 2026-06-02

### Added
- **Patrolling whale** in the Set save folder input dialog (was missing
  the bottom separator present on all other screens)

### Fixed
- **Path injection via paste** — control characters (newlines, tabs, null
  bytes) are now stripped from user-provided paths before they reach
  `config.ini` or `transaction.log`
- **Draft releases** — CI now creates releases as drafts. You can download
  and test the binaries before publishing them live.

### Removed
- **`Locate save files` menu item** — the deprecated auto-discovery entry
  point is gone. `Set save folder` is the only path for configuring the
  save location. The `discovery.rs` module remains for `validate_custom_path`
  and `derive_ini_path` but is marked for removal in v0.4.0.

## [v0.3.0] — 2026-06-02

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
  in v0.4.0 — use `Set save folder` instead.

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
- **Zero clippy warnings on CI** — fixed `collapsible_match` (6 instances),
  `empty_line_after_doc_comments`, and `manual_is_multiple_of` lints from
  rustc 1.95 nightly.
- **`KNOWN_ISSUES.md` moved from `docs/` to project root** — reflects v0.3.0
  privacy improvements (manual path entry live, discovery deprecated).

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
