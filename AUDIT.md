# Security Audit

**Project:** NotAlterra (Subnautica 2 save-file manager)
**Date:** 2026-06-01
**Scope:** Full source code — `src/`, `examples/`, build pipeline

## Summary

No malicious or exploitable behavior detected.  NotAlterra is an offline
terminal application with no network access, no unsafe code, and
no data exfiltration surface.

## Audit Results

| Category | Finding |
|---|---|
| `unsafe` blocks | None |
| Network (sockets, HTTP, TLS) | None — zero network dependencies |
| Process spawning | `guard.rs` (tasklist / pgrep) — dormant, not called |
| Dynamic loading | None |
| `include_bytes!` / obfuscation | None |
| Thread spawning | Whale animation only — benign |

## Process Guard

`guard.rs` contains code to detect a running Subnautica 2 instance via
`tasklist` (Windows) and `pgrep` (Linux).  This code is **dormant** —
`check_game_not_running()` is never called from `main.rs`.

The intent is to prevent accidental save corruption by warning the user
if the game is running during backup or recovery operations.  This
feature will be re-enabled once the project completes SignPath
certification.

## File I/O

All file writes are confined to declared paths:

- `config.ini` — save-path cache, scan timestamp, disclaimer flag
- `NotAlterra_Backups/` — backup archives
- `transaction.log` — timestamped action log

No writes outside these directories.  No reads beyond the Subnautica 2
Saved folder tree.

## Dependencies

No dependency introduces network access or code execution risks.  Full
dependency tree is pinned via `Cargo.lock`.

## Conclusion

NotAlterra is safe to use.  It operates entirely within the user's
local filesystem and performs only the save-management operations it
declares.
