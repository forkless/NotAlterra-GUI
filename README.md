# NotAlterra

**Subnautica 2 Save Manager — Windows Native GUI**

![NotAlterra screenshot](docs/screenshot.jpg)

NotAlterra is an unofficial tool for managing Subnautica 2 save files. It locates your save folders, backs up and restores `.sav` / `.bak` files, and lets you inspect the internal metadata of every save.

Not affiliated with Unknown Worlds Entertainment or KRAFTON.

## Features

- **Manual path entry** — set your save folder from the menu (paste supported)
- **Recover** a corrupted `.sav` from its `.bak` backup with rollback
- **Create / restore** full backups (only `savegame_*` files)
- **Manage** UE5 Config `.ini` files — backup, restore, delete
- **Inspect** any `.sav` or `.bak` file — full GVAS metadata dump
- **Slot grouping** with Multiplayer / Single Player detection
- **Playtime extraction** — displays total playtime from save metadata
- **Warns** on name changes and mode switches before recovery
- **Transaction log** — all actions timestamped to `transaction.log`

### ⚠ Important

**NotAlterra is a read-only metadata inspector and backup tool.** It does not edit `.sav` files in-place. Recovery operations restore a `.bak` copy to `.sav` (identical to a manual file copy). No save data is ever patched, rewritten, or modified byte-by-byte.

**Metadata extraction uses a heuristic byte-scan** — it searches for known property names (`DisplayName`, `GameMode`, `PlaytimeSeconds`, etc.) as raw byte patterns in the binary GVAS data. This is NOT a full UE5 GVAS structure walker. Some fields may be absent or misidentified if the save format changes in a game update.

## Installation

Download the latest installer from the [releases page](https://github.com/forkless/NotAlterra/releases/latest).

| Installer | Size | Dependencies |
|-----------|------|--------------|
| NotAlterra-*-x64.exe | ~19 MB | .NET 9 runtime + Windows App SDK 1.8 |

The installer will check for the Windows App SDK 1.8 runtime. If missing, it prompts to download and install it (one-time setup, ~60 MB). .NET 9 runtime must be installed separately. Run the installer and follow the prompts. A desktop shortcut is optional.
## Where Files Live

```
%LOCALAPPDATA%\Subnautica2\Saved\SaveGames\
    savegame_0.sav
    savegame_0.bak
    savegame_0_1.bak
    savegame_1.sav
    ...
```

## Privacy

**NotAlterra makes no network connections. No telemetry. No data leaves your machine.**

- The application never calls home — there are no update checks, no analytics, no crash reporters.
- It runs entirely offline. Your save files and their metadata stay on your machine.
- The Windows App SDK runtime has opt-in telemetry which NotAlterra does not enable.
- Configuration (`app.ini`) stores your save-folder and backup paths locally. These may reveal your system username; the file is plain text you can inspect or delete at any time.

If network features are added in a future version, they will require explicit user opt-in.

## Safety

- Runs in your user context — no admin privileges required
- No network connections
- Read-only inspect mode won't touch files
- Pre-restore snapshots created automatically
- .ini delete requires a prior backup

## Backup Format

Backups are stored as `tar.gz` archives: one archive per backup event, containing all `savegame_*` files. Standard `tar -xzf` recovers data without the tool (no vendor lock-in). Each archive contains a SHA256 manifest for integrity verification.

## Credits

- **Sidebar background video** by Pachon in Motion — [@Pachon.In.Motion](https://instagram.com/Pachon.In.Motion), Provincia de Buenos Aires, Argentina

## License

MIT — see `LICENSE.md`.
