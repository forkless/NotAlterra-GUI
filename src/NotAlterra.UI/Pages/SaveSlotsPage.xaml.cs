using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using NotAlterra.Gvas;

namespace NotAlterra_UI.Pages;

public record BakInfo(string FilePath, long Size, DateTime Modified)
{
    public string Name => Path.GetFileName(FilePath);
    public string SizeDisplay => Size switch
    {
        > 1_000_000 => $"{Size / 1_000_000.0:F1} MB",
        > 1_000 => $"{Size / 1_000.0:F1} KB",
        _ => $"{Size} B"
    };
    public string ModifiedDisplay => Modified.ToString("yyyy-MM-dd HH:mm");
}

public record SlotInfo(int Number, string SavPath, long Size, DateTime Modified, FullMetadata? Meta, List<BakInfo> Backups)
{
    public string SizeDisplay => Size switch
    {
        > 1_000_000 => $"{Size / 1_000_000.0:F1} MB",
        > 1_000 => $"{Size / 1_000.0:F1} KB",
        _ => $"{Size} B"
    };
    public string ModifiedDisplay => Modified.ToString("yyyy-MM-dd HH:mm");
    public string BakStatus => Backups.Count > 0 ? $"{Backups.Count} backup(s)" : "No backup";
    public string PlaytimeDisplay => Meta?.PlaytimeSeconds is double s
        ? $"{(int)s / 3600}h {(int)(s % 3600) / 60}m" : "—";
    public string ModeDisplay => Meta?.IsOnline == true ? "Online" :
        Meta?.WasMultiplayer == true ? "MP (was)" : "Single Player";
    public string NameDisplay => !string.IsNullOrEmpty(Meta?.DisplayName)
        ? Meta.DisplayName : $"Slot {Number}";
    public string GameModeDisplay => !string.IsNullOrEmpty(Meta?.GameMode)
        ? Meta.GameMode.Replace("BP_", "").Replace("_C", "") : "";
    public string SavesCountDisplay => Meta?.SavesCount is uint sc ? $"×{sc}" : "";
    public string BuildDisplay => Meta?.BuildNumber is uint bn ? $"Build {bn}" : "";
}

public sealed partial class SaveSlotsPage : Page
{
    public SaveSlotsPage()
    {
        InitializeComponent();
        Loaded += OnLoaded;
    }

    private void OnLoaded(object sender, RoutedEventArgs e)
    {
        var dir = @"D:\Development\NotAlterra-GUI\gvas-files";
        var savFiles = Directory.GetFiles(dir, "savegame_*.sav");
        if (savFiles.Length == 0) { StatusText.Text = "No save files found"; return; }

        var bakFiles = Directory.GetFiles(dir, "savegame_*.bak");
        var allBaks = bakFiles.Select(f => new FileInfo(f)).ToList();

        var slots = new List<SlotInfo>();
        foreach (var sav in savFiles)
        {
            var name = Path.GetFileNameWithoutExtension(sav);
            var parts = name.Split('_');
            if (parts.Length < 2 || !int.TryParse(parts[1], out int slot)) continue;

            var fi = new FileInfo(sav);
            var slotPrefix = $"savegame_{slot}";

            var backups = allBaks
                .Where(b => b.Name.StartsWith(slotPrefix, StringComparison.OrdinalIgnoreCase))
                .Select(b => new BakInfo(b.FullName, b.Length, b.LastWriteTime))
                .OrderBy(b => b.Name)
                .ToList();

            FullMetadata? meta = null;
            try { meta = GvasParser.ExtractFullMetadata(sav); } catch { }

            slots.Add(new SlotInfo(slot, sav, fi.Length, fi.LastWriteTime, meta, backups));
        }

        slots.Sort((a, b) => a.Number.CompareTo(b.Number));
        StatusText.Text = $"{slots.Count} slot(s)";
        SlotList.ItemsSource = slots;
    }
}
