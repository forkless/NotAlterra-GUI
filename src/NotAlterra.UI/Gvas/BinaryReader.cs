// Low-level binary read primitives for UE5 GVAS format.
// Mirrors reader.h — bounds-checked, all static methods.

using System.Text;

namespace NotAlterra.Gvas;

internal static class BinaryReader
{
    /// Read a little-endian u32 at offset, null if out of bounds.
    public static uint? ReadU32(ReadOnlySpan<byte> data, int offset)
    {
        if (offset < 0 || offset + 4 > data.Length)
            return null;
        return (uint)(data[offset] | (data[offset + 1] << 8) |
                      (data[offset + 2] << 16) | (data[offset + 3] << 24));
    }

    /// Read a little-endian i32 at offset, null if out of bounds.
    public static int? ReadI32(ReadOnlySpan<byte> data, int offset)
    {
        if (offset < 0 || offset + 4 > data.Length)
            return null;
        return data[offset] | (data[offset + 1] << 8) |
               (data[offset + 2] << 16) | (data[offset + 3] << 24);
    }

    /// Read a little-endian f64 at offset, null if out of bounds.
    public static double? ReadF64(ReadOnlySpan<byte> data, int offset)
    {
        if (offset < 0 || offset + 8 > data.Length)
            return null;
        return BitConverter.ToDouble(data.Slice(offset, 8));
    }

    /// Read an FName: &lt;u32 length&gt;&lt;bytes&gt;&lt;optional null&gt;
    /// Returns (string, new_offset) or (null, offset) on failure.
    public static (string? Value, int NextOffset) ReadFName(
        ReadOnlySpan<byte> data, int offset)
    {
        var len = ReadU32(data, offset);
        if (len == null)
            return (null, offset);

        int off = offset + 4;
        int length = (int)len.Value;
        if (length <= 0 || off + length > data.Length)
            return (null, off);

        int strLen = length;
        if (data[off + strLen - 1] == 0)
            strLen--;

        var s = Encoding.UTF8.GetString(data.Slice(off, strLen));
        return (s, off + length);
    }

    /// Read an FString: &lt;i32 length&gt; — negative=UTF16, positive=UTF8 (incl null).
    /// Returns (string, new_offset) or (null, offset) on failure.
    public static (string? Value, int NextOffset) ReadFString(
        ReadOnlySpan<byte> data, int offset)
    {
        var rawLen = ReadI32(data, offset);
        if (rawLen == null)
            return (null, offset);

        int off = offset + 4;
        int rawLenVal = rawLen.Value;

        if (rawLenVal == 0)
            return (string.Empty, off);

        bool isUtf16 = rawLenVal < 0;
        int bytes = isUtf16
            ? -rawLenVal * 2
            : rawLenVal;

        if (off + bytes > data.Length)
            return (null, off);

        string value;
        if (isUtf16)
        {
            // Decode UTF-16 LE
            int codeUnits = bytes / 2;
            Span<char> chars = stackalloc char[codeUnits];
            for (int i = 0; i < codeUnits; i++)
            {
                chars[i] = (char)(data[off + i * 2] | (data[off + i * 2 + 1] << 8));
            }
            value = new string(chars);
        }
        else
        {
            int strLen = bytes;
            if (strLen > 0 && data[off + strLen - 1] == 0)
                strLen--;
            value = Encoding.UTF8.GetString(data.Slice(off, strLen));
        }

        return (value, off + bytes);
    }
}
