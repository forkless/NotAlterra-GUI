# NotAlterra

**Subnautica 2 Save Manager**

NotAlterra is an unofficial tool for managing Subnautica 2 save files. It locates your save folders, backs up and restores `.sav` / `.bak` files, and lets you inspect the internal metadata of every save.

Not affiliated with Unknown Worlds Entertainment or KRAFTON.

---

## Features

- **Auto-locate** save folders across Steam, Xbox, Epic, and custom installs
- **Recover** a corrupted `.sav` from its `.bak` backup with rollback
- **Create / restore** full backups (only `savegame_*` files)
- **Manage** UE5 Config `.ini` files — backup, restore, delete
- **Inspect** any `.sav` or `.bak` file — full GVAS metadata dump
- **Slot grouping** with Multiplayer / Single Player detection
- **Playtime extraction** — displays total playtime from save metadata
- **Warns** on name changes and mode switches before recovery
- **Startup reminder** to close Subnautica 2 before use
- **Transaction log** — all actions timestamped to `transaction.log`
- **Cross-platform** — Linux and Windows console builds

---

## How to Build

```bash
# Linux
cargo build --release

# Windows (from Linux with mingw-w64)
rustup target add x86_64-pc-windows-gnu
sudo apt install mingw-w64
cargo build --release --target x86_64-pc-windows-gnu
```

---

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

> You can also extract the `.zip` using Windows Explorer (right-click →
> **Extract All**), then open **Command Prompt** or **PowerShell** in that
> folder and run `.\NotAlterra.exe`.

## Usage

Run the binary. On first launch it auto-scans for your save folder. The menu is keyboard-driven:

| Key | Action |
|---|---|
| `↑` `↓` | Navigate |
| `Enter` | Select |
| `Esc` | Cancel / Back |
| `Y` / `N` | Accept / Decline in dialogs |

### Menu

1. **Recover save file** — pick a backup, preview metadata, overwrite the live save
2. **Create full backup** — copies all `savegame_*` files to `NotAlterra_Backups`
3. **Restore full backup** — overwrite the save folder from a previous backup
4. **Inspect save files** — view all GVAS properties of any `.sav` / `.bak`
5. **Manage UE5 Config (.ini) files** — backup, restore, or delete `.ini` files
6. **View disclaimer**
7. **Exit**

---

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

> For older Steam installs, replace `~/.steam` with
> `~/.local/share/Steam` — NotAlterra checks both.

Backups are stored in `NotAlterra_Backups\` alongside the binary.

---

## config.ini

Created automatically next to the binary:

```ini
[alterra]
last_path = C:\Users\...\Subnautica2\Saved\SaveGames
last_scan = 2026-05-31 18:00:00
disclaimer_accepted = true
config_path = C:\Users\...\Subnautica2\Saved\Config\Windows
```

Delete `config.ini` to force a fresh scan on next launch.

> Only the disclaimer flag and save-folder paths are stored — no backup
> state or file metadata is persisted.

---

## Platform Support

- **Windows** — fully tested and supported.
- **Linux** — builds and runs, but automatic save-file detection has not been
  tested on a Linux/Steam Deck install yet. The directory layout should be the
  same, but feedback and bug reports are appreciated.

---

## Safety

- Runs in your user context — no admin privileges required
- No network connections
- Read-only inspect mode won't touch files
- Pre-restore snapshots created automatically
- .ini delete requires a prior backup

---

## License

MIT — see `LICENSE.md`.
