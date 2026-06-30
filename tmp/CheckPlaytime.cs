using NotAlterra.Core.Gvas;
using NotAlterra.Core.Types;

var meta = Parser.ExtractMetadata(@"gvas-files\_extracted\savegame_0.sav");
Console.WriteLine($"DisplayName: {meta.DisplayName}");
Console.WriteLine($"SlotName: {meta.SlotName}");
Console.WriteLine($"IsOnline: {meta.IsOnline}");
Console.WriteLine($"PlaytimeSeconds: {meta.PlaytimeSeconds}");
if (meta.PlaytimeSeconds.HasValue)
{
    var ts = TimeSpan.FromSeconds(meta.PlaytimeSeconds.Value);
    Console.WriteLine($"Playtime: {ts.Hours}h {ts.Minutes}m");
}
else
{
    Console.WriteLine("Playtime: null");
}
if (meta.Errors?.Count > 0)
    Console.WriteLine($"Errors: {string.Join(", ", meta.Errors)}");
