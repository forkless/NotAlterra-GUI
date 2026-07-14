# WinUI 3 Build for NotAlterra

## Build

```powershell
# Debug
dotnet build src/NotAlterra.UI -c Debug

# Release - Minimal (framework-dependent, ~19 MB)
Remove-Item "src\NotAlterra.UI\obj" -Recurse -Force
dotnet publish src/NotAlterra.UI -c Release -f net9.0-windows10.0.26100.0 -o tmp_min/

# Installer
iscc scripts/setup.iss /DMyAppVersion="v0.5.1-test" /DSourceDir=tmp_min /F"NotAlterra-v0.5.1-test-Windows-x64" /O"publish"
```

## Key Properties (csproj)

```xml
<WindowsPackageType>None</WindowsPackageType>
<WindowsAppSdkDeploymentManagerInitialize>false</WindowsAppSdkDeploymentManagerInitialize>
<WindowsAppSdkUndockedRegFreeWinRTInitialize>true</WindowsAppSdkUndockedRegFreeWinRTInitialize>
```

## XAML Rules

- No `ms-appx:///` URIs — use relative paths instead
- Font references: `../Assets/FontAwesomeBrands.ttf#Font Awesome 6 Brands`
- `x:Bind` on `<Run>` elements doesn't work — set Text via code-behind

## Deployment

- Inno Setup installer with [Code] for runtime detection
- `ArchitecturesInstallIn64BitMode=x64` — 64-bit only
- `runasoriginaluser` in [Run] section to prevent minimized window after install

## Launch Quirks

- `Start-Process` from PowerShell kills the child on this machine
- Use `cmd /c start "" "path\to.exe"` instead
- Debug build running from source tree can lock files during ISCC compile
