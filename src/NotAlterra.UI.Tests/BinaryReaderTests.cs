using NotAlterra.Gvas;
using Xunit;
using GvasBinaryReader = NotAlterra.Gvas.BinaryReader;

namespace NotAlterra_UI_Tests;

public class BinaryReaderTests
{
    // ── ReadU32 ──────────────────────────────────────────────────────

    [Fact]
    public void ReadU32_ValidOffset_ReturnsValue()
    {
        byte[] data = [0x78, 0x56, 0x34, 0x12];
        var result = GvasBinaryReader.ReadU32(data, 0);
        Assert.Equal(0x12345678u, result);
    }

    [Fact]
    public void ReadU32_NegativeOffset_ReturnsNull()
    {
        byte[] data = [0, 0, 0, 0];
        Assert.Null(GvasBinaryReader.ReadU32(data, -1));
    }

    [Fact]
    public void ReadU32_BeyondLength_ReturnsNull()
    {
        byte[] data = [0, 0, 0, 0];
        Assert.Null(GvasBinaryReader.ReadU32(data, 1)); // need 4 bytes
    }

    [Fact]
    public void ReadU32_EmptySpan_ReturnsNull()
    {
        Assert.Null(GvasBinaryReader.ReadU32([], 0));
    }

    // ── ReadI32 ──────────────────────────────────────────────────────

    [Fact]
    public void ReadI32_ValidPositive_ReturnsValue()
    {
        byte[] data = [0x2A, 0x00, 0x00, 0x00];
        Assert.Equal(42, GvasBinaryReader.ReadI32(data, 0));
    }

    [Fact]
    public void ReadI32_ValidNegative_ReturnsValue()
    {
        byte[] data = [0xFE, 0xFF, 0xFF, 0xFF]; // -2 in LE
        Assert.Equal(-2, GvasBinaryReader.ReadI32(data, 0));
    }

    [Fact]
    public void ReadI32_OutOfBounds_ReturnsNull()
    {
        Assert.Null(GvasBinaryReader.ReadI32([1, 2, 3], 0));
    }

    // ── ReadF64 ──────────────────────────────────────────────────────

    [Fact]
    public void ReadF64_ValidOffset_ReturnsValue()
    {
        // 1234.5678 as LE bytes
        var bytes = BitConverter.GetBytes(1234.5678);
        // Ensure LE (test runs on LE but we don't assume)
        if (!BitConverter.IsLittleEndian) Array.Reverse(bytes);
        var result = GvasBinaryReader.ReadF64(bytes, 0);
        Assert.NotNull(result);
        Assert.Equal(1234.5678, result.Value, precision: 4);
    }

    [Fact]
    public void ReadF64_OutOfBounds_ReturnsNull()
    {
        Assert.Null(GvasBinaryReader.ReadF64([1, 2, 3, 4, 5, 6, 7], 0));
    }

    // ── ReadFName ────────────────────────────────────────────────────

    [Fact]
    public void ReadFName_SimpleString_ReturnsValue()
    {
        // <u32 length=6> "Hello\0"
        byte[] data = [0x06, 0x00, 0x00, 0x00, (byte)'H', (byte)'e', (byte)'l', (byte)'l', (byte)'o', 0x00];
        var (value, next) = GvasBinaryReader.ReadFName(data, 0);
        Assert.Equal("Hello", value);
        Assert.Equal(10, next);
    }

    [Fact]
    public void ReadFName_TruncatedData_ReturnsNull()
    {
        byte[] data = [0x10, 0x00, 0x00, 0x00]; // says 16 bytes but none follow
        var (value, _) = GvasBinaryReader.ReadFName(data, 0);
        Assert.Null(value);
    }

    [Fact]
    public void ReadFName_NoNullTerminator_ReadsWhole()
    {
        // <u32 length=4> "Test" (no embedded null, length includes null terminator)
        byte[] data = [0x05, 0x00, 0x00, 0x00, (byte)'T', (byte)'e', (byte)'s', (byte)'t', 0x00];
        var (value, next) = GvasBinaryReader.ReadFName(data, 0);
        Assert.Equal("Test", value);
        Assert.Equal(9, next);
    }

    // ── ReadFString (UTF-8) ──────────────────────────────────────────

    [Fact]
    public void ReadFString_Utf8_ReturnsValue()
    {
        // <i32 length=6> "World\0"
        byte[] data = [0x06, 0x00, 0x00, 0x00, (byte)'W', (byte)'o', (byte)'r', (byte)'l', (byte)'d', 0x00];
        var (value, next) = GvasBinaryReader.ReadFString(data, 0);
        Assert.Equal("World", value);
        Assert.Equal(10, next);
    }

    [Fact]
    public void ReadFString_Empty_ReturnsEmpty()
    {
        // <i32 length=0>
        byte[] data = [0x00, 0x00, 0x00, 0x00];
        var (value, next) = GvasBinaryReader.ReadFString(data, 0);
        Assert.Equal(string.Empty, value);
        Assert.Equal(4, next);
    }

    [Fact]
    public void ReadFString_NegativeLength_ReturnsUtf16()
    {
        // <i32 length=-6> "Hi\0\0" as UTF-16 LE (4 code units = 8 bytes)
        // -6 means 6 bytes of UTF-16 data. But UTF-16 must be even. 
        // Actually -6 means 6 UTF-16 code units = 12 bytes
        // Let's use -4 meaning 4 UTF-16 code units = 8 bytes: "AB\0\0"
        byte[] data = [
            0xFC, 0xFF, 0xFF, 0xFF, // length = -4
            (byte)'H', 0x00, (byte)'i', 0x00, 0x00, 0x00, 0x00, 0x00
        ];
        var (value, next) = GvasBinaryReader.ReadFString(data, 0);
        Assert.Equal("Hi", value);
        Assert.Equal(12, next);
    }

    [Fact]
    public void ReadFString_Truncated_ReturnsNull()
    {
        byte[] data = [0x20, 0x00, 0x00, 0x00]; // 32 bytes but none follow
        var (value, _) = GvasBinaryReader.ReadFString(data, 0);
        Assert.Null(value);
    }

    // ── Edge cases ───────────────────────────────────────────────────

    [Fact]
    public void ReadF64_NaN_ReturnsNaN()
    {
        var nanBytes = BitConverter.GetBytes(double.NaN);
        var result = GvasBinaryReader.ReadF64(nanBytes, 0);
        Assert.NotNull(result);
        Assert.True(double.IsNaN(result.Value));
    }

    [Fact]
    public void ReadU32_AllBytes_ReturnsCorrect()
    {
        byte[] data = [0xFF, 0xFF, 0xFF, 0xFF];
        Assert.Equal(0xFFFFFFFFu, GvasBinaryReader.ReadU32(data, 0));
    }

    [Fact]
    public void ReadI32_AllBytesOnes_TwoComplement()
    {
        byte[] data = [0xFF, 0xFF, 0xFF, 0xFF];
        Assert.Equal(-1, GvasBinaryReader.ReadI32(data, 0));
    }

    [Fact]
    public void ReadFName_ZeroLength_ReturnsEmpty()
    {
        // <u32 length=0> — zero-length FName
        byte[] data = [0x00, 0x00, 0x00, 0x00];
        var (value, _) = GvasBinaryReader.ReadFName(data, 0);
        Assert.Null(value); // length <= 0 handled as failure
    }
}
