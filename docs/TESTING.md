# Testing

## Automated CI

GitHub Actions — runs on push/PR to `main`. Two jobs:

- **check**: restore → build → test → publish → 15s smoke test
- **release** (tags only): splash gen → publish → iscc → draft release

CI runs on `windows-latest` — clean ephemeral Windows Server environment.

## Manual Sandbox Testing

For destructive tests (recovery, backup, ini delete) without risking daily driver:

### Prerequisites

- Windows 11 Pro/Enterprise (for Windows Sandbox) or
- VirtualBox/VMware + Microsoft's free Win11 dev VM

### Option A: Windows Sandbox (lightest, Pro/Enterprise only)

1. `Win+R` → `Windows Sandbox`
2. Inside sandbox: install .NET 9 Desktop Runtime + WinAppSDK 1.8
3. Mount host folder with published NotAlterra build
4. Run NotAlterra.UI.exe, test destructive operations
5. Close sandbox — everything destroyed on exit

### Option B: VirtualBox snapshot (recommended for repeat testing)

1. Download Win11 dev VM from https://developer.microsoft.com/en-us/windows/downloads/virtual-machines/
2. Import to VirtualBox
3. Inside VM: install .NET 9 + WinAppSDK 1.8
4. **Visual cue** — set a solid red desktop background to distinguish from daily driver (right-click desktop → Personalize → Background → Solid color → red)
5. Take snapshot named `clean`
6. Mount shared folder with NotAlterra build
7. Test freely — backup, recovery, ini delete, whatever
8. Revert to `clean` snapshot
9. **Monthly**: delete old VM, download fresh copy (avoids 90-day eval expiry)

### Cleanup

No state persists between test sessions. The VM eval never expires if re-downloaded monthly — each fresh copy resets the 90-day clock.

## Known Issues

See `docs/KNOWN_ISSUES.md`.
