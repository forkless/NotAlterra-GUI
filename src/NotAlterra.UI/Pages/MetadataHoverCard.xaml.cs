using Microsoft.UI.Xaml;
using NotAlterra.Gvas;
using NotAlterra.Services;

namespace NotAlterra_UI.Pages;

public sealed partial class MetadataHoverCard
{
    public MetadataHoverCard()
    {
        InitializeComponent();
    }

    public void ShowForSlot(SlotInfo si)
    {
        DisplayNameText.Text = !string.IsNullOrEmpty(si.Meta?.DisplayName)
            ? si.Meta.DisplayName
            : $"Slot {si.Number}";

        var meta = si.Meta;
        Populate(meta, si.SavPath, si.Size, si.Modified);
    }

    public void ShowForBak(BakInfo bi)
    {
        DisplayNameText.Text = !string.IsNullOrEmpty(bi.Meta?.DisplayName)
            ? bi.Meta.DisplayName
            : bi.Name;

        Populate(bi.Meta, bi.FilePath, bi.Size, bi.Modified);
    }

    private void Populate(FullMetadata? meta, string filePath, long fileSize, DateTime modified)
    {
        var fi = new FileInfo(filePath);

        if (meta != null)
        {
            SlotText.Text = meta.SlotName ?? "?";
            GameModeText.Text = FormatGameMode(meta.GameMode);
            LevelText.Text = meta.LevelName ?? "?";
            ModeText.Text = FormatMode(meta.IsOnline, meta.WasMultiplayer);
            BranchText.Text = meta.BuildBranch ?? "?";
            BuildText.Text = meta.BuildNumber is uint bn ? $"#{bn}" : "?";
            PlaytimeText.Text = meta.PlaytimeSeconds is double s
                ? $"{(int)s / 3600}h {(int)(s % 3600) / 60}m"
                : "?";
            SaveCountText.Text = meta.SavesCount?.ToString() ?? "?";
            SaveVerText.Text = meta.LatestVersion?.ToString() ?? "?";
            DataVerText.Text = meta.DataVersion?.ToString() ?? "?";

            CorruptionText.Text = !string.IsNullOrEmpty(meta.CorruptionReason)
                ? meta.CorruptionReason : "None";
        }
        else
        {
            SlotText.Text = "?";
            GameModeText.Text = "?";
            LevelText.Text = "?";
            ModeText.Text = "?";
            BranchText.Text = "?";
            BuildText.Text = "?";
            PlaytimeText.Text = "?";
            SaveCountText.Text = "?";
            SaveVerText.Text = "?";
            DataVerText.Text = "?";
            CorruptionText.Text = "None";
        }

        // File info (always available)
        FileText.Text = fi.Name;
        SizeText.Text = FormatSize(fi.Length);
        ModifiedText.Text = fi.LastWriteTime.ToString("yyyy-MM-dd HH:mm");
    }

    private static string FormatGameMode(string? raw)
    {
        if (string.IsNullOrEmpty(raw)) return "?";
        return raw.Replace("BP_", "").Replace("_C", "");
    }

    private static string FormatMode(bool isOnline, bool wasMultiplayer)
    {
        if (isOnline) return "Multiplayer";
        if (wasMultiplayer) return "Was Multiplayer";
        return "Single Player";
    }

    private static string FormatSize(long bytes)
    {
        return bytes switch
        {
            > 1_000_000 => $"{bytes / 1_000_000.0:F1} MB",
            > 1_000 => $"{bytes / 1_000.0:F1} KB",
            _ => $"{bytes} B"
        };
    }
}