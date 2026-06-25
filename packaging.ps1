# MakeAppx + SignTool packaging for NotAlterra
# Run this after building the Release configuration.

$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$buildDir = Join-Path $scriptDir "build"
$pkgDir = Join-Path $scriptDir "package"
$msixPath = Join-Path $scriptDir "NotAlterra.msix"

New-Item -ItemType Directory -Force -Path $pkgDir | Out-Null

# Copy built files
Copy-Item (Join-Path $buildDir "NotAlterra.exe") (Join-Path $pkgDir "NotAlterra.exe") -Force
Copy-Item (Join-Path $scriptDir "AppxManifest.xml") (Join-Path $pkgDir "AppxManifest.xml") -Force

# Create placeholder icon
$assetsDir = Join-Path $pkgDir "Assets"
New-Item -ItemType Directory -Force -Path $assetsDir | Out-Null
Add-Type -AssemblyName System.Drawing
$bmp = New-Object System.Drawing.Bitmap(1, 1)
$bmp.Save((Join-Path $assetsDir "icon.png"), [System.Drawing.Imaging.ImageFormat]::Png)
$bmp.Dispose()

# Find MakeAppx
$makeAppx = Get-ChildItem "$env:ProgramFiles (x86)\Windows Kits\10\bin\*" -Recurse -Filter "makeappx.exe" | Sort-Object FullName -Descending | Select-Object -First 1 -ExpandProperty FullName
if (-not $makeAppx) { Write-Error "MakeAppx.exe not found"; exit 1 }

# Find SignTool
$signTool = Get-ChildItem "$env:ProgramFiles (x86)\Windows Kits\10\bin\*" -Recurse -Filter "signtool.exe" | Sort-Object FullName -Descending | Select-Object -First 1 -ExpandProperty FullName
if (-not $signTool) { Write-Error "SignTool.exe not found"; exit 1 }

# Find cert
$cert = Get-ChildItem Cert:\CurrentUser\My | Where-Object { $_.Subject -eq "CN=NotAlterra" } | Select-Object -First 1
if (-not $cert) { Write-Error "Cert CN=NotAlterra not found"; exit 1 }

Write-Host "Creating MSIX package..."
& $makeAppx pack /d $pkgDir /p $msixPath /l
if ($LASTEXITCODE -ne 0) { Write-Error "MakeAppx failed"; exit 1 }

Write-Host "Signing MSIX package..."
& $signTool sign /fd SHA256 /a /s MY /n "NotAlterra" $msixPath
if ($LASTEXITCODE -ne 0) { Write-Error "SignTool failed"; exit 1 }

Remove-Item $pkgDir -Recurse -Force -ErrorAction SilentlyContinue
Write-Host "MSIX created: $msixPath" -ForegroundColor Green
