using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using NotAlterra.Gvas;
using NotAlterra.Services;

namespace NotAlterra_UI.Pages;

public record BakInfo(FileInfo Info, FullMetadata? Meta)
{
    public bool IsCorrupted => Meta?.CorruptionReason != null || Size < 2000;
    public string CorruptionIcon => IsCorrupted ? "\uE7BA" : "";
    public string CorruptionFont => IsCorrupted ? "Segoe MDL2 Assets" : "";
    public string CorruptionTooltip => Meta?.CorruptionReason ?? "";
    public string CorruptionPrefix => CorruptionHelper.ExtractPrefix(CorruptionTooltip);
    public string CorruptionDetail => CorruptionHelper.ExtractDetail(CorruptionTooltip);
    public Visibility CorruptionVisible => IsCorrupted ? Visibility.Visible : Visibility.Collapsed;
    public string FilePath => Info.FullName;
    public string Name => Info.Name;
    public long Size => Info.Length;
    public DateTime Modified => Info.LastWriteTime;
    public string SizeDisplay => Size switch
    {
        > 1_000_000 => $"{Size / 1_000_000.0:F1} MB",
        > 1_000 => $"{Size / 1_000.0:F1} KB",
        _ => $"{Size} B"
    };
    public string ModifiedDisplay => Modified.ToString("yyyy-MM-dd HH:mm");
    public string PlaytimeDisplay => Meta?.PlaytimeSeconds is double s
        ? $"{(int)s / 3600}h {(int)(s % 3600) / 60}m" : "n/a";
    public string NameDisplay => !string.IsNullOrEmpty(Meta?.DisplayName)
        ? Meta.DisplayName : Name;
    public string ModeDisplay => Meta?.IsOnline == true ? "Multiplayer" :
        Meta?.WasMultiplayer == true ? "Was Multiplayer" : "Single Player";
    public string BuildDisplay => Meta?.BuildNumber is uint bn ? $"Build {bn}" : "";
}

public record SlotInfo(int Number, string SavPath, long Size, DateTime Modified, FullMetadata? Meta, List<BakInfo> Backups)
{
    public bool IsExpanded = false;
    public string ChevronGlyph => IsExpanded ? "\uE70D" : "\uE76C";
    public string ChevronFont => "Segoe MDL2 Assets";
    public Visibility BackupsVisibility => IsExpanded ? Visibility.Visible : Visibility.Collapsed;
    public bool IsCorrupted => Meta?.CorruptionReason != null || Size < 2000;
    public string CorruptionIcon => IsCorrupted ? "\uE7BA" : "";
    public string CorruptionFont => IsCorrupted ? "Segoe MDL2 Assets" : "";
    public string CorruptionTooltip => Meta?.CorruptionReason ?? (Size < 2000 ? "File appears truncated" : "");
    public string CorruptionPrefix => CorruptionHelper.ExtractPrefix(CorruptionTooltip);
    public string CorruptionDetail => CorruptionHelper.ExtractDetail(CorruptionTooltip);
    public Visibility CorruptionVisible => IsCorrupted ? Visibility.Visible : Visibility.Collapsed;
    public string SizeDisplay => Size switch
    {
        > 1_000_000 => $"{Size / 1_000_000.0:F1} MB",
        > 1_000 => $"{Size / 1_000.0:F1} KB",
        _ => $"{Size} B"
    };
    public string ModifiedDisplay => Modified.ToString("yyyy-MM-dd HH:mm");
    public string BakStatus => Backups.Count > 0 ? $"{Backups.Count} backup(s)" : "No backup";
    public string PlaytimeDisplay => Meta?.PlaytimeSeconds is double s
        ? $"{(int)s / 3600}h {(int)(s % 3600) / 60}m" : "n/a";
    public string ModeDisplay => Meta?.IsOnline == true ? "Multiplayer" :
        Meta?.WasMultiplayer == true ? "Was Multiplayer" : "Single Player";
    public string ModeTooltip => Meta?.IsOnline == true ? "Actively playing Multiplayer" :
        Meta?.WasMultiplayer == true ? "Save was previously in Multiplayer mode" : "Playing in Single Player mode";
    public string NameDisplay => !string.IsNullOrEmpty(Meta?.DisplayName)
        ? Meta.DisplayName : $"Slot {Number}";
    public string GameModeDisplay => !string.IsNullOrEmpty(Meta?.GameMode)
        ? Meta.GameMode.Replace("BP_", "").Replace("_C", "") : "";
    public string BuildDisplay => Meta?.BuildNumber is uint bn ? $"Build {bn}" : "";
}

static class CorruptionHelper
{
    public static string ExtractPrefix(string full)
    {
        if (string.IsNullOrEmpty(full)) return "";
        var start = full.LastIndexOf('(');
        return start >= 0 ? full[..start] : full;
    }
    public static string ExtractDetail(string full)
    {
        if (string.IsNullOrEmpty(full)) return "";
        var start = full.LastIndexOf('(');
        var end = full.LastIndexOf(')');
        if (start >= 0 && end > start)
            return full[start..(end + 1)];
        return "";
    }
}

public sealed partial class SaveSlotsPage : Page
{
    public SaveSlotsPage()
    {
        InitializeComponent();
        Loaded += OnLoaded;
    }

    private void OnGoBack(object sender, Microsoft.UI.Xaml.Input.TappedRoutedEventArgs e)
    {
        if (Frame.CanGoBack) Frame.GoBack();
    }

    private void OnLoaded(object? sender, RoutedEventArgs? e)
    {
        var cfg = AppConfig.LoadAppConfig();
        var dir = cfg.SaveFolder
            ?? System.IO.Path.Combine(
                Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData),
                "Subnautica2", "Saved", "SaveGames");
        if (!Directory.Exists(dir))
        {
            SlotCountText.Text = "?";
            StatusText.Text = "Save folder not found. Set it in Settings.";
            StatusText.Visibility = Visibility.Visible;
            return;
        }
        var savFiles = Directory.GetFiles(dir, "savegame_*.sav");
        if (savFiles.Length == 0) { SlotCountText.Text = "0"; StatusText.Visibility = Visibility.Visible; return; }

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
                .OrderByDescending(b =>
                {
                    var parts = System.IO.Path.GetFileNameWithoutExtension(b.Name).Split('_');
                    return parts.Length >= 3 && int.TryParse(parts[^1], out var idx) ? idx : 0;
                })
                .Select(b => { FullMetadata? m = null; try { m = GvasParser.ExtractFullMetadata(b.FullName); } catch { } return new BakInfo(b, m); })
                .ToList();

            FullMetadata? meta = null;
            try { meta = GvasParser.ExtractFullMetadata(sav); } catch { }

            slots.Add(new SlotInfo(slot, sav, fi.Length, fi.LastWriteTime, meta, backups));
        }

        slots.Sort((a, b) => a.Number.CompareTo(b.Number));
        SlotCountText.Text = $"{slots.Count}";
        SlotList.ItemsSource = slots;
    }

    private void OnToggleCard(object sender, Microsoft.UI.Xaml.Input.TappedRoutedEventArgs e)
    {
        if (sender is FrameworkElement fe && fe.DataContext is SlotInfo si)
        {
            si.IsExpanded = !si.IsExpanded;
            // Force re-bind by swapping ItemsSource
            var items = SlotList.ItemsSource as List<SlotInfo>;
            SlotList.ItemsSource = null;
            SlotList.ItemsSource = items;
        }
    }

    private async void OnRecover(object sender, RoutedEventArgs e)
    {
        if (sender is not Button btn) return;
        var bak = btn.DataContext as BakInfo;
        var filePath = btn.Tag as string;
        if (filePath == null) return;

        // Guard: warn about corruption, mode change, name change
        if (btn.XamlRoot != null)
        {
            var warnings = new List<string>();
            if (bak?.IsCorrupted == true)
                warnings.Add($"Backup is corrupt: {bak!.CorruptionTooltip}");

            // Parse current save metadata for comparison
            var fi = new FileInfo(filePath);
            var dir = fi.DirectoryName!;
            var name = Path.GetFileNameWithoutExtension(filePath);
            var parts = name.Split('_');
            FullMetadata? currentMeta = null;
            if (parts.Length >= 2 && int.TryParse(parts[1], out int slot))
            {
                var canonSav = Path.Combine(dir, $"savegame_{slot}.sav");
                if (File.Exists(canonSav))
                    try { currentMeta = GvasParser.ExtractFullMetadata(canonSav); } catch { }
            }
            if (currentMeta != null && bak?.Meta != null)
            {
                var currentMode = currentMeta.IsOnline ? "Multiplayer" :
                    currentMeta.WasMultiplayer ? "Was Multiplayer" : "Single Player";
                var backupMode = bak.Meta.IsOnline ? "Multiplayer" :
                    bak.Meta.WasMultiplayer ? "Was Multiplayer" : "Single Player";
                if (currentMode != backupMode)
                    warnings.Add($"Mode change: {currentMode} → {backupMode}");

                if (!string.IsNullOrEmpty(currentMeta.DisplayName) &&
                    !string.IsNullOrEmpty(bak.Meta.DisplayName) &&
                    currentMeta.DisplayName != bak.Meta.DisplayName)
                    warnings.Add($"Name change: \"{currentMeta.DisplayName}\" → \"{bak.Meta.DisplayName}\"");
            }

            var msg = warnings.Count > 0
                ? "The following issues were detected:\n\n• " + string.Join("\n• ", warnings) + "\n\nA snapshot will be created before proceeding."
                : "A snapshot will be created before recovery. Continue?";
            var dlg = new ContentDialog
            {
                XamlRoot = btn.XamlRoot,
                Title = "Recover Backup",
                Content = msg,
                PrimaryButtonText = warnings.Count > 0 ? "Recover anyways" : "Recover",
                CloseButtonText = "Cancel",
                DefaultButton = ContentDialogButton.Close
            };
            var result = await dlg.ShowAsync();
            if (result != ContentDialogResult.Primary) return;
        }

        try
        {
            // Derive canonical .sav path from backup filename
            var fi = new FileInfo(filePath);
            var dir = fi.DirectoryName!;
            var name = Path.GetFileNameWithoutExtension(filePath);
            var parts = name.Split('_');
            if (parts.Length < 2 || !int.TryParse(parts[1], out int slot))
            { StatusText.Text = "Could not determine slot number"; StatusText.Visibility = Visibility.Visible; return; }

            var canonSav = Path.Combine(dir, $"savegame_{slot}.sav");
            if (!File.Exists(canonSav))
            { StatusText.Text = "Canonical save file not found"; StatusText.Visibility = Visibility.Visible; return; }

            // Create pre-recovery snapshot (skip if identical to latest existing)
            string? snapName = null;
            var existingSnapshots = Directory.GetFiles(dir, $"savegame_{slot}_pre_recover_*.bak")
                .OrderByDescending(f => f)
                .ToArray();
            bool needSnapshot = true;
            if (existingSnapshots.Length > 0)
            {
                var latest = existingSnapshots[0];
                var currentBytes = File.ReadAllBytes(canonSav);
                var latestBytes = File.ReadAllBytes(latest);
                if (currentBytes.Length == latestBytes.Length &&
                    currentBytes.AsSpan().SequenceEqual(latestBytes.AsSpan()))
                    needSnapshot = false;
            }
            if (needSnapshot)
            {
                var timestamp = DateTime.Now.ToString("yyyyMMdd_HHmmss_fff");
                var snapPath = Path.Combine(dir, $"savegame_{slot}_pre_recover_{timestamp}.bak");
                File.Copy(canonSav, snapPath, overwrite: false);
                snapName = Path.GetFileName(snapPath);
                Guard.LogAction("RECOVER", $"Pre-recovery snapshot: {snapName}", "INFO");
            }
            else
            {
                Guard.LogAction("RECOVER", $"Snapshot skipped (identical to {Path.GetFileName(existingSnapshots[0])})", "INFO");
            }

            // Copy backup over canonical
            File.Copy(filePath, canonSav, overwrite: true);

            if (btn.XamlRoot != null)
            {
                var doneDlg = new ContentDialog
                {
                    XamlRoot = btn.XamlRoot,
                    Title = "Recovery Complete",
                    Content = $"Recovered from: {Path.GetFileName(filePath)}\n" + (snapName != null ? $"Snapshot: {snapName}" : "Snapshot skipped (current save already backed up)"),
                    PrimaryButtonText = "OK",
                    DefaultButton = ContentDialogButton.Primary
                };
                await doneDlg.ShowAsync();
            }

            // Reload slot list
            Loaded -= OnLoaded;
            OnLoaded(null, null!);
        }
        catch (Exception ex)
        {
            StatusText.Text = $"Recovery failed: {ex.Message}";
            StatusText.Visibility = Visibility.Visible;
        }
    }

    private async void OnDeleteBak(object sender, RoutedEventArgs e)
    {
        if (sender is not Button btn || btn.Tag is not string filePath) return;
        var fi = new System.IO.FileInfo(filePath);
        if (btn.XamlRoot != null)
        {
            var dlg = new ContentDialog
            {
                XamlRoot = btn.XamlRoot,
                Title = "Delete Backup",
                Content = $"Delete {fi.Name} permanently?",
                PrimaryButtonText = "Delete",
                CloseButtonText = "Cancel",
                DefaultButton = ContentDialogButton.Close
            };
            var result = await dlg.ShowAsync();
            if (result != ContentDialogResult.Primary) return;
        }
        try
        {
            File.Delete(filePath);
            Guard.LogAction("DELETE", $"{fi.Name}", "INFO");
            Loaded -= OnLoaded;
            OnLoaded(null, null!);
        }
        catch (Exception ex)
        {
            StatusText.Text = $"Delete failed: {ex.Message}";
            StatusText.Visibility = Visibility.Visible;
        }
    }
}
