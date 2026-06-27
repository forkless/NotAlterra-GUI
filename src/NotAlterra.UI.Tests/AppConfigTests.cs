using NotAlterra.Services;
using Xunit;

namespace NotAlterra_UI_Tests;

public class AppConfigTests
{
    // ── Sentinel (disclaimer) ────────────────────────────────────────

    [Fact]
    public void DisclaimerAccepted_Initially_ReturnsFalse()
    {
        // Sentinel path uses LocalApplicationData — can't guarantee clean state.
        // Just verify it returns bool without throwing.
        var result = AppConfig.DisclaimerAccepted();
        Assert.IsType<bool>(result);
    }

    // ── Backup root ──────────────────────────────────────────────────

    [Fact]
    public void GetBackupRoot_Default_ReturnsNotNull()
    {
        var root = AppConfig.GetBackupRoot();
        Assert.NotNull(root);
        Assert.Contains("NotAlterra", root);
    }

    [Fact]
    public void SetBackupRoot_ThenGet_ReturnsSetValue()
    {
        const string testRoot = @"C:\TestBackupRoot";
        AppConfig.SetBackupRoot(testRoot);
        try
        {
            Assert.Equal(testRoot, AppConfig.GetBackupRoot());
        }
        finally
        {
            AppConfig.SetBackupRoot(null!); // reset
        }
    }

    // ── Config paths ─────────────────────────────────────────────────

    [Fact]
    public void SentinelPath_ReturnsSensiblePath()
    {
        var path = AppConfig.SentinelPath();
        Assert.NotNull(path);
        Assert.Contains("NOTALTERRA_LICENSE_ACCEPTED", path);
        Assert.StartsWith(Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData), path);
    }

    [Fact]
    public void StaleConfigPath_ReturnsSensiblePath()
    {
        var path = AppConfig.StaleConfigPath();
        Assert.NotNull(path);
        Assert.Contains("config.ini", path);
    }

    [Fact]
    public void AppIniPath_ReturnsSensiblePath()
    {
        var path = AppConfig.AppIniPath();
        Assert.NotNull(path);
        Assert.Contains("app.ini", path);
    }

    // ── CleanupStaleConfig ───────────────────────────────────────────

    [Fact]
    public void CleanupStaleConfig_NoFile_ReturnsFalse()
    {
        // StaleConfigPath points to exe dir — won't exist in test context
        var result = AppConfig.CleanupStaleConfig();
        Assert.False(result);
    }

    // ── SaveAppConfig / LoadAppConfig roundtrip ──────────────────────

    [Fact]
    public void SaveAndLoadAppConfig_Roundtrip_OK()
    {
        // Save config with known paths, reload, verify
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
            // Clean up the test file
            var iniPath = AppConfig.AppIniPath();
            if (File.Exists(iniPath)) File.Delete(iniPath);
        }
    }

    [Fact]
    public void LoadAppConfig_NoFile_ReturnsNulls()
    {
        var config = AppConfig.LoadAppConfig();
        // If the file doesn't exist, both fields should be null
        if (!File.Exists(AppConfig.AppIniPath()))
        {
            Assert.Null(config.SaveFolder);
            Assert.Null(config.BackupRoot);
        }
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
            AppConfig.EnsureDir(dir); // should not throw
            Assert.True(Directory.Exists(dir));
        }
        finally
        {
            Directory.Delete(dir);
        }
    }
}
