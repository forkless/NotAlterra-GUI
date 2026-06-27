using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using NotAlterra.Gvas;
using NotAlterra.Services;

namespace NotAlterra_UI.Pages;

public sealed partial class HomePage : Page
{
    public HomePage()
    {
        InitializeComponent();
    }

    private async void OnScanSaves(object sender, RoutedEventArgs e)
    {
        ScanButton.IsEnabled = false;
        SavePathText.Text = "Scanning...";
        MetaList.Visibility = Visibility.Collapsed;
        ResultText.Text = "";

        var path = await Task.Run(() => SaveDiscovery.QuickDiscover());

        if (path != null)
            SavePathText.Text = $"Found saves at: {path}";
        else
        {
            SavePathText.Text = "No save folder found. Set it in Settings.";
            Guard.LogAction("SCAN", "QuickDiscover found nothing", "INFO");
        }
        ScanButton.IsEnabled = true;
    }

    private async void OnParseSave(object sender, RoutedEventArgs e)
    {
        ParseButton.IsEnabled = false;
        MetaList.Visibility = Visibility.Collapsed;
        SavePathText.Text = "Parsing...";

        var dir = await Task.Run(() => SaveDiscovery.QuickDiscover());
        if (dir == null)
        {
            SavePathText.Text = "No save folder found. Scan first.";
            ParseButton.IsEnabled = true;
            return;
        }

        var saves = await Task.Run(() =>
            Directory.GetFiles(dir, "savegame_*.sav").OrderBy(f => f).ToArray());
        if (saves.Length == 0)
        {
            SavePathText.Text = "No .sav files found.";
            ParseButton.IsEnabled = true;
            return;
        }

        try
        {
            var meta = await Task.Run(() => GvasParser.ExtractFullMetadata(saves[0]));

            var items = new List<KeyValuePair<string, string>>
            {
                new("File", Path.GetFileName(saves[0])),
                new("Slot", meta.SlotName ?? "?"),
                new("Display Name", meta.DisplayName ?? "?"),
                new("Game Mode", meta.GameMode ?? "?"),
                new("Level", meta.LevelName?.Split('/').Last() ?? "?"),
                new("Build Branch", meta.BuildBranch ?? "?"),
                new("Build Number", meta.BuildNumber?.ToString() ?? "?"),
                new("Saves Count", meta.SavesCount?.ToString() ?? "?"),
                new("Latest Version", meta.LatestVersion?.ToString() ?? "?"),
                new("Playtime", meta.PlaytimeSeconds.HasValue
                    ? $"{(int)(meta.PlaytimeSeconds.Value / 3600)}h {(int)(meta.PlaytimeSeconds.Value % 3600 / 60)}m"
                    : "?"),
            };

            if (meta.WasMultiplayer)
                items.Add(new("Mode", meta.IsOnline ? "Multiplayer" : "Was MP"));
            else
                items.Add(new("Mode", "Single Player"));

            MetaList.ItemsSource = items;
            MetaList.Visibility = Visibility.Visible;
            SavePathText.Text = $"Parsed: {Path.GetFileName(saves[0])}";
        }
        catch (Exception ex)
        {
            SavePathText.Text = $"Parse error: {ex.Message}";
        }
        ParseButton.IsEnabled = true;
    }

    private async void OnTestBackup(object sender, RoutedEventArgs e)
    {
        BackupButton.IsEnabled = false;
        ResultText.Text = "Creating backup...";

        var dir = await Task.Run(() => SaveDiscovery.QuickDiscover());
        if (dir == null)
        {
            ResultText.Text = "No save folder found. Scan first.";
            BackupButton.IsEnabled = true;
            return;
        }

        try
        {
            var result = await Task.Run(() => SaveOps.CreateFullBackup(dir));
            ResultText.Text = $"Backup created: {result.FilesCopied} files, " +
                              $"{(result.TotalSize / 1024.0):F0}KB → {Path.GetFileName(result.DestPath)}";
            Guard.LogAction("BACKUP", Path.GetFileName(result.DestPath), "OK");
        }
        catch (Exception ex)
        {
            ResultText.Text = $"Backup failed: {ex.Message}";
        }
        BackupButton.IsEnabled = true;
    }
}
