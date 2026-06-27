// Game-running guard and transaction logging.
// Ported from src/guard.rs
// Process detection uses Process.GetProcessesByName (Win32 API, no shell exec)
// to avoid Defender false positives from tasklist.exe.

using System.Diagnostics;

namespace NotAlterra.Services;

public static class Guard
{
    // ── process detection ────────────────────────────────────────────────

    /// Process names Subnautica 2 uses.
    private static readonly string[] GameProcessNames =
        ["Subnautica2", "Subnautica2-Win64-Shipping"];

    /// Set to false if Windows Defender flags the detection.
    /// Users can toggle this in Settings.
    public static bool ProcessDetectionEnabled { get; set; } = true;

    /// Returns true if Subnautica 2 appears to be running.
    /// Uses Process.GetProcessesByName (internal CreateToolhelp32Snapshot)
    /// — no shell exec, no command parsing, no spawned processes for Defender to flag.
    /// Check both Subnautica2 and Subnautica2-Win64-Shipping for all platforms.
    public static bool GameRunning()
    {
        if (!ProcessDetectionEnabled) return false;

        foreach (var name in GameProcessNames)
        {
            var processes = Process.GetProcessesByName(name);
            if (processes.Length > 0) return true;
        }
        return false;
    }

    // ── transaction logging ──────────────────────────────────────────────

    /// Path to transaction.log inside the logs/ directory.
    private static string LogPath()
    {
        var dir = Path.Combine(
            Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData),
            "NotAlterra", "logs");
        Directory.CreateDirectory(dir);
        return Path.Combine(dir, "transaction.log");
    }

    /// Maximum lines before rotation.
    private const int MaxLogLines = 10_000;

    /// Append a timestamped log entry.
    /// Format: `YYYY-MM-DD HH:MM:SS | ACTION | detail | result`
    public static void LogAction(string action, string detail, string result)
    {
        var logPath = LogPath();
        var stamp = DateTime.Now.ToString("yyyy-MM-dd HH:mm:ss");
        var line = $"{stamp} | {action,-8} | {detail} | {result}\n";

        // Rotate if needed
        if (File.Exists(logPath))
        {
            var content = File.ReadAllText(logPath);
            var lines = content.Split('\n');
            if (lines.Length > MaxLogLines)
            {
                var keep = string.Join("\n", lines[^MaxLogLines..]);
                File.WriteAllText(logPath, keep + "\n");
            }
        }

        File.AppendAllText(logPath, line);
    }

    /// Truncate a path to start at "Subnautica2/", stripping user prefix.
    public static string SanitizePath(string path)
    {
        const string needle = "Subnautica2";
        var sep = path.Contains('\\') ? "\\" : "/";
        var idx = path.IndexOf(needle, StringComparison.OrdinalIgnoreCase);
        return idx >= 0 ? $"...{sep}{path[idx..]}" : path;
    }

    /// Check whether a path looks like a network/UNC path.
    public static bool IsNetworkPath(string path)
    {
        return path.StartsWith("\\\\") || path.StartsWith("//");
    }

    // ── migration ────────────────────────────────────────────────────────

    /// Migrate old transaction.log (next to exe) into logs/ directory.
    public static bool MigrateOldLog()
    {
        var old = Path.Combine(AppDomain.CurrentDomain.BaseDirectory, "transaction.log");
        if (!File.Exists(old)) return false;

        var content = File.ReadAllText(old);
        if (!string.IsNullOrWhiteSpace(content))
        {
            var header = $"───── migrated from old location [{DateTime.Now:yyyy-MM-dd HH:mm:ss}] ─────\n";
            File.AppendAllText(LogPath(), header + content);
        }

        var migrated = old + ".migrated";
        if (File.Exists(migrated)) File.Delete(migrated);
        File.Move(old, migrated);
        return true;
    }
}
