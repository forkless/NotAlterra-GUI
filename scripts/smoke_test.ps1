# Smoke test: launch published NotAlterra, verify it stays alive for 15s
param([string]$ExePath = "tmp_full\NotAlterra.UI.exe")

$p = Start-Process -FilePath $ExePath -PassThru -NoNewWindow
Write-Host "Launched PID: $($p.Id)"

Start-Sleep -Seconds 15
$p.Refresh()

if ($p.HasExited) {
    Write-Host "FAIL: Exited with code $($p.ExitCode)"
    exit 1
} else {
    Write-Host "PASS: Process $($p.Id) still alive after 15s"
    $p.Kill()
    exit 0
}
