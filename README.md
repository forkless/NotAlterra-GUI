# NotAlterra

**Subnautica 2 save-file manager — cross-platform terminal application.**

NotAlterra is an unofficial tool for managing Subnautica 2 save files. It locates your save folders, backs up and restores `.sav` / `.bak` files, and lets you inspect the internal metadata of every save.

Not affiliated with Unknown Worlds Entertainment or KRAFTON.

---

## Features

- **Auto-locate** save folders across Steam, Xbox, Epic, and custom installs
- **Recover** a corrupted `.sav` from its `.bak` backup with rollback
- **Create / restore** full backups (only `savegame_*` files)
- **Manage** UE5 Config `.ini` files — backup, restore, delete
- **Inspect** any `.sav` or `.bak` file — full GVAS metadata dump
- **Slot grouping** with Online / Local detection
- **Warns** on name changes and online↔local mode switches before recovery
- **Game-running guard** — exits or warns if Subnautica 2 is active
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

## Usage

Run the binary. On first launch it auto-scans for your save folder. The menu is keyboard-driven:

| Key | Action |
|---|---|
| `↑` `↓` | Navigate |
| `Enter` | Select |
| `Esc` | Cancel / Back |
| `Y` / `N` | Accept / Decline in dialogs |

### Menu

1. **Recover .sav from .bak** — pick a backup, preview metadata, overwrite the live save
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

---

## Safety

- Runs in your user context — no admin privileges required
- No network connections
- Read-only inspect mode won't touch files
- Pre-restore snapshots created automatically
- Guard against file corruption: refuses overwrite if Subnautica 2 is running

---

## License

MIT — see `LICENSE.md`.
