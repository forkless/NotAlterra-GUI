# Sandbox setup — runs once at logon inside Windows Sandbox
# No persistence between sessions (intentional)

# ── 1. Solid red desktop background ──
Add-Type -AssemblyName System.Drawing
$bmp = [System.Drawing.Bitmap]::new(1, 1)
$bmp.SetPixel(0, 0, [System.Drawing.Color]::FromArgb(100, 20, 30))
$bmp.Save("$env:TEMP\red.bmp")
$bmp.Dispose()

Add-Type -TypeDefinition @"
using System;
using System.Runtime.InteropServices;
public class Wallpaper {
    [DllImport("user32.dll", CharSet = CharSet.Auto)]
    public static extern int SystemParametersInfo(int uAction, int uParam, string lpvParam, int fuWinIni);
}
"@
[Wallpaper]::SystemParametersInfo(0x0014, 0, "$env:TEMP\red.bmp", 0x0002) | Out-Null

# ── 2. Remove desktop shortcuts (clean slate) ──
$shortcuts = @(
    "$env:PUBLIC\Desktop\*.lnk",
    "$env:USERPROFILE\Desktop\*.lnk"
)
foreach ($pattern in $shortcuts) {
    Remove-Item $pattern -Force -ErrorAction SilentlyContinue
}

# ── 3. Open installer folder ──
Start-Process explorer.exe "C:\Installer"
