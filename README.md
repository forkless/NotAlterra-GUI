# NotAlterra

**Subnautica 2 Save Manager — Windows Native GUI**

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

## Privacy

**NotAlterra makes no network connections. No telemetry. No data leaves your machine.**

- The application never calls home — there are no update checks, no analytics, no crash reporters.
- It runs entirely offline. Your save files and their metadata stay on your machine.
- The Windows App SDK runtime has opt-in telemetry which NotAlterra does not enable.
- Configuration (`app.ini`) stores your save-folder and backup paths locally. These may reveal your system username; the file is plain text you can inspect or delete at any time.

If network features are added in a future version, they will require explicit user opt-in.

## How to Build

### Prerequisites

- Windows 11
- [Visual Studio 2022 Build Tools](https://aka.ms/vs/17/release/vs_BuildTools.exe) with **Desktop development with C++** workload
- [CMake](https://cmake.org/download/) 3.20+
- Windows App SDK (included with the C++ workload)

```powershell
# From a Developer Command Prompt for VS 2022 (x64):
cmake -B build -G "NMake Makefiles" -DCMAKE_BUILD_TYPE=Release
cmake --build build --config Release
```

### Quick start with bootstrap

Run `bootstrap.ps1` as Administrator to install all prerequisites automatically:

```powershell
.\bootstrap.ps1
```

## How to Run

Build output is at `build/notalterra_tests.exe` (test runner) and will eventually be a WinUI 3 application.

Pre-compiled binaries will be available from the [releases page](https://github.com/forkless/NotAlterra/releases).

## Where Files Live

```
%LOCALAPPDATA%\Subnautica2\Saved\SaveGames\
    savegame_0.sav
    savegame_0.bak
    savegame_0_1.bak
    savegame_1.sav
    ...
```

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
