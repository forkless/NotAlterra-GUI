# Security Audit Notes

## Attack Surface

- **No network connections** — zero networking code in NotAlterra
- **No telemetry** — Windows App SDK opt-in telemetry not enabled
- **No external dependencies at runtime** — .NET 9 + WinAppSDK 1.8 are the only dependencies
- **Registry-only config** — no files written beyond registry and backup archives

## Data Storage

| Data | Location | Sensitivity |
|------|----------|-------------|
| Save folder path | HKCU\Software\NotAlterra | Low (path only) |
| Backup root path | HKCU\Software\NotAlterra | Low (path only) |
| Backup archives | %USERPROFILE%\NotAlterra\ | Medium (contains save data) |
| Transaction log | %USERPROFILE%\NotAlterra\transaction.log | Low (timestamps only) |

## Signing

Installer is self-signed with a generated certificate. Not an EV certificate — Windows will show "Unknown publisher" warning.

## Known Security Gaps

- No integrity verification of the installer
- No automatic updates — user must manually download new versions
- Registry keys are writable by the user (standard HKCU behavior)
