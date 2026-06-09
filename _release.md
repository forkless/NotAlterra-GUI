NotAlterra is an unofficial Subnautica 2 save-file manager.
Cross-platform terminal application.  No admin permissions or network access required.

Pre-compiled binaries — no installation, no dependencies.  Just download,
extract, and run.

---

### v0.4.3 — Dashboard & backup integrity fixes

**Fixed**
• Dashboard counts now correct: "Saves" shows `.bak` recovery file count, "Backups" shows `.tar.gz` archive count
• Backup archives now preserve source file modification times — restored files keep their original dates instead of showing 1970-01-01 on Windows
• Restore backup picker navigation no longer stops before migrated entries
• Save folder input dialog pre-fills with the currently set path
• SLSA provenance attestation (automatic for tagged releases)

Builds:  Linux (amd64)  •  Windows x64

For a full history of changes across all versions, see
[`CHANGELOG.md`](https://github.com/forkless/NotAlterra/blob/master/CHANGELOG.md).
