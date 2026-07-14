using NotAlterra.Services;
using Xunit;

namespace NotAlterra_UI_Tests;

public class GuardTests
{
    // ── SanitizePath ─────────────────────────────────────────────────

    [Theory]
    [InlineData(@"C:\Users\Bob\AppData\Local\Subnautica2\Saved\SaveGames\savegame_0.sav",
                 @"...\Subnautica2\Saved\SaveGames\savegame_0.sav")]
    [InlineData(@"/home/bob/.config/Subnautica2/Saved/savegame_0.sav",
                 @".../Subnautica2/Saved/savegame_0.sav")]
    public void SanitizePath_TrimsToSubnautica2(string input, string expected)
    {
        Assert.Equal(expected, Guard.SanitizePath(input));
    }

    [Fact]
    public void SanitizePath_NoSubnautica2_ReturnsOriginal()
    {
        const string input = @"C:\Games\SomeOtherGame\save.sav";
        Assert.Equal(input, Guard.SanitizePath(input));
    }

    [Fact]
    public void SanitizePath_CaseInsensitive_MatchWorks()
    {
        var result = Guard.SanitizePath(@"D:\Stuff\subnautica2\saves\slot.sav");
        Assert.Contains("subnautica2", result, StringComparison.OrdinalIgnoreCase);
    }

    [Fact]
    public void SanitizePath_Empty_ReturnsEmpty()
    {
        Assert.Equal("", Guard.SanitizePath(""));
    }

    // ── IsNetworkPath ────────────────────────────────────────────────

    [Theory]
    [InlineData(@"\\server\share\file.sav", true)]
    [InlineData(@"//server/share/file.sav", true)]
    [InlineData(@"\\192.168.1.1\share", true)]
    [InlineData(@"C:\Local\Path\file.sav", false)]
    [InlineData(@"/unix/path/file.sav", false)]
    [InlineData("", false)]
    public void IsNetworkPath_VariousInputs(string path, bool expected)
    {
        Assert.Equal(expected, Guard.IsNetworkPath(path));
    }

    // ── GameRunning ──────────────────────────────────────────────────

    [Fact]
    public void GameRunning_DetectionDisabled_ReturnsFalse()
    {
        Guard.ProcessDetectionEnabled = false;
        try
        {
            Assert.False(Guard.GameRunning());
        }
        finally
        {
            Guard.ProcessDetectionEnabled = true; // restore
        }
    }

    [Fact]
    public void GameRunning_DetectionEnabled_ReturnsBool()
    {
        // Don't assume Subnautica2 is running — just verify it returns a bool
        Guard.ProcessDetectionEnabled = true;
        var result = Guard.GameRunning();
        Assert.IsType<bool>(result);
    }
}
