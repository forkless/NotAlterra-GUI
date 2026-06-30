# Release Workflow

## Creating a Release

1. Ensure all tests pass: `dotnet test src/NotAlterra.UI.Tests -c Debug`
2. Update CHANGELOG.md
3. Commit: `git add -A && git commit -m "release: vX.Y.Z"`
4. Tag: `git tag vX.Y.Z && git push origin vX.Y.Z`
5. CI builds installer and creates draft release
6. Publish draft on GitHub Releases page

## Version Scheme

- v0.5.x — pre-release development
- v1.0.0 — first stable release (post-Early Access)
- Tags use `v` prefix (e.g., `v0.5.1`)

## Installer Output

- `NotAlterra-{tag}-Windows-x64.exe` — Minimal (framework-dep)
- ~19 MB installer
- Draft release created automatically by CI
