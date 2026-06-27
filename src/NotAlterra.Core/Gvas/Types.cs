// Metadata structs extracted from UE5 GVAS save files.
// Mirrors types.h.

namespace NotAlterra.Gvas;

/// Full metadata extracted from a UE5 GVAS save file.
public record FullMetadata
{
    public string? SlotName { get; init; }
    public string? DisplayName { get; init; }
    public bool IsOnline { get; init; }
    public bool WasMultiplayer { get; init; }
    public string? GameMode { get; init; }
    public string? LevelName { get; init; }
    public uint? BuildNumber { get; init; }
    public string? BuildBranch { get; init; }
    public uint? SavesCount { get; init; }
    public uint? LatestVersion { get; init; }
    public uint? DataVersion { get; init; }
    public double? PlaytimeSeconds { get; init; }
    public string? CorruptionReason { get; init; }
}

/// User-facing metadata for the restore picker.
public record SaveMetadata
{
    public string? SlotName { get; init; }
    public string? DisplayName { get; init; }
    public bool IsOnline { get; init; }
    public double? PlaytimeSeconds { get; init; }
    public List<string> Errors { get; init; } = new();
}

/// Slot name derived from filename, e.g. "savegame_0"
public static class SlotUtils
{
    public static string? DeriveSlotFromFilename(string filename)
    {
        var pos = filename.IndexOf("savegame_", StringComparison.OrdinalIgnoreCase);
        if (pos < 0) return null;

        var rest = filename[(pos + 9)..]; // after "savegame_"
        if (string.IsNullOrEmpty(rest) || !char.IsDigit(rest[0]))
            return null;

        var slot = "savegame_";
        foreach (var c in rest)
        {
            if (char.IsDigit(c)) slot += c;
            else break;
        }
        return slot;
    }

    public static string? CorruptionCheck(
        string filename, string? slotName)
    {
        var expected = DeriveSlotFromFilename(filename);
        if (expected == null) return null;
        if (slotName != null && slotName != expected)
            return $"Slot name mismatch: file says \"{slotName}\", expected \"{expected}\"";
        return null;
    }
}
