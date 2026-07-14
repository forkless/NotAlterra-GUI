$t = [Environment]::GetEnvironmentVariable("GH_PAT")
if ($t) {
    Write-Host "token set: length $($t.Length)"
} else {
    Write-Host "not set in local env"
}
