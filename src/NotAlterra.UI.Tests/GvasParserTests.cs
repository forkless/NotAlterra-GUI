using NotAlterra.Gvas;
using Xunit;

namespace NotAlterra_UI_Tests;

public class GvasParserTests
{
    private static readonly string TestDataDir = Path.Combine(
        AppDomain.CurrentDomain.BaseDirectory, // dotnet test CWD
        "..", "..", "..", "..", "..", "gvas-files");

    // ── Helpers ──────────────────────────────────────────────────────

    /// Build a minimal GVAS-like byte buffer that contains a named StrProperty.
    /// Actual GVAS format is complex; this tests the heuristic scanner logic.
    private static byte[] BuildMinimalGvasBytes(
        string propName, string propValue,
        string typeName = "StrProperty")
    {
        var nameBytes = System.Text.Encoding.UTF8.GetBytes(propName);
        var valBytes = System.Text.Encoding.UTF8.GetBytes(propValue);
        var typeBytes = System.Text.Encoding.UTF8.GetBytes(typeName);

        // FName header: <u32 len_incl_null> <bytes> <null>
        // then type FName, then 9 bytes padding, then FString value
        using var ms = new MemoryStream();
        var w = new BinaryWriter(ms);

        // Property name FName
        w.Write((uint)(nameBytes.Length + 1)); // length including null
        w.Write(nameBytes);
        w.Write((byte)0); // null terminator

        // Type FName
        w.Write((uint)(typeBytes.Length + 1));
        w.Write(typeBytes);
        w.Write((byte)0);

        // 9 bytes padding (property metadata: flags, size, etc)
        w.Write(new byte[9]);

        // FString value: <i32 len> <bytes> <null>
        w.Write(valBytes.Length + 1);
        w.Write(valBytes);
        w.Write((byte)0);

        return ms.ToArray();
    }

    private static byte[] BuildBoolPropertyBytes(string propName, bool value)
    {
        var nameBytes = System.Text.Encoding.UTF8.GetBytes(propName);
        var typeBytes = System.Text.Encoding.UTF8.GetBytes("BoolProperty");

        using var ms = new MemoryStream();
        var w = new BinaryWriter(ms);

        w.Write((uint)(nameBytes.Length + 1));
        w.Write(nameBytes);
        w.Write((byte)0);

        w.Write((uint)(typeBytes.Length + 1));
        w.Write(typeBytes);
        w.Write((byte)0);

        w.Write(new byte[9]); // padding
        w.Write(value ? (byte)1 : (byte)0);

        return ms.ToArray();
    }

    // ── ExtractMetadataFromBytes ─────────────────────────────────────

    [Fact]
    public void ExtractMetadataFromBytes_EmptyData_ReturnsErrors()
    {
        var meta = GvasParser.ExtractMetadataFromBytes([]);
        Assert.Null(meta.SlotName);
        Assert.Null(meta.DisplayName);
        Assert.NotEmpty(meta.Errors);
    }

    [Fact]
    public void ExtractMetadataFromBytes_FindsSlotName()
    {
        var data = BuildMinimalGvasBytes("SlotName", "savegame_0");
        var meta = GvasParser.ExtractMetadataFromBytes(data);
        Assert.Equal("savegame_0", meta.SlotName);
    }

    [Fact]
    public void ExtractMetadataFromBytes_FindsDisplayName()
    {
        var data = BuildMinimalGvasBytes("DisplayName", "Spoonmore");
        var meta = GvasParser.ExtractMetadataFromBytes(data);
        Assert.Equal("Spoonmore", meta.DisplayName);
    }

    [Fact]
    public void ExtractMetadataFromBytes_FindsOnlineFlag()
    {
        var data = BuildBoolPropertyBytes("bIsMultiplayerSave", true);
        var meta = GvasParser.ExtractMetadataFromBytes(data);
        Assert.True(meta.IsOnline);
    }

    [Fact]
    public void ExtractMetadataFromBytes_OnlineFlagAbsent_DefaultsFalse()
    {
        var data = BuildMinimalGvasBytes("SlotName", "savegame_0");
        var meta = GvasParser.ExtractMetadataFromBytes(data);
        Assert.False(meta.IsOnline);
    }

    [Fact]
    public void ExtractMetadataFromBytes_Playtime_ReturnsNullForNonGvas()
    {
        // Random bytes without "Elapsed" marker — no playtime
        var data = "Not a GVAS file at all!!!!"u8.ToArray();
        var meta = GvasParser.ExtractMetadataFromBytes(data);
        Assert.Null(meta.PlaytimeSeconds);
    }

    [Fact]
    public void ExtractMetadataFromBytes_MultipleProps_AllExtracted()
    {
        // Build multi-property data
        var slotData = BuildMinimalGvasBytes("SlotName", "savegame_9");
        var displayData = BuildMinimalGvasBytes("DisplayName", "TestSave");
        var onlineData = BuildBoolPropertyBytes("bIsMultiplayerSave", false);

        var combined = slotData.Concat(displayData).Concat(onlineData).ToArray();
        var meta = GvasParser.ExtractMetadataFromBytes(combined);

        Assert.Equal("savegame_9", meta.SlotName);
        Assert.Equal("TestSave", meta.DisplayName);
        Assert.False(meta.IsOnline);
        Assert.Null(meta.PlaytimeSeconds);
    }

    // ── ExtractFullMetadata ──────────────────────────────────────────

    private static string WriteTempFile(byte[] data)
    {
        var path = Path.Combine(Path.GetTempPath(), Guid.NewGuid() + ".sav");
        File.WriteAllBytes(path, data);
        return path;
    }

    [Fact]
    public void ExtractFullMetadata_NullByteData_DetectsCorruption()
    {
        var path = WriteTempFile(new byte[600]); // zeroed 600 bytes
        try
        {
            var meta = GvasParser.ExtractFullMetadata(path);
            Assert.NotNull(meta.CorruptionReason);
        }
        finally { File.Delete(path); }
    }

    [Fact]
    public void ExtractFullMetadata_TooSmall_DetectsCorruption()
    {
        var path = WriteTempFile(new byte[100]); // under 500
        try
        {
            var meta = GvasParser.ExtractFullMetadata(path);
            Assert.NotNull(meta.CorruptionReason);
            Assert.Contains("too small", meta.CorruptionReason, StringComparison.OrdinalIgnoreCase);
        }
        finally { File.Delete(path); }
    }

    [Fact]
    public void ExtractFullMetadata_MissingHeader_DetectsCorruption()
    {
        var data = new byte[200_000];
        data[0] = (byte)'X'; // not 'G'
        var path = WriteTempFile(data);
        try
        {
            var meta = GvasParser.ExtractFullMetadata(path);
            Assert.NotNull(meta.CorruptionReason);
            Assert.Contains("header", meta.CorruptionReason, StringComparison.OrdinalIgnoreCase);
        }
        finally { File.Delete(path); }
    }

    [Fact]
    public void ExtractFullMetadata_ZeroedLargeFile_DetectsBlank()
    {
        var path = WriteTempFile(new byte[200_000]); // above 100KB, all zeroed
        try
        {
            var meta = GvasParser.ExtractFullMetadata(path);
            Assert.NotNull(meta.CorruptionReason);
            Assert.Contains("blank", meta.CorruptionReason, StringComparison.OrdinalIgnoreCase);
        }
        finally { File.Delete(path); }
    }

    [Fact(Skip = "Requires real GVAS file at gvas-files/savegame_0.sav")]
    public void ExtractFullMetadata_RealSaveFile_ParsesCorrectly()
    {
        var path = Path.Combine(TestDataDir, "savegame_0.sav");
        Assert.True(File.Exists(path), $"Test file not found: {path}");

        var meta = GvasParser.ExtractFullMetadata(path);
        Assert.Null(meta.CorruptionReason);
        Assert.NotNull(meta.DisplayName);
        Assert.True(meta.PlaytimeSeconds > 0, "Playtime should be positive");
    }

    [Fact(Skip = "Requires real GVAS file at gvas-files/savegame_corrupt_magic.sav")]
    public void ExtractFullMetadata_CorruptFile_DetectsIssue()
    {
        var path = Path.Combine(TestDataDir, "savegame_corrupt_magic.sav");
        Assert.True(File.Exists(path));

        var meta = GvasParser.ExtractFullMetadata(path);
        Assert.NotNull(meta.CorruptionReason);
    }
}
