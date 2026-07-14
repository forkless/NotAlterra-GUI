using Microsoft.Win32;
using NotAlterra.Services;
using Xunit;

namespace NotAlterra_UI_Tests;

public class AppConfigTests
{
    private const string TestRegKey = @"Software\NotAlterra";

    private static void CleanRegistry()
    {
        try { Registry.CurrentUser.DeleteSubKeyTree(TestRegKey, throwOnMissingSubKey: false); } catch { }
    }

    // ── Backup root ──────────────────────────────────────────────────

    [Fact]
    public void GetBackupRoot_Default_ReturnsNotNull()
    {
        var root = AppConfig.GetBackupRoot();
        Assert.NotNull(root);
        Assert.Contains("NotAlterra", root);
    }

    // ── Config paths ─────────────────────────────────────────────────

    [Fact]
    public void StaleConfigPath_ReturnsSensiblePath()
    {
        var path = AppConfig.StaleConfigPath();
        Assert.NotNull(path);
        Assert.Contains("config.ini", path);
    }

    // ── CleanupStaleConfig ───────────────────────────────────────────

    [Fact]
    public void CleanupStaleConfig_NoFile_ReturnsFalse()
    {
        var result = AppConfig.CleanupStaleConfig();
        Assert.False(result);
    }

    // ── SaveAppConfig / LoadAppConfig roundtrip ──────────────────────

    [Fact]
    public void SaveAndLoadAppConfig_Roundtrip_OK()
    {
        CleanRegistry();
        const string testSaveFolder = @"C:\TestSaves\Subnautica2\SaveGames";
        const string testBackupRoot = @"C:\TestNotAlterraBackups";

        AppConfig.SaveAppConfig(testSaveFolder, testBackupRoot);
        try
        {
            var loaded = AppConfig.LoadAppConfig();
            Assert.NotNull(loaded.SaveFolder);
            Assert.NotNull(loaded.BackupRoot);
            Assert.Contains(testSaveFolder, loaded.SaveFolder);
            Assert.Contains(testBackupRoot, loaded.BackupRoot);
        }
        finally
        {
            CleanRegistry();
        }
    }

    [Fact]
    public void LoadAppConfig_NoKey_ReturnsNulls()
    {
        CleanRegistry();
        var config = AppConfig.LoadAppConfig();
        Assert.Null(config.SaveFolder);
        Assert.Null(config.BackupRoot);
    }

    // ── EnsureDir ────────────────────────────────────────────────────

    [Fact]
    public void EnsureDir_CreatesDirectory()
    {
        var dir = Path.Combine(Path.GetTempPath(), Guid.NewGuid().ToString());
        try
        {
            Assert.False(Directory.Exists(dir));
            AppConfig.EnsureDir(dir);
            Assert.True(Directory.Exists(dir));
        }
        finally
        {
            if (Directory.Exists(dir)) Directory.Delete(dir);
        }
    }

    [Fact]
    public void EnsureDir_ExistingDir_NoError()
    {
        var dir = Path.Combine(Path.GetTempPath(), Guid.NewGuid().ToString());
        Directory.CreateDirectory(dir);
        try
        {
            AppConfig.EnsureDir(dir);
            Assert.True(Directory.Exists(dir));
        }
        finally
        {
            Directory.Delete(dir);
        }
    }
}
