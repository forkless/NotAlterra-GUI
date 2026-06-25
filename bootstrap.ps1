# bootstrap.ps1 — Install Rust + VS Build Tools for NotAlterra-GUI
# Run from D:\Development\NotAlterra-GUI
# Interactive mode — you'll see the installer UI.
# Run as Administrator.

$Script:LogFile = Join-Path $PSScriptRoot "bootstrap.log"
$Script:StartTime = Get-Date

function Log {
    param([string]$Message, [string]$Level = "INFO")
    $timestamp = Get-Date -Format "yyyy-MM-dd HH:mm:ss"
    $line = "[$timestamp] [$Level] $Message"
    Write-Host $line
    Add-Content -Path $Script:LogFile -Value $line -Encoding UTF8
}

function LogOk  { Log "OK: $($args -join ' ')" "OK" }
function LogWarn { Log "WARN: $($args -join ' ')" "WARN" }
function LogFail { Log "FAIL: $($args -join ' ')" "FAIL" }

# Remove old log
Remove-Item $Script:LogFile -Force -ErrorAction SilentlyContinue

Log "=== Bootstrap started ==="
Log "Working directory: $PSScriptRoot"
Log "OS: $((Get-CimInstance Win32_OperatingSystem).Caption)"
Log "Admin: $([Security.Principal.WindowsPrincipal]::new([Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator))"

$ErrorActionPreference = "Continue"

# ── 1. Rust toolchain ────────────────────────────────────────────────────────

Log ""
Log "=== Step 1: Rust toolchain ==="

$haveRust = $null -ne (Get-Command rustc -ErrorAction SilentlyContinue)

if (-not $haveRust) {
    Log "Rust NOT detected in PATH."
    Log "Downloading rustup-init.exe..."
    $url = "https://win.rustup.rs/x86_64"
    $out = "$env:TEMP\rustup-init.exe"
    try {
        Invoke-WebRequest -Uri $url -OutFile $out -UseBasicParsing
        LogOk "Downloaded rustup-init.exe ($((Get-Item $out).Length) bytes)"
    } catch {
        LogFail "Download failed: $_"
        exit 1
    }

    Log "Running rustup installer (default host: x86_64-pc-windows-msvc)..."
    $proc = Start-Process -FilePath $out -ArgumentList "--default-host x86_64-pc-windows-msvc --default-toolchain stable --profile default -y" -NoNewWindow -PassThru -Wait
    if ($proc.ExitCode -eq 0) {
        LogOk "rustup installer exited with code 0"
    } else {
        LogFail "rustup installer exited with code $($proc.ExitCode)"
    }

    # Refreshenv
    $env:Path = [Environment]::GetEnvironmentVariable('Path', 'User') + ';' + [Environment]::GetEnvironmentVariable('Path', 'Machine')

    $testRust = Get-Command rustc -ErrorAction SilentlyContinue
    if ($testRust) {
        LogOk "Rust installed: $(rustc --version)"
    } else {
        LogFail "Rust still not found in PATH after install"
    }
} else {
    LogOk "Already installed: $(rustc --version)"
    Log "PATH includes: $(Get-Command rustc).Source"
}

# Ensure MSVC target
Log "Checking Rust targets..."
$targets = rustup target list --installed
if ($targets -contains "x86_64-pc-windows-msvc") {
    LogOk "Target x86_64-pc-windows-msvc already installed"
} else {
    Log "Adding x86_64-pc-windows-msvc target..."
    $proc = Start-Process -FilePath (Get-Command rustup).Source -ArgumentList "target add x86_64-pc-windows-msvc" -NoNewWindow -PassThru -Wait
    if ($proc.ExitCode -eq 0) {
        LogOk "Target added"
    } else {
        LogFail "Failed to add target (exit $($proc.ExitCode))"
    }
}

# ── 2. VS Build Tools with C++ workload ─────────────────────────────────────

Log ""
Log "=== Step 2: Visual Studio Build Tools (C++ workload) ==="

# Check both possible install paths
$vcPaths = @(
    "C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Tools\MSVC",
    "C:\Program Files\Microsoft Visual Studio\18\VC\Tools\MSVC",
    "C:\Program Files\Microsoft Visual Studio\2022\BuildTools\VC\Tools\MSVC"
)
$needsVS = $true
foreach ($p in $vcPaths) {
    if (Test-Path $p) {
        LogOk "MSVC tools found at: $p"
        $needsVS = $false
    }
}

if (-not $needsVS) {
    LogOk "VS Build Tools already installed — skipping"
} else {
    LogWarn "MSVC tools NOT found (checked: $($vcPaths -join ', '))"

    # Check if VS Installer is present
    $vsInstaller = "C:\Program Files (x86)\Microsoft Visual Studio\Installer\setup.exe"
    if (Test-Path $vsInstaller) {
        $vsVer = (Get-Item $vsInstaller).VersionInfo
        Log "VS Installer found: $vsInstaller (v$($vsVer.FileVersion))"
    } else {
        LogWarn "VS Installer not found at expected path"
    }

    # Check for any existing VS installations
    Log "Checking for existing VS products..."
    $vswhere = "C:\Program Files (x86)\Microsoft Visual Studio\Installer\vswhere.exe"
    if (Test-Path $vswhere) {
        $installed = & $vswhere -latest -products * -format json 2>&1 | ConvertFrom-Json
        if ($installed) {
            Log "Existing VS installations found:"
            $installed | % { Log "  - $($_.displayName) at $($_.installationPath)" }
        } else {
            Log "No existing VS installations found"
        }
    }

    # Clean up stale state from previous failed attempts
    $stalePaths = @(
        "$env:LOCALAPPDATA\Microsoft\VisualStudio\Packages\_Instances",
        "$env:LOCALAPPDATA\Microsoft\VisualStudio\Packages\_Channels",
        "C:\Program Files\Microsoft Visual Studio\18",
        "$env:LOCALAPPDATA\Microsoft\VisualStudio\Packages\_bootstrapper"
    )
    foreach ($sp in $stalePaths) {
        if (Test-Path $sp) {
            Log "Cleaning stale path: $sp"
            Remove-Item $sp -Recurse -Force -ErrorAction SilentlyContinue
        }
    }

    # Download bootstrapper
    $bootstrapper = "$env:TEMP\vs_BuildTools.exe"
    $bootstrapperUrl = "https://aka.ms/vs/17/release/vs_BuildTools.exe"
    if (Test-Path $bootstrapper) {
        Log "Bootstrapper already cached: $bootstrapper ($((Get-Item $bootstrapper).Length) bytes)"
    } else {
        Log "Downloading VS Build Tools bootstrapper from $bootstrapperUrl..."
        try {
            Invoke-WebRequest -Uri $bootstrapperUrl -OutFile $bootstrapper -UseBasicParsing
            LogOk "Downloaded ($((Get-Item $bootstrapper).Length) bytes)"
        } catch {
            LogFail "Download failed: $_"
            exit 1
        }
    }

    Log ""
    Log "=== LAUNCHING INSTALLER ==="
    Log "The Visual Studio Installer will open in a new window."
    Log "Click YES on any UAC prompt."
    Log ""
    Log "In the installer:"
    Log "  1. Find 'Desktop development with C++' workload"
    Log "  2. On the right panel, ensure 'Windows 11 SDK' is CHECKED"
    Log "  3. Click the INSTALL button (bottom-right)"
    Log "  4. WAIT for completion (may take 5-15 minutes, downloading ~2GB)"
    Log "  5. Close the installer window when done"
    Log ""
    Log "Waiting for installer to close..."

    $proc = Start-Process -FilePath $bootstrapper -PassThru -Wait
    Log "Bootstrapper exited with code $($proc.ExitCode)"

    # Check installer logs for clues
    $tmpLogs = Get-ChildItem "$env:TEMP" -Filter "dd_*" -ErrorAction SilentlyContinue | Sort-Object LastWriteTime -Descending | Select-Object -First 5
    if ($tmpLogs) {
        Log "Recent installer logs:"
        $tmpLogs | % {
            $lines = Get-Content $_.FullName -Tail 3 -ErrorAction SilentlyContinue
            Log "  $($_.Name): $($lines -join ' | ')"
        }
    }
}

# ── 3. Verify ────────────────────────────────────────────────────────────────

Log ""
Log "=== Step 3: Verification ==="

$env:Path = [Environment]::GetEnvironmentVariable('Path', 'User') + ';' + [Environment]::GetEnvironmentVariable('Path', 'Machine')

$rustcOk = $null -ne (Get-Command rustc -ErrorAction SilentlyContinue)
$cargoOk = $null -ne (Get-Command cargo -ErrorAction SilentlyContinue)

if ($rustcOk) { LogOk "rustc: $(rustc --version)" } else { LogFail "rustc NOT FOUND" }
if ($cargoOk) { LogOk "cargo: $(cargo --version)" } else { LogFail "cargo NOT FOUND" }

Log "Installed Rust targets:"
rustup target list --installed | ForEach-Object { Log "  $_" }

$msvcFound = $false
foreach ($p in $vcPaths) {
    if (Test-Path $p) {
        $tools = Get-ChildItem $p -ErrorAction SilentlyContinue | Select-Object -ExpandProperty Name
        LogOk "MSVC tools at $($p): $($tools -join ', ')"
        $msvcFound = $true
    }
}

# Check for link.exe
$linkPath = Get-Command link.exe -ErrorAction SilentlyContinue
if ($linkPath) {
    LogOk "link.exe found: $($linkPath.Source)"
} else {
    LogWarn "link.exe NOT in PATH — cargo may need vcvars64.bat / Developer Command Prompt"
}

# Check Windows SDK
$sdkPaths = @(
    "C:\Program Files (x86)\Windows Kits\10\Include\10.0*",
    "C:\Program Files (x86)\Windows Kits\10\Lib\10.0*"
)
foreach ($pattern in $sdkPaths) {
    $matches = Get-ChildItem $pattern -ErrorAction SilentlyContinue
    if ($matches) {
        LogOk "Windows SDK: $($matches[-1].Name)"
    } else {
        LogWarn "Windows SDK not found at: $pattern"
    }
}

$elapsed = (Get-Date) - $Script:StartTime
Log ""
Log "=== Bootstrap finished in $($elapsed.TotalSeconds.ToString('F1'))s ==="

if ($msvcFound -and $rustcOk -and $cargoOk) {
    $summary = "ALL CHECKS PASSED"
    $color = "Green"
} else {
    $summary = "SOME CHECKS FAILED — review log above"
    $color = "Red"
}
Log $summary

Write-Host ""
Write-Host "=== $summary ===" -ForegroundColor $color
Write-Host "Log written to: bootstrap.log" -ForegroundColor Cyan
Write-Host "If everything passed, open a NEW PowerShell window and run:" -ForegroundColor White
Write-Host "  cd D:\Development\NotAlterra-GUI" -ForegroundColor Yellow
Write-Host "  cargo build" -ForegroundColor Yellow
