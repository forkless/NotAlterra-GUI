// Heuristic byte-scan GVAS parser.
// Searches binary for property names as raw byte patterns, validates
// the FName length field before each hit, then extracts typed values.
// Mirrors parser.cpp.

using System.Runtime.InteropServices;

namespace NotAlterra.Gvas;

public static class GvasParser
{
    // ── private extractors ────────────────────────────────────────────────

    /// Scan for a StrProperty by name, return its FString value.
    /// Walks binary looking for FName header → type "StrProperty" → value.
    private static string? ExtractStrProperty(
        ReadOnlySpan<byte> data, string propName, List<string>? errors = null)
    {
        var target = System.Text.Encoding.UTF8.GetBytes(propName);
        int offset = 0;
        int attempts = 0;

        while (offset + 20 <= data.Length && attempts < 100)
        {
            var found = data.Slice(offset).IndexOf(target);
            if (found < 0)
            {
                if (errors != null) errors.Add($"{propName} not found");
                return null;
            }
            found += offset;

            // Validate FName length field precedes the name
            if (found < 4) { offset = found + 1; attempts++; continue; }
            var nameLen = BinaryReader.ReadU32(data, found - 4);
            if (nameLen != target.Length + 1) { offset = found + 1; attempts++; continue; }

            // Validate null terminator after name
            if (found + target.Length >= data.Length || data[found + target.Length] != 0)
            { offset = found + 1; attempts++; continue; }

            // Read the type name (should match "StrProperty")
            var afterName = found + target.Length + 1;
            var (typeName, typeOff) = BinaryReader.ReadFName(data, afterName);
            if (typeName != "StrProperty") { offset = found + 1; attempts++; continue; }

            // Skip 9 bytes of property metadata, then read the FString value
            int metaOff = typeOff + 9;
            if (metaOff + 4 > data.Length) { offset = found + 1; attempts++; continue; }

            var (value, _) = BinaryReader.ReadFString(data, metaOff);
            if (!string.IsNullOrEmpty(value) && value.Length < 100)
                return value;

            offset = found + 1;
            attempts++;
        }

        if (errors != null) errors.Add($"no valid {propName}/StrProperty pair found");
        return null;
    }

    /// Find a BoolProperty by name, return true/false.
    private static bool? ExtractBoolProperty(ReadOnlySpan<byte> data, string propName)
    {
        var target = System.Text.Encoding.UTF8.GetBytes(propName);
        int offset = 0;
        int attempts = 0;

        while (offset + 20 <= data.Length && attempts < 100)
        {
            var found = data.Slice(offset).IndexOf(target);
            if (found < 0) return null;
            found += offset;

            if (found < 4) { offset = found + 1; attempts++; continue; }
            var nameLen = BinaryReader.ReadU32(data, found - 4);
            if (nameLen != target.Length + 1) { offset = found + 1; attempts++; continue; }
            if (found + target.Length >= data.Length || data[found + target.Length] != 0)
            { offset = found + 1; attempts++; continue; }

            var afterName = found + target.Length + 1;
            var (typeName, typeOff) = BinaryReader.ReadFName(data, afterName);
            if (typeName != "BoolProperty") { offset = found + 1; attempts++; continue; }

            int valOff = typeOff + 9;
            if (valOff >= data.Length) { offset = found + 1; attempts++; continue; }

            return data[valOff] != 0;
        }
        return null;
    }

    /// Find a DoubleProperty by name.
    private static double? ExtractDoubleProperty(ReadOnlySpan<byte> data, string propName)
    {
        var target = System.Text.Encoding.UTF8.GetBytes(propName);
        int offset = 0;
        int attempts = 0;

        while (offset + 30 <= data.Length && attempts < 100)
        {
            var found = data.Slice(offset).IndexOf(target);
            if (found < 0) return null;
            found += offset;

            if (found < 4) { offset = found + 1; attempts++; continue; }
            var nameLen = BinaryReader.ReadU32(data, found - 4);
            if (nameLen != target.Length + 1) { offset = found + 1; attempts++; continue; }
            if (found + target.Length >= data.Length || data[found + target.Length] != 0)
            { offset = found + 1; attempts++; continue; }

            var afterName = found + target.Length + 1;
            var (typeName, typeOff) = BinaryReader.ReadFName(data, afterName);
            if (typeName != "DoubleProperty") { offset = found + 1; attempts++; continue; }

            int valOff = typeOff + 9;
            if (valOff + 8 > data.Length) { offset = found + 1; attempts++; continue; }

            return BinaryReader.ReadF64(data, valOff);
        }
        return null;
    }

    /// Find an IntProperty by name, return u32 value.
    private static uint? ExtractIntProperty(ReadOnlySpan<byte> data, string propName)
    {
        var target = System.Text.Encoding.UTF8.GetBytes(propName);
        int offset = 0;
        int attempts = 0;

        while (offset + 20 <= data.Length && attempts < 100)
        {
            var found = data.Slice(offset).IndexOf(target);
            if (found < 0) return null;
            found += offset;

            if (found < 4) { offset = found + 1; attempts++; continue; }
            var nameLen = BinaryReader.ReadU32(data, found - 4);
            if (nameLen != target.Length + 1) { offset = found + 1; attempts++; continue; }
            if (found + target.Length >= data.Length || data[found + target.Length] != 0)
            { offset = found + 1; attempts++; continue; }

            var afterName = found + target.Length + 1;
            var (typeName, typeOff) = BinaryReader.ReadFName(data, afterName);
            if (typeName != "IntProperty") { offset = found + 1; attempts++; continue; }

            int valOff = typeOff + 9;
            if (valOff + 4 > data.Length) { offset = found + 1; attempts++; continue; }

            return BinaryReader.ReadU32(data, valOff);
        }
        return null;
    }

    /// Heuristic: scan for "Elapsed" bytes and look for a plausible double nearby.
    private static double? ScanDoubleNearElapsed(ReadOnlySpan<byte> data)
    {
        var marker = "Elapsed"u8;
        var pos = data.IndexOf(marker);
        if (pos < 0) return null;

        int end = Math.Min(pos + 60, data.Length);

        for (int off = 8; off < 50 && pos + off + 8 <= end; off++)
        {
            var val = BinaryReader.ReadF64(data, pos + off);
            if (val.HasValue && val.Value > 1.0 && val.Value < 10_000_000.0)
                return val;
        }
        return null;
    }

    // ── public API ────────────────────────────────────────────────────────

    /// Parse a .sav or .bak file and return all known GVAS metadata.
    public static FullMetadata ExtractFullMetadata(string filePath)
    {
        var data = File.ReadAllBytes(filePath);
        var span = new ReadOnlySpan<byte>(data);

        // ── Corruption scan ──
        string? corruption = data.Length < 100_000
            ? "File is too small to be a valid save (under 100 KB)"
            : null;
        if (corruption == null && data.AsSpan(0, Math.Min(100, data.Length)).IndexOfAnyExcept((byte)0) < 0)
            corruption = "File appears to be blank or zeroed";
        if (corruption == null && data[0] != 'G')
            corruption = "Missing GVAS/GSWU header (truncated or not a save file)";

        var slot = ExtractStrProperty(span, "SlotName");
        var display = ExtractStrProperty(span, "DisplayName");
        var game = ExtractStrProperty(span, "GameMode");
        var level = ExtractStrProperty(span, "LevelName");
        var branch = ExtractStrProperty(span, "BuildBranch");

        // If metadata scan found nothing and corruption not already flagged
        if (corruption == null && slot == null && display == null && game == null)
            corruption = "No recognizable save metadata found (file structure is corrupt)";

        return new FullMetadata
        {
            SlotName = slot,
            DisplayName = display,
            IsOnline = ExtractBoolProperty(span, "bIsMultiplayerSave") ?? false,
            WasMultiplayer = ExtractBoolProperty(span, "bWasMultiplayerSave") ?? false,
            GameMode = game,
            LevelName = level,
            BuildNumber = ExtractIntProperty(span, "BuildNumber"),
            BuildBranch = branch,
            SavesCount = ExtractIntProperty(span, "SavesCount"),
            LatestVersion = ExtractIntProperty(span, "LatestVersion"),
            DataVersion = ExtractIntProperty(span, "DataVersion"),
            PlaytimeSeconds = ScanDoubleNearElapsed(span),
            CorruptionReason = corruption,
        };
    }

    /// Parse GVAS metadata from an in-memory byte array. Useful for testing.
    public static SaveMetadata ExtractMetadataFromBytes(byte[] data)
    {
        var span = new ReadOnlySpan<byte>(data);
        var errors = new List<string>();

        var slot = ExtractStrProperty(span, "SlotName", errors);
        var display = ExtractStrProperty(span, "DisplayName", errors);

        return new SaveMetadata
        {
            SlotName = slot,
            DisplayName = display,
            IsOnline = ExtractBoolProperty(span, "bIsMultiplayerSave") ?? false,
            PlaytimeSeconds = ScanDoubleNearElapsed(span),
            Errors = errors,
        };
    }

    /// Parse a .sav or .bak file and return user-facing metadata.
    public static SaveMetadata ExtractMetadata(string filePath)
    {
        var data = File.ReadAllBytes(filePath);
        return ExtractMetadataFromBytes(data);
    }
}
