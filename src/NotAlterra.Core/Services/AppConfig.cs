// Path utilities and persistent app configuration.
// Ported from src/config.rs

using Microsoft.Win32;

namespace NotAlterra.Services;

public static class AppConfig
{
    private const string RegKey = @"Software\NotAlterra";

    // ── backup root ──────────────────────────────────────────────────────

    public static string GetBackupRoot()
    {
        var fromReg = LoadAppConfig().BackupRoot;
        return fromReg ?? Path.Combine(
            Environment.GetFolderPath(Environment.SpecialFolder.UserProfile),
            "NotAlterra");
    }

    // ── stale config.ini cleanup (pre-v0.3.0) ────────────────────────────

    public static string StaleConfigPath()
    {
        return Path.Combine(AppDomain.CurrentDomain.BaseDirectory, "config.ini");
    }

    public static bool CleanupStaleConfig()
    {
        var path = StaleConfigPath();
        if (File.Exists(path)) { File.Delete(path); return true; }
        return false;
    }

    // ── backup directories ───────────────────────────────────────────────

    public static string BackupsSavesDir()
    {
        var p = Path.Combine(GetBackupRoot(), "backups", "saves");
        Directory.CreateDirectory(p);
        return p;
    }

    public static string BackupsConfigDir()
    {
        var p = Path.Combine(GetBackupRoot(), "backups", "ue5");
        Directory.CreateDirectory(p);
        return p;
    }

    // ── registry persistence ─────────────────────────────────────────────

    public record ConfigValues(
        string? SaveFolder,
        string? BackupRoot);

    /// Read config from HKCU\Software\NotAlterra. Expands %LOCALAPPDATA%, %USERPROFILE%.
    public static ConfigValues LoadAppConfig()
    {
        using var key = Registry.CurrentUser.OpenSubKey(RegKey);
        if (key == null) return new ConfigValues(null, null);
        var localAppData = Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData);
        var userProfile = Environment.GetFolderPath(Environment.SpecialFolder.UserProfile);
        return new ConfigValues(
            (key.GetValue("SaveFolder") as string)?.Replace("%LOCALAPPDATA%", localAppData).Replace("%USERPROFILE%", userProfile),
            (key.GetValue("BackupRoot") as string)?.Replace("%LOCALAPPDATA%", localAppData).Replace("%USERPROFILE%", userProfile)
        );
    }

    /// Write config to HKCU\Software\NotAlterra. Stores paths with %LOCALAPPDATA%, %USERPROFILE% for portability.
    public static void SaveAppConfig(string? saveFolder, string? backupRoot)
    {
        using var key = Registry.CurrentUser.CreateSubKey(RegKey);
        var localAppData = Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData);
        var userProfile = Environment.GetFolderPath(Environment.SpecialFolder.UserProfile);
        if (saveFolder != null) key.SetValue("SaveFolder", saveFolder.Replace(localAppData, "%LOCALAPPDATA%").Replace(userProfile, "%USERPROFILE%"));
        if (backupRoot != null) key.SetValue("BackupRoot", backupRoot.Replace(localAppData, "%LOCALAPPDATA%").Replace(userProfile, "%USERPROFILE%"));
    }

    public static void EnsureDir(string path)
    {
        Directory.CreateDirectory(path);
    }
}
