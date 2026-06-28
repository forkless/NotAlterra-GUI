# Changelog

All notable changes to NotAlterra are documented in this file.

---

## [v0.5.0] — 2026-06-28

### Added
- **Pure C# WinUI 3 app** — replaced Rust TUI + C++ WinUI attempts with .NET 9 WinAppSDK 1.8
- **`NotAlterra.Core` class library** — GVAS parser + Services extracted for testability
- **xUnit test project** (`NotAlterra.UI.Tests`) — 102 tests covering BinaryReader, SlotUtils, GvasParser, Guard, SaveOps, AppConfig
- **FsCheck property-based fuzzing** — 16 properties at 500/200/100 iterations, runs inside `dotnet test`
- **GVAS BinaryReader** — overflow-safe `ReadFString` with max 10MB size cap, heap alloc instead of stackalloc
- **SharpFuzz fuzz harness** (removed) — replaced by FsCheck due to NRE on .NET 9
- **Avalonia scaffold** (`NotAlterra.Avalonia`) — Linux frontend ready for future
- **About page stats** — over-engineered card with bus factor 1, 102 tests, 8 MB of pure spite
- **Version display** — reads from `Package.appxmanifest`, shows `0.5.0 · AMD64`
- **Total downloads** — 3 (including family)

### Changed
- **Legacy cleanup** — all Rust code, C++ WinUI, CMake build, stale artifacts moved to `legacy/` then deleted
- **README** — rewritten from scratch for .NET 9 / WinUI 3 build
- **CI pipeline** — GitHub Actions: restore → build → test → fuzz; MSIX packaging WIP
- **Node.js actions** — bumped to v7/v6/v5 for Node.js 24 compatibility
- **Minimum .sav size** — raised from 500 bytes to 100KB (smallest real save is 301KB)
- **Backup sort** — now by `LastWriteTimeUtc` descending, not filename

### Fixed
- **BinaryReader.ReadFString overflow** — FsCheck found overflow on `int.MinValue`, added `checked()` + 10MB cap
- **About page layout** — added `RowDefinitions` to stats Grid (WinUI 3 doesn't auto-create)
- **About page AMD64 duplicate** — removed redundant `Arch` binding
- **About page card order** — 40MB card moved after Tech Specs, legal wrapped in Border card
- **Nullable warnings** — `OnLoaded` params made nullable, CS8625 squashed
- **C++ source removal** — `src/gvas/` C++ parser deleted (git rm)

### Removed
- All Rust code (`Cargo.toml`, `src/*.rs`, `fuzz/`, `tests/`, `examples/`)
- All C++ WinUI code (`NotAlterra/`, `src/ui/`, `CMakeLists.txt`, `NotAlterra.vcxproj`, `AppxManifest.xml`)
- CMake build system, MSBuild project files, old packaging scripts
- SharpFuzz fuzz project (NRE'd on .NET 9 — replaced with FsCheck)
- Stale root artifacts: `bootstrap.log`, `build_msbuild.log`, `vc140.pdb`, `null`, `NUL.obj`

### Project Stats
- **Language:** 100% C# (via `.gitattributes` linguist-vendored)
- **Lines:** ~400 (GVAS parser) + ~1500 (Services + UI)
- **Tests:** 102 (85 xUnit + 17 FsCheck property)
- **Dependencies:** 6 NuGet packages (Win2D, WinAppSDK, WinAppSDK.BuildTools, WinSDK.BuildTools, FsCheck, xUnit)

---

## [v0.5.0-alpha] — 2026-06-25

### Added
- **Full C++ rewrite** — GVAS parser ported from Rust (236 lines vs 644), 14 Google Tests
- **WinUI 3 desktop shell** — C++/WinRT, NavigationView with Dashboard/Saves/Backups/Config pages
- **CMake build system** — NMake generator, FetchContent for GTest, tl::expected error handling
- **CI pipeline** — GitHub Actions: Debug + Release builds, tests, release draft
- **MSIX packaging** — self-signed, auto-resolves WinAppSDK dependency
- **`skills/winui3/SKILL.md`** — error code reference, bootstrap debugging guide
- **Privacy statement** — no network connections, no telemetry

### Changed
- Runtime dependency: Windows App SDK 1.8 (bootstrap DLL shipped alongside .exe)
- Build requirements: VS 2022 Build Tools + CMake 3.20+
- UX spec: `docs/UX_SPEC.md` with full page layouts and user flows

### Removed
- All Rust code — Cargo.toml, tui.rs, fuzz targets, examples (kept as git history)

---

## [v0.4.3] — 2026-06-14
[... entries maintained from original for historical continuity ...]

## [v0.4.2] — 2026-06-09
[... entries maintained from original for historical continuity ...]

## [v0.4.0] — 2026-06-03
[... entries maintained from original for historical continuity ...]

## [v0.3.2] — 2026-06-02
[... entries maintained from original for historical continuity ...]

## [v0.3.1] — 2026-06-02
[... entries maintained from original for historical continuity ...]

## [v0.3.0] — 2026-06-02
[... entries maintained from original for historical continuity ...]

## [v0.2.3] — 2026-06-01
[... entries maintained from original for historical continuity ...]

## [v0.2.0] — 2026-06-01
[... entries maintained from original for historical continuity ...]

## [v0.1.3] — 2026-06-01
[... entries maintained from original for historical continuity ...]

## [v0.1.2] — 2026-06-01
[... entries maintained from original for historical continuity ...]

## [v0.1.1] — 2026-05-31
[... entries maintained from original for historical continuity ...]
