# CI/CD Pipeline Setup

## GitHub Actions Workflow

The CI workflow runs on push to main and tag pushes (v*):

1. **test** job (ubuntu-latest)
   - Checkout → setup-dotnet → dotnet restore → dotnet test
   - Runs xUnit tests (GvasParser, AppConfig, Guard, SaveOps)

2. **smoke** job (windows-latest)
   - dotnet publish → launch app → verify 15s uptime

3. **release** job (windows-latest, tag only)
   - dotnet publish → iscc (Inno Setup) → draft release with softprops/action-gh-release

## Inno Setup in CI

```powershell
choco install innosetup -y --no-progress
pip install Pillow -q
python scripts/_gen_splash.py "$tag"
dotnet publish src/NotAlterra.UI -c Release -f net9.0-windows10.0.26100.0 -o tmp_min/
iscc scripts/setup.iss /DMyAppVersion="$tag" /DSourceDir=tmp_min /F"NotAlterra-$tag-Windows-x64" /O"publish"
```

## Smoke Test (inlined in CI yml)

```powershell
$p = Start-Process -FilePath tmp_smoke/NotAlterra.UI.exe -PassThru -NoNewWindow
Start-Sleep -Seconds 15
if ($p.HasExited) { exit 1 } else { Stop-Process $p }
```
