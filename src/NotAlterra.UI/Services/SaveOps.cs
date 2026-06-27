// File operations: recover .bak→.sav, backup/restore, .ini management.
// Ported from src/ops.rs

using System.Formats.Tar;
using System.IO.Compression;
using System.Security.Cryptography;

namespace NotAlterra.Services;

public static class SaveOps
{
    // ── types ────────────────────────────────────────────────────────────

    public record BackupResult(
        int FilesCopied,
        long TotalSize,
        string DestPath,
        bool Verified);

    public record RecoveryResult(
        string Source,
        string Target,
        string? OldSavedAs);

    // ── .sav recovery from .bak ──────────────────────────────────────────

    public static RecoveryResult RecoverBakToSav(string saveFolder, string bakFilename)
    {
        var bakPath = Path.Combine(saveFolder, bakFilename);
        if (!File.Exists(bakPath))
            throw new FileNotFoundException($"Backup file not found: {bakPath}");

        var fi = new FileInfo(bakPath);
        if (fi.Length < 1024)
            throw new InvalidDataException($"Backup too small ({fi.Length} bytes)");

        var slot = Gvas.SlotUtils.DeriveSlotFromFilename(bakFilename)
            ?? throw new InvalidDataException($"Cannot derive slot from: {bakFilename}");

        var targetName = $"{slot}.sav";
        var targetPath = Path.Combine(saveFolder, targetName);
        string? oldSavedAs = null;

        if (File.Exists(targetPath))
        {
            var oldPath = Path.Combine(saveFolder, $"{targetName}.old");
            if (File.Exists(oldPath)) File.Delete(oldPath);
            File.Move(targetPath, oldPath);
            oldSavedAs = $"{targetName}.old";
        }

        File.Copy(bakPath, targetPath, overwrite: true);
        return new RecoveryResult(bakFilename, targetName, oldSavedAs);
    }

    // ── tar.gz helpers ──────────────────────────────────────────────────

    private static (int Count, long TotalSize, string Path) CreateTarGz(
        string srcDir, string destDir, string prefix, string name)
    {
        var ts = DateTime.Now.ToString("yyyy-MM-dd_HHmmss_fff");
        var archiveName = $"{name}_{ts}.tar.gz";
        var archivePath = System.IO.Path.Combine(destDir, archiveName);
        Directory.CreateDirectory(destDir);

        var entries = Directory.EnumerateFiles(srcDir)
            .Where(f => Path.GetFileName(f).StartsWith(prefix) ||
                        Path.GetFileName(f).StartsWith("savegame_"))
            .OrderBy(f => f)
            .ToList();

        // Build manifest
        var manifestLines = new List<string>();
        foreach (var entry in entries)
        {
            var fi = new FileInfo(entry);
            manifestLines.Add($"{fi.Length,12}  {fi.Name}");
        }
        var manifest = string.Join("\n", manifestLines) + "\n";

        using var fs = File.Create(archivePath);
        using var gz = new GZipStream(fs, CompressionLevel.SmallestSize);
        using var writer = new TarWriter(gz, TarEntryFormat.Pax, leaveOpen: false);

        // Write MANIFEST
        var manifestEntry = new PaxTarEntry(TarEntryType.RegularFile, "MANIFEST");
        manifestEntry.DataStream = new MemoryStream(System.Text.Encoding.UTF8.GetBytes(manifest));
        writer.WriteEntry(manifestEntry);

        long total = 0;
        foreach (var entry in entries)
        {
            var fi = new FileInfo(entry);
            var data = File.ReadAllBytes(entry);
            var tarEntry = new PaxTarEntry(TarEntryType.RegularFile, fi.Name);
            tarEntry.DataStream = new MemoryStream(data);
            tarEntry.ModificationTime = fi.LastWriteTimeUtc;
            writer.WriteEntry(tarEntry);
            total += fi.Length;
        }

        writer.Dispose();
        return (entries.Count + 1, total, archivePath);
    }

    private static int ExtractTarGz(string archivePath, string destDir)
    {
        Directory.CreateDirectory(destDir);
        using var fs = File.OpenRead(archivePath);
        using var gz = new GZipStream(fs, CompressionMode.Decompress);
        using var reader = new TarReader(gz);

        int count = 0;
        while (reader.GetNextEntry() is { } entry)
        {
            if (entry.Name == "MANIFEST") continue;
            var dest = System.IO.Path.Combine(destDir, entry.Name);
            entry.ExtractToFile(dest, overwrite: true);
            count++;
        }
        return count;
    }

    /// Quick integrity check: gzip magic bytes.
    public static bool CheckTarGzIntegrity(string path)
    {
        try
        {
            using var fs = File.OpenRead(path);
            if (fs.Length < 20) return false;
            var magic = new byte[2];
            fs.ReadExactly(magic, 0, 2);
            return magic[0] == 0x1F && magic[1] == 0x8B;
        }
        catch { return false; }
    }

    /// List tar.gz files in a directory, sorted by mtime descending.
    private static List<string> ListTarGz(string dir)
    {
        if (!Directory.Exists(dir)) return [];
        return Directory.EnumerateFiles(dir, "*.tar.gz")
            .OrderByDescending(f => new FileInfo(f).LastWriteTimeUtc)
            .ToList();
    }

    // ── full backup ─────────────────────────────────────────────────────

    public static BackupResult CreateFullBackup(string saveFolder)
    {
        var backupDir = AppConfig.BackupsSavesDir();
        var (count, total, path) = CreateTarGz(saveFolder, backupDir, "savegame_", "snapshot");
        return new BackupResult(count, total, path, File.Exists(path));
    }

    public static int RestoreFullBackup(string archivePath, string saveFolder)
    {
        // Pre-restore safety: backup current saves
        try { CreateTarGz(saveFolder, AppConfig.BackupsSavesDir(), "savegame_", "pre_restore"); }
        catch { /* non-fatal */ }

        return ExtractTarGz(archivePath, saveFolder);
    }

    // ── .ini management ──────────────────────────────────────────────────

    public static BackupResult BackupIniFiles(string configPath)
    {
        if (!Directory.Exists(configPath) || !Directory.EnumerateFiles(configPath, "*.ini").Any())
            throw new InvalidDataException("No .ini files found in config path");

        var backupDir = AppConfig.BackupsConfigDir();
        var (count, total, path) = CreateTarGz(configPath, backupDir, "", "ini");
        return new BackupResult(count, total, path, File.Exists(path));
    }

    public static int RestoreIniFiles(string archivePath, string configPath)
    {
        try { BackupIniFiles(configPath); } catch { /* non-fatal */ }
        return ExtractTarGz(archivePath, configPath);
    }

    public static int DeleteIniFiles(string configPath)
    {
        int deleted = 0;
        foreach (var f in Directory.EnumerateFiles(configPath, "*.ini"))
        {
            File.Delete(f); deleted++;
        }
        return deleted;
    }

    // ── listing ──────────────────────────────────────────────────────────

    public static List<string> ListFullBackups() => ListTarGz(AppConfig.BackupsSavesDir());
    public static List<string> ListIniBackups() => ListTarGz(AppConfig.BackupsConfigDir());

    /// List .bak files in save folder, sorted by mtime descending.
    public static List<string> ListBakFiles(string saveFolder)
    {
        if (!Directory.Exists(saveFolder)) return [];
        return Directory.EnumerateFiles(saveFolder, "*.bak")
            .OrderByDescending(f => new FileInfo(f).LastWriteTimeUtc)
            .ToList();
    }

    /// Enriched .bak file entry with GVAS metadata.
    public record BakFileSummary(
        string Path,
        string Filename,
        string Slot,
        string? DisplayName,
        bool IsOnline,
        long Size,
        string? Mtime,
        double? PlaytimeSeconds);

    /// List .bak files with parsed GVAS metadata.
    public static List<BakFileSummary> ListBakFilesWithMeta(string saveFolder)
    {
        var results = new List<BakFileSummary>();
        if (!Directory.Exists(saveFolder)) return results;

        foreach (var file in Directory.EnumerateFiles(saveFolder, "*.bak"))
        {
            var fi = new FileInfo(file);
            var filename = fi.Name;
            var slot = Gvas.SlotUtils.DeriveSlotFromFilename(filename) ?? "?";

            try
            {
                var meta = Gvas.GvasParser.ExtractMetadata(file);
                var mtime = fi.LastWriteTimeUtc.Year > 1970
                    ? fi.LastWriteTimeUtc.ToString("yyyy-MMM-dd HH:mm")
                    : null;

                results.Add(new BakFileSummary(
                    file, filename, slot,
                    meta.DisplayName, meta.IsOnline,
                    fi.Length, mtime, meta.PlaytimeSeconds));
            }
            catch
            {
                results.Add(new BakFileSummary(
                    file, filename, slot,
                    null, false, fi.Length, null, null));
            }
        }

        return results.OrderBy(r => r.Slot).ThenByDescending(r => r.Mtime).ToList();
    }

    // ── dedup by slot ────────────────────────────────────────────────────

    public static List<BakFileSummary> DedupBySlot(List<BakFileSummary> files)
    {
        var seen = new HashSet<string>();
        return files.Where(f => seen.Add(f.Slot)).ToList();
    }

    // ── folder stats ─────────────────────────────────────────────────────

    /// Scan save folder and return (liveSaves, bakSaves, hasIniBackup).
    public static (int Live, int Bak, bool HasIniBackup) FolderStats(string? saveFolder)
    {
        int live = 0, bak = 0;
        if (saveFolder != null && Directory.Exists(saveFolder))
        {
            foreach (var f in Directory.EnumerateFiles(saveFolder))
            {
                var name = Path.GetFileName(f);
                if (name.StartsWith("savegame_") && name.EndsWith(".sav")) live++;
                else if (name.StartsWith("savegame_") && name.EndsWith(".bak")) bak++;
            }
        }

        bool hasIniBackup = false;
        var configDir = AppConfig.BackupsConfigDir();
        if (Directory.Exists(configDir))
        {
            hasIniBackup = Directory.EnumerateFiles(configDir, "*.tar.gz").Any();
        }

        AppConfig.BackupsSavesDir(); // ensure exists
        return (live, bak, hasIniBackup);
    }

    // ── migration ────────────────────────────────────────────────────────

    /// Migrate old NotAlterra_Backups/ directory-tree backups into tar.gz.
    public static int MigrateOldBackups()
    {
        var oldRoot = Path.Combine(AppDomain.CurrentDomain.BaseDirectory, "NotAlterra_Backups");
        if (!Directory.Exists(oldRoot)) return 0;

        int migrated = 0;
        foreach (var dir in Directory.EnumerateDirectories(oldRoot))
        {
            var dirName = Path.GetFileName(dir);

            if (dirName.StartsWith("notalterra_copy_"))
            {
                var hasSaves = Directory.EnumerateFiles(dir, "savegame_*").Any();
                if (!hasSaves) continue;

                try
                {
                    var (_, _, path) = CreateTarGz(dir, AppConfig.BackupsSavesDir(),
                        "savegame_", $"migrated_{dirName}");
                    if (File.Exists(path)) migrated++;
                }
                catch { }
            }
            else if (dirName.StartsWith("config_"))
            {
                try
                {
                    var (_, _, path) = CreateTarGz(dir, AppConfig.BackupsConfigDir(),
                        "", $"migrated_{dirName}");
                    if (File.Exists(path)) migrated++;
                }
                catch { }
            }
        }
        return migrated;
    }
}
