// Sequential GVAS property stream walker.
// Walks top-level properties by reading their headers.
// Skips struct values by scanning forward for the next valid property name.

using System.Collections.Generic;
using System.Text;

namespace NotAlterra.Gvas;

public record GvasProperty(string Name, string Type, uint Size, uint ArrayIndex, object? Value);

public static class PropertyWalker
{
    public static List<GvasProperty> WalkAll(byte[] data)
    {
        var props = new List<GvasProperty>();

        int gvasStart = IndexOfMagic(data, "GVAS");
        if (gvasStart < 0) return props;

        int offset = gvasStart + 12; // past "GVAS" + FileVersion + PackageVersion
        int headerEnd = FindSaveGameClassName(data, gvasStart);
        if (headerEnd <= offset) return props;
        offset = headerEnd;

        while (offset < data.Length)
        {
            var (propName, nameEnd) = BinaryReader.ReadFName(data, offset);
            if (propName == null || nameEnd <= offset) break;

            if (propName == "None")
            { offset = nameEnd; break; }

            offset = nameEnd;
            var (typeName, typeEnd) = BinaryReader.ReadFName(data, offset);
            if (typeName == null || typeEnd <= offset) break;
            offset = typeEnd;

            uint propSize = BinaryReader.ReadU32(data, offset) ?? 0;
            offset += 4;
            uint arrayIndex = BinaryReader.ReadU32(data, offset) ?? 0;
            offset += 4;

            object? value = null;
            int consumed = (int)propSize;

            switch (typeName)
            {
                case "IntProperty":
                    if (propSize >= 4) value = BinaryReader.ReadU32(data, offset);
                    break;
                case "FloatProperty":
                    if (propSize >= 4)
                    {
                        var bits = BinaryReader.ReadU32(data, offset);
                        if (bits.HasValue) value = BitConverter.ToSingle(BitConverter.GetBytes(bits.Value), 0);
                    }
                    break;
                case "DoubleProperty":
                    value = BinaryReader.ReadF64(data, offset);
                    break;
                case "BoolProperty":
                    if (propSize >= 1) value = data[offset] != 0;
                    break;
                case "StrProperty":
                case "TextProperty":
                    { var (s, _) = BinaryReader.ReadFString(data, offset); value = s; }
                    break;
                case "NameProperty":
                    { var (s, _) = BinaryReader.ReadFName(data, offset); value = s; }
                    break;
                case "StructProperty":
                    // Skip struct value by name + data; find end by scanning for next property
                    consumed = SkipStruct(data, offset, propSize);
                    value = consumed >= 0 ? "(struct)" : null;
                    if (consumed < 0) consumed = (int)propSize;
                    break;
                case "ByteProperty":
                    {
                        var (enumType, enumEnd) = BinaryReader.ReadFName(data, offset);
                        if (enumType != null && enumEnd > offset && enumEnd < offset + propSize)
                            value = data[enumEnd];
                        else if (propSize >= 1) value = data[offset];
                    }
                    break;
                case "EnumProperty":
                    {
                        var (innerType, innerEnd) = BinaryReader.ReadFName(data, offset);
                        if (innerType != null)
                        {
                            var (enumValue, valEnd) = BinaryReader.ReadFName(data, innerEnd);
                            if (enumValue != null) value = $"{innerType}.{enumValue}";
                        }
                    }
                    break;
                default:
                    value = $"({typeName}:{propSize}b)";
                    break;
            }

            if (consumed < 0) consumed = 0;
            if (offset + consumed > data.Length) consumed = data.Length - offset;

            props.Add(new GvasProperty(propName, typeName, propSize, arrayIndex, value));
            offset += consumed;
        }

        return props;
    }

    // Skip a struct value: read struct_type_name FName + its data.
    // Returns the number of bytes consumed from 'offset'.
    private static int SkipStruct(byte[] data, int offset, uint propSize)
    {
        int start = offset;
        var (sname, snameEnd) = BinaryReader.ReadFName(data, offset);
        if (sname == null || snameEnd <= offset) return (int)propSize;

        // After struct name, scan for the end of struct data.
        // The struct value is not a GVAS property stream — it's binary C++ data.
        // We need to find where it ends by looking for the next top-level property.
        // Strategy: scan forward for a valid FName that looks like a property name.
        int end = FindNextProperty(data, snameEnd, start + 100000);
        if (end > snameEnd)
            return end - start;

        // Fallback: use propSize (might be wrong but better than infinite loop)
        return (int)propSize;
    }

    // Scan forward for the start of the next top-level property.
    // Looks for known property names with valid FName length prefixes.
    private static int FindNextProperty(byte[] data, int searchStart, int searchEnd)
    {
        searchEnd = Math.Min(searchEnd, data.Length);
        string[] knownProps = {
            "SlotName", "DisplayName", "GameMode", "LevelName", "bIsMultiplayerSave",
            "bWasMultiplayerSave", "BuildNumber", "BuildBranch", "SavesCount",
            "LatestVersion", "DataVersion", "EngineVersion", "MTime",
            "CreatedAt", "LastModified", "None"
        };

        for (int i = searchStart; i <= searchEnd - 10; i++)
        {
            foreach (var name in knownProps)
            {
                if (i + name.Length > data.Length) continue;
                bool match = true;
                for (int j = 0; j < name.Length; j++)
                    if (data[i + j] != name[j]) { match = false; break; }
                if (!match) continue;

                // Verify FName length prefix
                int prefixLen = BitConverter.ToInt32(data, i - 4);
                if (prefixLen != name.Length + 1) continue;
                if (data[i + name.Length] != 0) continue;

                return i - 4; // Return start of this property
            }
        }
        return -1;
    }

    private static int FindSaveGameClassName(byte[] data, int gvasStart)
    {
        int searchEnd = Math.Min(data.Length, gvasStart + 4096);
        for (int i = gvasStart + 12; i <= searchEnd - 10; i++)
        {
            if (data[i] != '/' || data[i + 1] != 'S' || data[i + 2] != 'c') continue;
            int rawLen = BitConverter.ToInt32(data, i - 4);
            if (rawLen <= 0 || rawLen > 500) continue;
            if (i - 4 + rawLen > data.Length) continue;
            if (data[i + rawLen - 1] != 0) continue;

            int propStart = i + rawLen;
            while (propStart < data.Length && data[propStart] == 0) propStart++;
            return propStart;
        }
        return -1;
    }

    private static int IndexOfMagic(byte[] data, string magic)
    {
        var m = Encoding.UTF8.GetBytes(magic);
        int max = Math.Min(data.Length, 512);
        for (int i = 0; i <= max - 4; i++)
            if (data[i] == m[0] && data[i + 1] == m[1] && data[i + 2] == m[2] && data[i + 3] == m[3])
                return i;
        return -1;
    }

    public static Dictionary<string, object?> WalkToDict(byte[] data)
    {
        var dict = new Dictionary<string, object?>();
        foreach (var prop in WalkAll(data))
            dict[prop.Name] = prop.Value;
        return dict;
    }
}
