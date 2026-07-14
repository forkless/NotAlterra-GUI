using Microsoft.Win32;
using NotAlterra.Services;
using Xunit;

namespace NotAlterra_UI_Tests;

public class SaveOpsTests
{
    // ── DedupBySlot ──────────────────────────────────────────────────

    [Fact]
    public void DedupBySlot_EmptyList_ReturnsEmpty()
    {
        var result = SaveOps.DedupBySlot([]);
        Assert.Empty(result);
    }

    [Fact]
    public void DedupBySlot_NoDuplicates_ReturnsAll()
    {
        var files = new List<SaveOps.BakFileSummary>
        {
            new("path1", "savegame_0_1.bak", "savegame_0", "Spoonmore", true, 100, "2025-Jan-01", 3600),
            new("path2", "savegame_1.bak", "savegame_1", "Test", false, 200, "2025-Jan-02", null),
            new("path3", "savegame_2.bak", "savegame_2", null, false, 300, null, null),
        };
        var result = SaveOps.DedupBySlot(files);
        Assert.Equal(3, result.Count);
    }

    [Fact]
    public void DedupBySlot_KeepsFirstPerSlot()
    {
        var files = new List<SaveOps.BakFileSummary>
        {
            new("pathA", "savegame_0_1.bak", "savegame_0", "First", true, 100, "A", null),
            new("pathB", "savegame_0_2.bak", "savegame_0", "Second", true, 200, "B", null),
            new("pathC", "savegame_0_3.bak", "savegame_0", "Third", true, 300, "C", null),
        };
        var result = SaveOps.DedupBySlot(files);
        Assert.Single(result);
        Assert.Equal("First", result[0].DisplayName);
        Assert.Equal("pathA", result[0].Path);
    }

    [Fact]
    public void DedupBySlot_MixedSlots_DeduplicatesPerSlot()
    {
        var files = new List<SaveOps.BakFileSummary>
        {
            new("pA", "sg0_1.bak", "savegame_0", "A", false, 1, null, null),
            new("pB", "sg0_2.bak", "savegame_0", "B", false, 2, null, null),
            new("pC", "sg1_1.bak", "savegame_1", "C", false, 3, null, null),
            new("pD", "sg1_2.bak", "savegame_1", "D", false, 4, null, null),
        };
        var result = SaveOps.DedupBySlot(files);
        Assert.Equal(2, result.Count);
        Assert.Equal("A", result[0].DisplayName); // first sg0
        Assert.Equal("C", result[1].DisplayName); // first sg1
    }

    // ── CheckTarGzIntegrity ──────────────────────────────────────────

    [Fact]
    public void CheckTarGzIntegrity_NonExistentFile_ReturnsFalse()
    {
        var path = Path.Combine(Path.GetTempPath(), Guid.NewGuid() + ".tar.gz");
        Assert.False(SaveOps.CheckTarGzIntegrity(path));
    }

    [Fact]
    public void CheckTarGzIntegrity_TooSmall_ReturnsFalse()
    {
        var path = Path.GetTempFileName();
        try
        {
            File.WriteAllBytes(path, [0x1F, 0x8B]); // valid magic but too small
            Assert.False(SaveOps.CheckTarGzIntegrity(path));
        }
        finally
        {
            File.Delete(path);
        }
    }

    [Fact]
    public void CheckTarGzIntegrity_BadMagic_ReturnsFalse()
    {
        var path = Path.GetTempFileName();
        try
        {
            File.WriteAllBytes(path, new byte[100]); // no gzip magic
            Assert.False(SaveOps.CheckTarGzIntegrity(path));
        }
        finally
        {
            File.Delete(path);
        }
    }

    // ── RecoverBakToSav ──────────────────────────────────────────────

    [Fact]
    public void RecoverBakToSav_NonExistentBak_Throws()
    {
        var dir = Path.Combine(Path.GetTempPath(), Guid.NewGuid().ToString());
        Assert.Throws<FileNotFoundException>(() =>
            SaveOps.RecoverBakToSav(dir, "savegame_99.bak"));
    }

    [Fact]
    public void RecoverBakToSav_TooSmall_Throws()
    {
        var dir = Path.Combine(Path.GetTempPath(), Guid.NewGuid().ToString());
        Directory.CreateDirectory(dir);
        try
        {
            var bakPath = Path.Combine(dir, "savegame_0.bak");
            File.WriteAllBytes(bakPath, new byte[100]); // under 1024
            Assert.Throws<InvalidDataException>(() =>
                SaveOps.RecoverBakToSav(dir, "savegame_0.bak"));
        }
        finally
        {
            Directory.Delete(dir, recursive: true);
        }
    }

    [Fact]
    public void RecoverBakToSav_InvalidFilename_Throws()
    {
        var dir = Path.Combine(Path.GetTempPath(), Guid.NewGuid().ToString());
        Directory.CreateDirectory(dir);
        try
        {
            var bakPath = Path.Combine(dir, "nope.bak");
            File.WriteAllBytes(bakPath, new byte[2000]);
            Assert.Throws<InvalidDataException>(() =>
                SaveOps.RecoverBakToSav(dir, "nope.bak"));
        }
        finally
        {
            Directory.Delete(dir, recursive: true);
        }
    }

    // ── FolderStats ──────────────────────────────────────────────────

    [Fact]
    public void FolderStats_NullDir_ReturnsZeroes()
    {
        var (live, bak, hasIni) = SaveOps.FolderStats(null);
        Assert.Equal(0, live);
        Assert.Equal(0, bak);
        Assert.IsType<bool>(hasIni); // just structural check, value depends on state
    }

    [Fact]
    public void FolderStats_NonExistentDir_ReturnsZeroes()
    {
        var dir = Path.Combine(Path.GetTempPath(), Guid.NewGuid().ToString());
        var (live, bak, _) = SaveOps.FolderStats(dir);
        Assert.Equal(0, live);
        Assert.Equal(0, bak);
    }

    // ── RecoverBakToSav success ───────────────────────────────────────

    [Fact]
    public void RecoverBakToSav_Success_CreatesSavFromBak()
    {
        var dir = Path.Combine(Path.GetTempPath(), Guid.NewGuid().ToString());
        Directory.CreateDirectory(dir);
        try
        {
            var bakPath = Path.Combine(dir, "savegame_3.bak");
            var content = new byte[2048];
            new Random(42).NextBytes(content);
            File.WriteAllBytes(bakPath, content);

            var result = SaveOps.RecoverBakToSav(dir, "savegame_3.bak");

            Assert.Equal("savegame_3.bak", result.Source);
            Assert.Equal("savegame_3.sav", result.Target);
            Assert.Null(result.OldSavedAs); // no existing .sav

            var savPath = Path.Combine(dir, "savegame_3.sav");
            Assert.True(File.Exists(savPath));
            Assert.Equal(content, File.ReadAllBytes(savPath));
        }
        finally
        {
            Directory.Delete(dir, recursive: true);
        }
    }

    [Fact]
    public void RecoverBakToSav_Success_BacksUpExistingSav()
    {
        var dir = Path.Combine(Path.GetTempPath(), Guid.NewGuid().ToString());
        Directory.CreateDirectory(dir);
        try
        {
            // Create existing .sav
            var savPath = Path.Combine(dir, "savegame_0.sav");
            File.WriteAllBytes(savPath, new byte[1500]);

            // Create .bak to recover
            var bakPath = Path.Combine(dir, "savegame_0.bak");
            var bakContent = new byte[2048];
            new Random(99).NextBytes(bakContent);
            File.WriteAllBytes(bakPath, bakContent);

            var result = SaveOps.RecoverBakToSav(dir, "savegame_0.bak");

            Assert.Equal("savegame_0.bak", result.Source);
            Assert.Equal("savegame_0.sav", result.Target);
            Assert.Equal("savegame_0.sav.old", result.OldSavedAs);

            // New .sav should be bak content
            Assert.Equal(bakContent, File.ReadAllBytes(savPath));

            // Old .sav saved as .old
            Assert.True(File.Exists(Path.Combine(dir, "savegame_0.sav.old")));
        }
        finally
        {
            Directory.Delete(dir, recursive: true);
        }
    }

    // ── tar.gz create/verify/restore roundtrip ────────────────────────

    private const string TestRegKey = @"Software\NotAlterra";

    private static void CleanTestRegistry()
    {
        try { Registry.CurrentUser.DeleteSubKeyTree(TestRegKey, throwOnMissingSubKey: false); } catch { }
    }

    [Fact]
    public void Backup_Roundtrip_CreateVerifyRestore()
    {
        // Use a temp backup root via registry so we don't pollute real profile
        var backupRoot = Path.Combine(Path.GetTempPath(), Guid.NewGuid().ToString());
        var saveDir = Path.Combine(Path.GetTempPath(), Guid.NewGuid().ToString());
        var restoreDir = Path.Combine(Path.GetTempPath(), Guid.NewGuid().ToString());

        Directory.CreateDirectory(saveDir);
        Directory.CreateDirectory(restoreDir);

        // Seed save folder with test save files
        File.WriteAllBytes(Path.Combine(saveDir, "savegame_0.sav"), new byte[2048]);
        File.WriteAllBytes(Path.Combine(saveDir, "savegame_1.sav"), new byte[4096]);

        CleanTestRegistry();
        try
        {
            AppConfig.SaveAppConfig(null, backupRoot);

            // 1. Create backup
            var backupResult = SaveOps.CreateFullBackup(saveDir);
            Assert.True(backupResult.FilesCopied > 0, "Should copy at least MANIFEST + save files");
            Assert.True(backupResult.TotalSize > 0, "Backup should have content");
            Assert.True(File.Exists(backupResult.DestPath), $"Backup file should exist at {backupResult.DestPath}");
            Assert.True(backupResult.Verified, "Backup should be marked verified");

            // 2. Verify tar.gz integrity
            var quickCheck = SaveOps.CheckTarGzIntegrity(backupResult.DestPath);
            Assert.True(quickCheck, "Quick integrity check should pass");

            var fullCheck = SaveOps.VerifyTarGzIntegrity(backupResult.DestPath);
            Assert.True(fullCheck.Ok, $"Full integrity check should pass: {fullCheck.Details}");

            // 3. Read manifest
            var manifest = SaveOps.ReadTarGzManifest(backupResult.DestPath);
            Assert.Contains("savegame_0.sav", manifest);
            Assert.Contains("savegame_1.sav", manifest);

            // 4. Restore to clean directory
            var restored = SaveOps.RestoreFullBackup(backupResult.DestPath, restoreDir);
            Assert.Equal(2, restored);

            Assert.True(File.Exists(Path.Combine(restoreDir, "savegame_0.sav")));
            Assert.True(File.Exists(Path.Combine(restoreDir, "savegame_1.sav")));
        }
        finally
        {
            CleanTestRegistry();
            try { Directory.Delete(backupRoot, recursive: true); } catch { }
            try { Directory.Delete(saveDir, recursive: true); } catch { }
            try { Directory.Delete(restoreDir, recursive: true); } catch { }
        }
    }

    // ── FolderStats real scenario ─────────────────────────────────────

    [Fact]
    public void FolderStats_WithRealFiles_CountsCorrectly()
    {
        var dir = Path.Combine(Path.GetTempPath(), Guid.NewGuid().ToString());
        Directory.CreateDirectory(dir);
        try
        {
            File.WriteAllBytes(Path.Combine(dir, "savegame_0.sav"), new byte[100]);
            File.WriteAllBytes(Path.Combine(dir, "savegame_1.sav"), new byte[200]);
            File.WriteAllBytes(Path.Combine(dir, "savegame_0.bak"), new byte[300]);
            File.WriteAllBytes(Path.Combine(dir, "savegame_0_1.bak"), new byte[400]);
            // Non-save files should be ignored
            File.WriteAllBytes(Path.Combine(dir, "random.txt"), new byte[50]);

            var (live, bak, _) = SaveOps.FolderStats(dir);
            Assert.Equal(2, live);
            Assert.Equal(2, bak);
        }
        finally
        {
            Directory.Delete(dir, recursive: true);
        }
    }
}
