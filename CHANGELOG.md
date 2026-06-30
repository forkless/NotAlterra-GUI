# Changelog

All notable changes to NotAlterra are documented in this file.

---

## [v0.5.3] 2026-06-30

### Added
- **Metadata hover card** - hover slot number badge or bak index badge, popup shows all 13 GVAS metadata fields (460px wide, card-relative positioning, vertical flip if near page bottom)
- **Pre-recovery snapshots** - automatic .bak snapshots now stored in `{BackupRoot}/backups/saves/` instead of game save folder
- **Pre-recovery Snapshots panel** - collapsible section on Backups page, lists snapshots by slot, Restore and Delete support
- **Collapsible backup panels** - Manual Backups and Pre-recovery Snapshots sections on Backups page, chevron toggle matching save slots style
- **Bak index badge** - orange numbered badge on left of each backup entry, serves as hover trigger for metadata card
- **Synthetic test fixtures** - 4 save slots with all 13 metadata fields, 3 backups, non-corrupt (300KB padded past threshold)
- **Windows Sandbox config** - `NotAlterra.wsb` with read-only folder mappings for installer, scripts, and test fixtures
- **Sandbox setup script** - `_sandbox_setup.ps1` sets burgundy desktop background, removes shortcuts, opens C:\Installer

### Changed
- **GVAS parser size check** - "too small" corruption only triggers when no metadata is extracted, allowing small valid files through
- **Corruption tooltips removed** - redundant ToolTips on corruption glyphs removed; hover card provides full metadata including corruption status
- **Corruption status in card** - plain text "None" or reason, no colored text, no glyph in card value area
- **Slot card hover** - trigger moved from entire card to the slot number badge only
- **Hover card border removed** - no orange border, clean card background only
- **Backups page** - split into two panels instead of one (Manual Backups + Pre-recovery Snapshots)
- **Hover card width** - 340px to 460px for corruption message fitting
- **CI release** - set to draft mode by default
- **Assets cleaned** - removed 15 redundant MSIX logo references from csproj, keep wildcard `Assets\**`
- **Sidebar video** - converted loop.webm to loop.mp4 (CRF 30, 7.86 MB), ships only MP4 in installer

### Fixed
- **Pre-recovery snapshot location** - moved from game save folder to backup directory (was polluting slot listing with timestamp bak files)
- **Bak index display** - `_pre_recover_` files excluded from backup listing, real backup versions show correct index
- **Hover card off-screen** - vertical flip positions popup above card when near page bottom
- **CI test regression** - `TooSmall` test updated to match new parser corruption logic

---

## [v0.5.1] 2026-06-29

### Added
- **Inno Setup installer** - MSIX replaced with Inno Setup (Minimal variant, ~19 MB)
- **Unpackaged WinUI 3** - WindowsPackageType=None, DeploymentManager disabled, UndockedRegFreeWinRT enabled
- **Title bar icon** - multi-res .ico (16-256px) via AppWindow.SetIcon() in Loaded event
- **Registry-based config** - replaced app.ini with HKCU\Software\NotAlterra, %LOCALAPPDATA%/%USERPROFILE% substitution
- **Folder pickers** - Browse + Reset for Game Save Folder and Backup Location in Settings
- **Production save path detection** - from registry with %LOCALAPPDATA%\Subnautica2 default fallback
- **Prerequisite installer** - setup.iss detects .NET 9 + WinAppSDK 1.8, downloads and installs if missing
- **Dynamic version** - Title bar and About page read assembly version, CI patches csproj from git tag

### Changed
- **About page** - Game Guard, Disclaimer, Planned Features moved from Settings; Privacy merged in
- **Home page** - Locations card showing active save/backup paths with %LOCALAPPDATA%/%USERPROFILE% display
- **Accent color** - Amber (#FFE8A84C) to Burnt Orange (#FFe85d04)
- **Installer logo** - Wizard BMPs generated from na.png, inno setup padded logo (65x65)
- **License page** - blanked LicenseLabel for clean MIT display
- **CI pipeline** - builds Minimal only, uploads versioned + version-agnostic installer
- **Synthetic .sav fixtures** - replaced real save files with generated test data
- **README** - end-user focused, no build instructions, direct download link

### Fixed
- **Game guard** - now checks before every destructive operation (Recover, Restore, Delete), not just at app launch
- **0xc000027b crash** - unpackaged WinUI 3 COM crash fixed (WindowsPackageType=None + UndockedRegFreeWinRT)
- **Splash DestroyWindow thread safety** - dispatched to DispatcherQueue
- **Window position after install** - runasoriginaluser flag in Inno Setup [Run] prevents minimized launch
- **ms-appx:/// URIs** - removed from MainWindow and AboutPage (crash unpackaged)
- **XAML builds** - FontAwesome ttf URI changed from ms-appx:/// to relative path

### Removed
- MSIX packaging (replaced by Inno Setup)
- Disclaimer sentinel file (replaced by Inno Setup License page)
- app.ini configuration file (replaced by registry)

---

## [v0.5.0] 2026-06-28

### Added
- **Pure C# WinUI 3 app** - replaced Rust TUI + C++ WinUI attempts with .NET 9 WinAppSDK 1.8
- **`NotAlterra.Core` class library** - GVAS parser + Services extracted for testability
- **xUnit test project** (`NotAlterra.UI.Tests`) - 96 tests covering BinaryReader, SlotUtils, GvasParser, Guard, SaveOps, AppConfig
- **FsCheck property-based fuzzing** - 15 properties at 500/200/100 iterations, runs inside `dotnet test`
- **GVAS BinaryReader** - overflow-safe `ReadFString` with max 10MB size cap, heap alloc instead of stackalloc
- **About page** - over-engineered stats card (bus factor 1, 96 tests)
- **Native splash screen** - Win32 layered transparent window (per-pixel alpha), 3s splash + 1.5s gap before app
- **Window centering** - app opens centered on desktop via `MonitorFromWindow` + `DisplayArea`
- **Disclaimer dialog** - ContentDialog on first launch, Accept/Decline, creates sentinel
- **Version display** - reads from assembly, shows `0.5.0 AMD64`

### Changed
- **Legacy cleanup** - all Rust, C++, CMake, stale artifacts moved to `legacy/` then removed
- **README** - rewritten for .NET 9 / WinUI 3, screenshot at top
- **CI pipeline** - GitHub Actions: restore -> build -> test; action versions bumped
- **Minimum .sav size** - raised from 500 bytes to 100KB (smallest real save is 301KB)
- **Backup sort** - changed from filename order to `LastWriteTimeUtc` descending
- **Changelog + handoff** - rewritten to reflect C# era

### Fixed
- **BinaryReader.ReadFString overflow** - FsCheck found overflow on `int.MinValue`, added `checked()` + 10MB cap
- **Nullable warnings** - `OnLoaded` params made nullable, CS8625 squashed

### Removed
- All Rust code (`Cargo.toml`, `src/*.rs`, `fuzz/`, `tests/`, `examples/`)
- All C++ WinUI code (`NotAlterra/`, `src/ui/`, `CMakeLists.txt`, `NotAlterra.vcxproj`, `AppxManifest.xml`)
- CMake build system, MSBuild project files, old packaging scripts
- SharpFuzz fuzz project (replaced with FsCheck)
- Stale root artifacts: `bootstrap.log`, `build_msbuild.log`, `vc140.pdb`, `NUL.obj`
- Stale test data: dummy slot 9, old backups, old snapshots in `gvas-files/`

### Project Stats
- **Language:** 100% C# (via `.gitattributes` linguist-vendored)
- **Lines:** ~400 (GVAS parser) + ~1500 (Services + UI)
- **Tests:** 96 (85 xUnit + 15 FsCheck property)
- **Dependencies:** 4 NuGet packages
- **Bus factor:** 1

---

## [v0.5.0-alpha] 2026-06-25

### Added
- **Full C++ rewrite** - GVAS parser ported from Rust (236 lines vs 644), 14 Google Tests
- **WinUI 3 desktop shell** - C++/WinRT, NavigationView with Dashboard/Saves/Backups/Config pages
- **CMake build system** - NMake generator, FetchContent for GTest, tl::expected error handling
- **CI pipeline** - GitHub Actions: Debug + Release builds, tests, release draft
- **MSIX packaging** - self-signed, auto-resolves WinAppSDK dependency
- **Privacy statement** - no network connections, no telemetry

### Changed
- Runtime dependency: Windows App SDK 1.8 (bootstrap DLL shipped alongside .exe)
- Build requirements: VS 2022 Build Tools + CMake 3.20+

### Removed
- All Rust code - Cargo.toml, tui.rs, fuzz targets, examples (kept as git history)
