// SharpFuzz fuzz targets for NotAlterra.Core.
//
// Targets:
//   gvas_parse        — GvasParser.ExtractMetadataFromBytes(byte[])
//   gvas_full_meta    — GvasParser.ExtractFullMetadata(filePath)
//   tar_gz_integrity  — SaveOps.CheckTarGzIntegrity + VerifyTarGzIntegrity
//   tar_gz_manifest   — SaveOps.ReadTarGzManifest
//
// Usage:
//   sharpfuzz bin/Release/net9.0/NotAlterra.Fuzz.dll
//   sharpfuzz bin/Release/net9.0/NotAlterra.Core.dll
//   dotnet run -- [target] -- corpus/[target]/ [-runs=N]

using NotAlterra.Gvas;
using NotAlterra.Services;
using SharpFuzz;

var target = args.Length > 0 && !args[0].StartsWith('-')
    ? args[0] : "gvas_parse";

switch (target)
{
    case "gvas_parse":
        Fuzzer.LibFuzzer.Run(static data =>
        {
            GvasParser.ExtractMetadataFromBytes(data.ToArray());
        });
        break;

    case "gvas_full_meta":
        Fuzzer.LibFuzzer.Run(static data =>
        {
            var bytes = data.ToArray();
            var tmp = Path.GetTempFileName() + ".sav";
            try
            {
                File.WriteAllBytes(tmp, bytes);
                GvasParser.ExtractFullMetadata(tmp);
            }
            finally { TryDelete(tmp); }
        });
        break;

    case "tar_gz_integrity":
        Fuzzer.LibFuzzer.Run(static data =>
        {
            var bytes = data.ToArray();
            var tmp = Path.GetTempFileName() + ".tar.gz";
            try
            {
                File.WriteAllBytes(tmp, bytes);
                SaveOps.CheckTarGzIntegrity(tmp);
                SaveOps.VerifyTarGzIntegrity(tmp);
            }
            finally { TryDelete(tmp); }
        });
        break;

    case "tar_gz_manifest":
        Fuzzer.LibFuzzer.Run(static data =>
        {
            var bytes = data.ToArray();
            var tmp = Path.GetTempFileName() + ".tar.gz";
            try
            {
                File.WriteAllBytes(tmp, bytes);
                SaveOps.ReadTarGzManifest(tmp);
            }
            finally { TryDelete(tmp); }
        });
        break;

    default:
        Console.Error.WriteLine($"Unknown target: {target}");
        Console.Error.WriteLine("Valid: gvas_parse, gvas_full_meta, tar_gz_integrity, tar_gz_manifest");
        Environment.Exit(1);
        break;
}

static void TryDelete(string path)
{
    try { File.Delete(path); } catch { /* best-effort cleanup */ }
}
