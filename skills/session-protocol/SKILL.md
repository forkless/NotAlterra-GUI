# Session Protocol

## Rules

1. Check git status before any operation
2. Read files before editing (required by edit_file tool)
3. Clean obj/ before publish to avoid stale compiled XAML
4. Verify edits by reading back the file
5. Commit frequently with descriptive messages
6. Tag versions after commit + push

## Build Cycle

1. Edit source
2. `dotnet build -c Debug` — verify compilation
3. `dotnet publish -c Release -f net9.0-windows10.0.26100.0 -o tmp_min/` — publish
4. `iscc scripts/setup.iss /D... /O"publish"` — build installer
5. Install and test on target machine

## Notes

- `Start-Process` kills children on this dev machine
- Use `cmd /c start "" "path\to.exe"` for launching
- Close NotAlterra processes before ISCC compile (file locking)
