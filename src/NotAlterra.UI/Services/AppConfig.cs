// Path utilities and persistent app configuration.
// Ported from src/config.rs

using System.Text;

namespace NotAlterra.Services;

public static class AppConfig
{
    // ── backup root ──────────────────────────────────────────────────────

    private static string? _backupRoot;

    public static void SetBackupRoot(string path)
    {
        _backupRoot = path;
    }

    public static string GetBackupRoot()
    {
        return _backupRoot ?? Path.Combine(
            Environment.GetFolderPath(Environment.SpecialFolder.UserProfile),
            "NotAlterra");
    }

    // ── config base directory ────────────────────────────────────────────

    /// Fixed base dir for app.ini and sentinel file.
    /// Separate from backup data so ~/NotAlterra stays visible.
    private static string ConfigBaseDir()
    {
        return Path.Combine(
            Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData),
            "NotAlterra", "config");
    }

    // ── sentinel (disclaimer acceptance) ─────────────────────────────────

    public static string SentinelPath()
    {
        return Path.Combine(ConfigBaseDir(), "NOTALTERRA_LICENSE_ACCEPTED");
    }

    public static bool DisclaimerAccepted()
    {
        return File.Exists(SentinelPath());
    }

    public static void AcceptDisclaimer()
    {
        var path = SentinelPath();
        Directory.CreateDirectory(Path.GetDirectoryName(path)!);
        File.WriteAllBytes(path, []);
    }

    public static void RejectDisclaimer()
    {
        var path = SentinelPath();
        if (File.Exists(path)) File.Delete(path);
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

    /// Path to backups/saves/ under backup root. Auto-creates.
    public static string BackupsSavesDir()
    {
        var p = Path.Combine(GetBackupRoot(), "backups", "saves");
        Directory.CreateDirectory(p);
        return p;
    }

    /// Path to backups/ue5/ under backup root. Auto-creates.
    public static string BackupsConfigDir()
    {
        var p = Path.Combine(GetBackupRoot(), "backups", "ue5");
        Directory.CreateDirectory(p);
        return p;
    }

    // ── app.ini persistence ──────────────────────────────────────────────

    public static string AppIniPath()
    {
        var dir = ConfigBaseDir();
        Directory.CreateDirectory(dir);
        return Path.Combine(dir, "app.ini");
    }

    /// Session config loaded from app.ini.
    public record ConfigValues(
        string? SaveFolder,
        string? BackupRoot);

    /// Load app.ini. Returns default if file missing.
    public static ConfigValues LoadAppConfig()
    {
        var path = AppIniPath();
        if (!File.Exists(path)) return new ConfigValues(null, null);

        string? saveFolder = null, backupRoot = null;
        foreach (var line in File.ReadAllLines(path))
        {
            var trimmed = line.Trim();
            if (trimmed.Length == 0 || trimmed.StartsWith('#')) continue;
            var eq = trimmed.IndexOf('=');
            if (eq < 0) continue;
            var key = trimmed[..eq].Trim();
            var val = trimmed[(eq + 1)..].Trim();
            if (key == "save_folder") saveFolder = val;
            else if (key == "backup_root") backupRoot = val;
        }
        return new ConfigValues(saveFolder, backupRoot);
    }

    /// Write current session paths to app.ini.
    public static void SaveAppConfig(string? saveFolder, string? backupRoot)
    {
        var sb = new StringBuilder();
        sb.AppendLine("# NotAlterra configuration");
        sb.AppendLine("# This file is auto-generated. Edit while the tool is not running.");
        sb.AppendLine();
        if (saveFolder != null) sb.AppendLine($"save_folder = {saveFolder}");
        if (backupRoot != null) sb.AppendLine($"backup_root = {backupRoot}");
        File.WriteAllText(AppIniPath(), sb.ToString());
    }

    /// Ensure a directory exists.
    public static void EnsureDir(string path)
    {
        Directory.CreateDirectory(path);
    }
}
