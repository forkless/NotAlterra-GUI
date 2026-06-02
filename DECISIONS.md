# Design Decisions

This file captures the rationale behind significant architecture and format
choices, so the reasoning is preserved for future maintainers (including
yourself six months from now).

---

## Sentinel File vs config.ini (v0.3.2)

### Problem
`config.ini` persisted the save folder path to disk, including the user's
filesystem-username. This is a privacy concern — paths are visible next to
the binary.

### Decision
Remove `config.ini` entirely. The save folder is session-only — set it each
time via **Set save folder**. The disclaimer acceptance is tracked via a
0-byte sentinel file (`NotAlterra_LICENSE_ACCEPTED`) alongside the binary.

### Rationale

**Privacy** — no paths written to disk. The save folder exists only in
memory while the tool runs.

**Simplicity** — no config parsing, no INI format to maintain, no migration
code for renamed keys.

**Sentinel, not config** — a 0-byte file communicates exactly one boolean
(disclaimer accepted). It cannot grow into a configuration file over time.
The format intentionally prevents scope creep.

**What was removed:**
- `AppConfig` struct (save_path, ini_path, save_scan, disclaimer_accepted)
- `load_config()` / `save_config()` with INI parsing
- Cached `ini_path` — now derived from save folder at runtime
- Four integration tests for config round-trips

---

## Manual Path Entry vs Auto-Discovery (v0.3.0)

### Problem
Auto-discovery scanned user profiles and system directories for Subnautica 2
save folders. This is a privacy concern — it traverses `/home/*` (Linux) and
`C:\Users\*` (Windows).

### Decision
Replace full auto-discovery with manual path entry via **Set save folder**.
Keep a lightweight `quick_discover()` that checks only the current user's
default install paths at startup.

### Rationale

**Privacy** — no scanning of other users' profiles or system drives.

**Current-user convenience** — `quick_discover()` checks 1 path on Windows,
3 paths on Linux, all within the current user's own directories. Returns
the first match silently, no UI. If nothing is found, the user enters their
path manually.

**Discovery module retained** — `validate_custom_path()` and
`derive_ini_path()` still live in `discovery.rs` for the manual entry flow.
The aggressive scan functions (`discover_save_folders()`, `scan_other_users()`,
`walk_for_subnautica()`) are removed.

---

## tar.gz Backup Format (v0.4.0)

### Problem
Directory-tree backups (`NotAlterra_Backups/notalterra_copy_<timestamp>/`)
are messy, uncompressed, and have no integrity guarantees.

### Decision
One `tar.gz` archive per backup event, stored in `backups/saves/`.

### Rationale

**No vendor lock-in** — standard `tar -xzf` recovers data without the tool.
If NotAlterra stops working, the user's backups are still accessible with
standard system utilities.

**Single file per event** — reduces clutter. One backup = one file, not
a directory tree with 15+ loose save files.

**Compression** — save files compress well (~75MB → ~20MB). Reduces disk
usage without user effort.

**Pure Rust implementation** — `tar` + `flate2` crates, 200M+ downloads
combined. No system dependencies, no external tools.

**Per-entry restore** — extracting a single save file from the archive
does not require decompressing the entire archive.

### Safeguards

- **Atomic write**: backup written to `.tmp` file, then atomically renamed.
  Power loss during backup discards a temp file, not a real backup.
- **Integrity check after creation**: archive is read back and validated
  before reporting success.
- **SHA256 manifest**: a `MANIFEST` file inside each archive records the
  hash of every contained save file. On restore, each extracted file is
  verified against its expected hash — silent bit-rot detected before bad
  data reaches the save folder.
- **Fuzz target**: round-trip fuzzing (create archive from diverse inputs →
  restore → compare) catches logic bugs.

### Migration
Existing `NotAlterra_Backups/` directory-tree backups are detected and
transparently imported on first run after upgrade. No manual migration
required.
