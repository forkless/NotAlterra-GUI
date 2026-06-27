# Fuzz runner for NotAlterra.
# Prerequisites: dotnet tool install -g SharpFuzz.CommandLine
#
# Usage:
#   .\fuzz.ps1                                # gvas_parse, 100k runs
#   .\fuzz.ps1 -Target gvas_full_meta         # specific target
#   .\fuzz.ps1 -Target tar_gz_integrity -Runs 50000
#   .\fuzz.ps1 -Target gvas_parse -Corpus .\corpus\gvas_parse\

param(
    [string]$Target = "gvas_parse",
    [int]$Runs = 100000,
    [string]$Corpus = ""
)

$ErrorActionPreference = "Stop"
$ProjectDir = "$PSScriptRoot\NotAlterra.Fuzz"
$OutDir = "$ProjectDir\bin\Release\net9.0"

Write-Host "=== Build fuzz project ===" -Foreground Cyan
dotnet build $ProjectDir -c Release --nologo -v q
if ($LASTEXITCODE -ne 0) { throw "Build failed" }

Write-Host "=== Instrument assemblies ===" -Foreground Cyan
sharpfuzz "$OutDir\NotAlterra.Fuzz.dll" 2>&1 | Out-Null
if ($LASTEXITCODE -ne 0) { throw "sharpfuzz instrumentation failed" }
sharpfuzz "$OutDir\NotAlterra.Core.dll" 2>&1 | Out-Null
if ($LASTEXITCODE -ne 0) { throw "sharpfuzz Core instrumentation failed" }

if ($Corpus -eq "") {
    $Corpus = "$PSScriptRoot\corpus\$Target"
}
if (-not (Test-Path $Corpus)) {
    Write-Warning "Corpus dir not found: $Corpus — creating empty"
    New-Item -ItemType Directory -Force -Path $Corpus | Out-Null
}

Write-Host "=== Fuzzing target: $Target ===" -Foreground Cyan
$fuzzArgs = @("run", "--project", $ProjectDir, "-c", "Release", "--no-build", "--", $Target, "--", "-runs=$Runs", $Corpus)
dotnet $fuzzArgs 2>&1

if ($LASTEXITCODE -eq 0) {
    Write-Host "=== $Target: $Runs runs, no crashes ===" -Foreground Green
} else {
    Write-Host "=== $Target: CRASH FOUND (exit $LASTEXITCODE) ===" -Foreground Red
    exit $LASTEXITCODE
}
