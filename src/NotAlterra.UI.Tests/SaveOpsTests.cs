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
}
