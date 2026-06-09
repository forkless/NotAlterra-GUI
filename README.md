# NotAlterra

**Subnautica 2 Save Manager**

NotAlterra is an unofficial tool for managing Subnautica 2 save files. It locates your save folders, backs up and restores `.sav` / `.bak` files, and lets you inspect the internal metadata of every save.

Not affiliated with Unknown Worlds Entertainment or KRAFTON.


## Features

- **Manual path entry** — set your save folder from the menu (paste supported)
- **Recover** a corrupted `.sav` from its `.bak` backup with rollback
- **Create / restore** full backups (only `savegame_*` files)
- **Manage** UE5 Config `.ini` files — backup, restore, delete
- **Inspect** any `.sav` or `.bak` file — full GVAS metadata dump
- **Slot grouping** with Multiplayer / Single Player detection
- **Playtime extraction** — displays total playtime from save metadata (read-only — not stored, not tracked; helps the user identify which savegame backup they are dealing with)
- **Warns** on name changes and mode switches before recovery
- **Startup reminder** to close Subnautica 2 before use
- **Transaction log** — all actions timestamped to `transaction.log`
- **Cross-platform** — Linux and Windows console builds
- [**Developer Documentation**](https://forkless.github.io/NotAlterra/notalterra/) — auto-generated from source


## How to Build

```bash
# Linux
cargo build --release

# Windows (from Linux with mingw-w64)
rustup target add x86_64-pc-windows-gnu
sudo apt install mingw-w64
cargo build --release --target x86_64-pc-windows-gnu
```


## How to Run

Download the pre-compiled executable for your platform from the [releases page](https://github.com/forkless/NotAlterra/releases).

### Linux

```bash
tar -xzf notalterra-v0.1.3-linux-amd64.tar.gz
chmod +x notalterra
./notalterra
```

### Windows

Extract the `.zip` archive and double-click `NotAlterra.exe`, or run from a terminal:

```powershell
Expand-Archive notalterra-v0.1.3-windows-x64.zip -DestinationPath .
.\NotAlterra.exe
```

You can also extract the `.zip` using Windows Explorer (right-click →
**Extract All**), then open **Command Prompt** or **PowerShell** in that
folder and run `.\NotAlterra.exe`.

> [!IMPORTANT]
> **Running an unsigned executable on Windows** triggers a UAC "Unknown
> Publisher" warning, forcing you to click "Run anyway" and bypass
> Windows' safety net. This tool lacks a signature not because it's
> unsafe, but because services like SignPath Foundation require a
> well-established CI/CD pipeline, audit trail, provenance
> documentation, and community standing to meet their acceptance
> criteria.

## Usage

Run the binary. Use **Set save folder** from the main menu to enter your save
path manually (paste is supported). The menu is keyboard-driven:

| Key | Action |
|---|---|
| `↑` `↓` | Navigate |
| `Enter` | Select |
| `Esc` | Cancel / Back |
| `Y` / `N` | Accept / Decline in dialogs |

### Menu

1. **Set Subnautica 2 location** — enter your save folder path manually (paste supported)
2. **Recover save file** — pick a backup, preview metadata, overwrite the live save
3. **Set backup location** — choose where backup archives are stored (default: `~/NotAlterra`)
4. **Create full backup** — copies all `savegame_*` files to the backup root
5. **Restore full backup** — overwrite the save folder from a previous backup
6. **Manage UE5 Config (.ini) files** — backup, restore, or delete `.ini` files
7. **View disclaimer**
8. **Exit**


## Where Files Live

```
%LOCALAPPDATA%\Subnautica2\Saved\SaveGames\
    savegame_0.sav
    savegame_0.bak
    savegame_0_1.bak
    savegame_1.sav
    ...
```

### Linux (Proton / Steam Deck)

```
~/.steam/steam/steamapps/compatdata/1962700/pfx/drive_c/users/steamuser/AppData/Local/Subnautica2/Saved/SaveGames/
    savegame_0.sav
    savegame_0.bak
    ...
```

Use **Set Subnautica 2 location** from the menu to enter your save path
(paste supported). The path is persisted to `app.ini` and restored on
next launch.

## Session Persistence

Configuration is stored in the standard platform config directory:

| Platform | Config path |
|---|---|
| Windows | `%LOCALAPPDATA%\NotAlterra\config\app.ini` |
| Linux | `~/.local/share/NotAlterra/config/app.ini` |

The `app.ini` file stores your save-folder path, backup location, and
disclaimer acceptance. It is auto-created when you first set any of
these options. Delete `NOTALTERRA_LICENSE_ACCEPTED` in the same directory
to re-prompt the disclaimer on next launch.

> **Privacy note**: `app.ini` stores your save-folder and backup paths —
> the minimum needed to avoid re-entering them each session. These paths
> may reveal your system username (e.g. `C:\Users\jane\...`). The
> information never leaves your machine — NotAlterra has no network
> access and no telemetry. The file is plain text; you can inspect or
> delete it at any time.

Backup archives are stored in your user data directory by default:

| Platform | Backup root |
|---|---|
| Windows | `C:\Users\<you>\NotAlterra\backups\saves\` |
| Linux | `~/NotAlterra/backups/saves/` |

UE5 Config `.ini` backups go into `backups/ue5/` under the same root.

You can change the backup root at any time via **Set backup location**
in the main menu. The path is persisted in `app.ini`.


## Platform Support

- **Windows** — fully tested and supported.
- **Linux** — builds and runs. Use **Set save folder** to enter your save
  path (typical Proton locations are shown under "Where Files Live" above).


## Backup Types

NotAlterra creates two kinds of save backups. The label in the restore
picker tells you which is which.

| Label | How it's created |
|---|---|
| `Full Backup — <date>` | Manually via **Create full backup** in the main menu. |
| `Pre-restore — <date>` | Automatically when you use **Restore full backup**. |

When you use **Restore full backup**, NotAlterra automatically takes a
safety snapshot of your current saves **before** it does anything. This is
called a **Pre-restore** backup.

If the restore doesn't go as expected, your old saves haven't disappeared —
they're right there in the restore picker labeled `Pre-restore — <date>`.
You can restore from that snapshot just like any other backup.

Pre-restore snapshots are never deleted automatically. Once you're happy
with the restore, you can delete them manually from the file system.

## Safety

- Runs in your user context — no admin privileges required
- No network connections
- Read-only inspect mode won't touch files
- Pre-restore snapshots created automatically
- .ini delete requires a prior backup


## License

MIT — see `LICENSE.md`.
