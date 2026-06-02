NotAlterra is an unofficial Subnautica 2 save-file manager.
Cross-platform terminal application.  No admin permissions or network access required.

Pre-compiled binaries — no installation, no dependencies.  Just download,
extract, and run.

### v0.3.2

• `config.ini` removed entirely — no paths written to disk
• Save folder is session-only, entered via `Set save folder`
• Disclaimer tracked via 0-byte sentinel file (`NotAlterra_LICENSE_ACCEPTED`)
• Sentinel and path utilities live in reduced `config.rs`

### v0.3.1

• Patrolling whale added to Set save folder input dialog
• Path injection sanitized — control characters stripped before writing to config or log
• Deprecated `Locate save files` menu item removed
• `Set save folder` is the sole method for configuring the save location

### v0.3.0

• `Set save folder` — manual path entry with clipboard paste support
• No auto-scan on startup (privacy: scanning user profiles is disabled)
• Fuzz testing for GVAS parser (2 targets, ~450k runs clean)
• SLSA v3 provenance attestation on all release artifacts
• GPG-signed release tags
• Index-out-of-bounds fix in GVAS property extractors (found by fuzzing)
• Auto-discovery deprecated — scheduled for removal in v0.4.0

_Builds:  Linux (amd64)  •  Windows x64_
