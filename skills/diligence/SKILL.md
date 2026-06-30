# Diligence — Pre-Commit Checklist

Run these checks before claiming a task is complete:

- [ ] `dotnet test src/NotAlterra.UI.Tests -c Debug` — all tests pass
- [ ] `dotnet build src/NotAlterra.UI -c Debug` — 0 errors, 0 warnings
- [ ] If XAML was changed: `Remove-Item src\NotAlterra.UI\obj -Recurse -Force` then rebuild
- [ ] If installer changed: run `iscc` to verify compile
- [ ] `git status` — no unexpected files
- [ ] Verify the app launches (Release build)
