# Known Issues

## config.ini persists the save path

`config.ini` caches the save-folder path on disk next to the binary. Paths are
sanitized in transaction logs, but the raw path remains in the config file for
re-use across sessions.

The recommended workflow now uses **Set save folder** from the main menu to
enter paths manually. Delete `config.ini` to clear the cached path — the app
will prompt you to set a new one on next launch.

**Planned**: Remove `config.ini` entirely in a future release. The path would
be re-requested from the user on each session. v0.4.0 target.

## Discovery module is deprecated

Auto-scan for save folders (`discovery.rs`) is deprecated in favor of the
**Set save folder** menu option. The `Locate save files` item shows a
deprecation notice once per session. Scanning user profiles and system
directories is a privacy concern and this module is scheduled for removal.

**Planned**: Remove `discovery.rs` entirely in v0.4.0. Users will enter their
save path manually via `Set save folder`.
