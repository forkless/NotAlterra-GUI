// Property-based fuzz tests using FsCheck.
// Replaces SharpFuzz (broken on .NET 9). Same crash-finding power,
// runs inside dotnet test, no CI infra needed.

using FsCheck;
using FsCheck.Xunit;
using NotAlterra.Gvas;
using NotAlterra.Services;

namespace NotAlterra_UI_Tests;

public class FuzzProperties
{
    // ── GVAS parser ─────────────────────────────────────────────────

    [Property(MaxTest = 500)]
    public void GvasParser_ExtractMetadataFromBytes_NeverCrashes(NonNull<byte[]> data)
    {
        // Feed random bytes to the heuristic parser.
        // Must never throw — any exception is a real bug.
        _ = GvasParser.ExtractMetadataFromBytes(data.Item);
    }

    [Property(MaxTest = 200)]
    public void GvasParser_ExtractFullMetadata_FileNeverCrashes(NonNull<byte[]> data)
    {
        // Feed random bytes as a .sav file, call the file-based API.
        var tmp = Path.GetTempFileName() + ".sav";
        try
        {
            File.WriteAllBytes(tmp, data.Item);
            _ = GvasParser.ExtractFullMetadata(tmp);
        }
        finally
        {
            try { File.Delete(tmp); } catch { }
        }
    }

    [Property(MaxTest = 200)]
    public void GvasParser_ExtractMetadata_NeverThrows(NonNull<byte[]> data)
    {
        var tmp = Path.GetTempFileName() + ".sav";
        try
        {
            File.WriteAllBytes(tmp, data.Item);
            _ = GvasParser.ExtractMetadata(tmp);
        }
        finally
        {
            try { File.Delete(tmp); } catch { }
        }
    }

    // ── BinaryReader ────────────────────────────────────────────────

    [Property(MaxTest = 200)]
    public void BinaryReader_ReadU32_NeverThrows(NonNull<byte[]> data, int offset)
    {
        _ = NotAlterra.Gvas.BinaryReader.ReadU32(data.Item, offset);
    }

    [Property(MaxTest = 200)]
    public void BinaryReader_ReadI32_NeverThrows(NonNull<byte[]> data, int offset)
    {
        _ = NotAlterra.Gvas.BinaryReader.ReadI32(data.Item, offset);
    }

    [Property(MaxTest = 200)]
    public void BinaryReader_ReadF64_NeverThrows(NonNull<byte[]> data, int offset)
    {
        _ = NotAlterra.Gvas.BinaryReader.ReadF64(data.Item, offset);
    }

    [Property(MaxTest = 200)]
    public void BinaryReader_ReadFName_NeverThrows(NonNull<byte[]> data, int offset)
    {
        _ = NotAlterra.Gvas.BinaryReader.ReadFName(data.Item, offset);
    }

    [Property(MaxTest = 200)]
    public void BinaryReader_ReadFString_NeverThrows(NonNull<byte[]> data, int offset)
    {
        _ = NotAlterra.Gvas.BinaryReader.ReadFString(data.Item, offset);
    }

    // ── SaveOps ─────────────────────────────────────────────────────

    [Property(MaxTest = 100)]
    public void SaveOps_CheckTarGzIntegrity_NeverThrows(NonNull<byte[]> data)
    {
        var tmp = Path.GetTempFileName() + ".tar.gz";
        try
        {
            File.WriteAllBytes(tmp, data.Item);
            _ = SaveOps.CheckTarGzIntegrity(tmp);
        }
        finally
        {
            try { File.Delete(tmp); } catch { }
        }
    }

    [Property(MaxTest = 100)]
    public void SaveOps_VerifyTarGzIntegrity_NeverThrows(NonNull<byte[]> data)
    {
        var tmp = Path.GetTempFileName() + ".tar.gz";
        try
        {
            File.WriteAllBytes(tmp, data.Item);
            _ = SaveOps.VerifyTarGzIntegrity(tmp);
        }
        finally
        {
            try { File.Delete(tmp); } catch { }
        }
    }

    [Property(MaxTest = 100)]
    public void SaveOps_ReadTarGzManifest_NeverThrows(NonNull<byte[]> data)
    {
        var tmp = Path.GetTempFileName() + ".tar.gz";
        try
        {
            File.WriteAllBytes(tmp, data.Item);
            _ = SaveOps.ReadTarGzManifest(tmp);
        }
        finally
        {
            try { File.Delete(tmp); } catch { }
        }
    }

    // ── Guard ───────────────────────────────────────────────────────

    [Property]
    public void Guard_SanitizePath_NeverThrows(NonNull<string> path)
    {
        _ = Guard.SanitizePath(path.Item);
    }

    [Property]
    public void Guard_IsNetworkPath_NeverThrows(NonNull<string> path)
    {
        _ = Guard.IsNetworkPath(path.Item);
    }

    [Property]
    public void Guard_LogAction_NeverThrows(NonNull<string> action, NonNull<string> detail, NonNull<string> result)
    {
        Guard.LogAction(action.Item, detail.Item, result.Item);
    }

    // ── SlotUtils ───────────────────────────────────────────────────

    [Property]
    public void SlotUtils_DeriveSlot_NeverThrows(NonNull<string> filename)
    {
        _ = SlotUtils.DeriveSlotFromFilename(filename.Item);
    }

    [Property]
    public void SlotUtils_CorruptionCheck_NeverThrows(NonNull<string> filename, NonNull<string> slot)
    {
        _ = SlotUtils.CorruptionCheck(filename.Item, slot.Item);
    }
}
