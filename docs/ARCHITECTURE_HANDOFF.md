# Architecture & Handoff Notes

## Project Structure

- `src/NotAlterra.UI/` — WinUI 3 unpackaged desktop app (Avalonia project is legacy)
- `src/NotAlterra.Core/` — shared library: GVAS parser, save operations, config
- `src/NotAlterra.UI.Tests/` — xUnit test suite (96 tests + 2 skipped)
- `scripts/` — build/CI scripts: _gen_splash.py, _gen_wizard_imgs.py, setup.iss
- `gvas-files/` — synthetic test .sav fixtures

## Build & Deploy

- **Minimal** (~19 MB): framework-dependent, needs .NET 9 + WinAppSDK 1.8 runtime
- **Installer**: Inno Setup with [Code] section — detects missing runtimes and prompts download
- **CI**: GitHub Actions — dotnet test → smoke test → publish + iscc → draft release

## Key Technical Decisions

- Registry (`HKCU\Software\NotAlterra`) for config, not app.ini
- Heuristic byte-scan GVAS parser (not full UE5 structure walker)
- UndockedRegFreeWinRT for unpackaged WinUI 3
- Self-contained publish not used (Minimal only)

## Session Quirks

- `Start-Process` from PowerShell kills child on this dev machine — use `cmd /c start "" "path\to.exe"` instead
- Debug builds launched from source tree can lock files during ISCC compile
- `runasoriginaluser` flag in setup.iss `[Run]` prevents minimized window after install
- ISCC requires `choco install innosetup` in CI
