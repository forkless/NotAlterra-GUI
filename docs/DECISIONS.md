# Architectural Decisions

## ADR-001: Registry over app.ini (2026-06-29)

**Status:** Accepted

**Context:** app.ini was a plain-text config file in `%LOCALAPPDATA%\NotAlterra\config\`. It stored save_folder and backup_root paths with %LOCALAPPDATA% substitution for portability.

**Decision:** Switch to HKCU\Software\NotAlterra registry keys. Windows-native, per-user, no stray files.

**Consequences:**
- No migration path — old app.ini is ignored
- CA1416 warning suppressed (Windows-only app)
- Paths stored with %LOCALAPPDATA% / %USERPROFILE% substitution for portability

## ADR-002: Unpackaged WinUI 3 (2026-06-29)

**Status:** Accepted

**Context:** Originally built as MSIX. MSIX caused deployment complications for end users (signing, store requirements).

**Decision:** Switch to unpackaged with Inno Setup installer. Use UndockedRegFreeWinRT for activation.

**Consequences:**
- `WindowsPackageType=None`
- `WindowsAppSdkUndockedRegFreeWinRTInitialize=true`
- No `ms-appx:///` URIs in XAML
- Bootstrap DLLs bundled in published output

## ADR-003: Minimal installer only (2026-06-29)

**Status:** Accepted

**Context:** Two installer variants (Minimal: 19MB, framework-dep. Full: 45MB, self-contained). Full caused WinUI version mismatch.

**Decision:** Ship Minimal only. Installer prompts to download .NET 9 + WinAppSDK 1.8 if missing.

**Consequences:**
- Smaller download for end users
- Requires runtime installation step
- WinUI DLLs used from installed runtime (must match SDK version)

## ADR-004: Heuristic GVAS parser (2026-06-29)

**Status:** Accepted

**Context:** UE5 GVAS format is complex with custom versioning. Full parser would be a significant investment.

**Decision:** Use heuristic byte-scan — search for known property names (DisplayName, GameMode, etc.) as byte patterns.

**Consequences:**
- May miss fields if save format changes
- No write support — read-only metadata inspection
- ~16 known property tags supported
