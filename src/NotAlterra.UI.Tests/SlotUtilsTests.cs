using NotAlterra.Gvas;
using Xunit;

namespace NotAlterra_UI_Tests;

public class SlotUtilsTests
{
    // ── DeriveSlotFromFilename ───────────────────────────────────────

    [Theory]
    [InlineData("savegame_0.sav", "savegame_0")]
    [InlineData("savegame_0.bak", "savegame_0")]
    [InlineData("savegame_12.sav", "savegame_12")]
    [InlineData("savegame_123.bak", "savegame_123")]
    [InlineData("savegame_0_1.bak", "savegame_0")]
    [InlineData("SAVEGAME_0.SAV", "savegame_0")]
    [InlineData("savegame_9.bak", "savegame_9")]
    public void DeriveSlot_ValidNames_ReturnsSlot(string filename, string expected)
    {
        Assert.Equal(expected, SlotUtils.DeriveSlotFromFilename(filename));
    }

    [Theory]
    [InlineData("")]
    [InlineData("whatever.sav")]
    [InlineData(".bak")]
    [InlineData("savegame_.sav")]
    public void DeriveSlot_InvalidNames_ReturnsNull(string filename)
    {
        Assert.Null(SlotUtils.DeriveSlotFromFilename(filename));
    }

    [Fact]
    public void DeriveSlot_HandlesLongNumbers()
    {
        // UE5 slot numbers realistically < 100, but handle general case
        var result = SlotUtils.DeriveSlotFromFilename("savegame_999999999999.bak");
        Assert.Equal("savegame_999999999999", result);
    }

    [Fact]
    public void DeriveSlot_StopsAtFirstNonDigit()
    {
        var result = SlotUtils.DeriveSlotFromFilename("savegame_0_backup.sav");
        Assert.Equal("savegame_0", result);
    }

    // ── CorruptionCheck ──────────────────────────────────────────────

    [Fact]
    public void CorruptionCheck_MatchingNames_ReturnsNull()
    {
        var result = SlotUtils.CorruptionCheck("savegame_0.sav", "savegame_0");
        Assert.Null(result);
    }

    [Fact]
    public void CorruptionCheck_MismatchedNames_ReturnsMessage()
    {
        var result = SlotUtils.CorruptionCheck("savegame_0.sav", "savegame_1");
        Assert.NotNull(result);
        Assert.Contains("Slot name mismatch", result, StringComparison.OrdinalIgnoreCase);
        Assert.Contains("savegame_1", result);
        Assert.Contains("savegame_0", result);
    }

    [Fact]
    public void CorruptionCheck_NullSlot_ReturnsNull()
    {
        var result = SlotUtils.CorruptionCheck("savegame_0.sav", null);
        Assert.Null(result);
    }

    [Fact]
    public void CorruptionCheck_InvalidFilename_ReturnsNull()
    {
        var result = SlotUtils.CorruptionCheck("nope.sav", "savegame_0");
        Assert.Null(result);
    }
}
