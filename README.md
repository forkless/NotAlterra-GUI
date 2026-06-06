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

1. **Set save folder** — enter your save folder path manually (paste supported)
2. **Recover save file** — pick a backup, preview metadata, overwrite the live save
3. **Create full backup** — copies all `savegame_*` files to `NotAlterra_Backups`
4. **Restore full backup** — overwrite the save folder from a previous backup
5. **Inspect save files** — view all GVAS properties of any `.sav` / `.bak`
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

Use **Set save folder** from the menu to enter your save path (paste is
supported). The path exists only in memory for the current session —
re-enter it the next time you run the tool.

Backups are stored in `NotAlterra_Backups\` alongside the binary.


## Session Persistence

No configuration file is written to disk. The save folder path exists only
in memory for the current session — set it each time via **Set save folder**
from the main menu (paste is supported).

The disclaimer acceptance is tracked via a 0-byte sentinel file
(`NotAlterra_LICENSE_ACCEPTED`) alongside the binary. Delete this file to
re-prompt the disclaimer on next launch.


## Platform Support

- **Windows** — fully tested and supported.
- **Linux** — builds and runs. Use **Set save folder** to enter your save
  path (typical Proton locations are shown under "Where Files Live" above).


## Safety

- Runs in your user context — no admin privileges required
- No network connections
- Read-only inspect mode won't touch files
- Pre-restore snapshots created automatically
- .ini delete requires a prior backup


## License

MIT — see `LICENSE.md`.
