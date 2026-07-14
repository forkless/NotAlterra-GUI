using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using NotAlterra.Services;

namespace NotAlterra_UI.Pages;

public sealed partial class IniConfigPage : Page
{
    public IniConfigPage()
    {
        InitializeComponent();
        Loaded += OnLoaded;
    }

    private void OnLoaded(object sender, RoutedEventArgs e)
    {
        LoadBackupList();
    }

    private void OnGoBack(object sender, Microsoft.UI.Xaml.Input.TappedRoutedEventArgs e)
    {
        if (Frame.CanGoBack) Frame.GoBack();
    }

    private string GetConfigPath()
    {
        var saveFolder = AppConfig.LoadAppConfig().SaveFolder
            ?? SaveDiscovery.QuickDiscover()
            ?? @"D:\Development\NotAlterra-GUI\gvas-files";
        // Derive config path: save folder parent + Config
        var dir = System.IO.Path.GetDirectoryName(saveFolder?.TrimEnd('\\', '/'));
        if (dir != null)
            return System.IO.Path.Combine(dir, "Config", "Windows");
        return @"D:\Development\NotAlterra-GUI\gvas-files";
    }

    private void LoadBackupList()
    {
        var archives = SaveOps.ListIniBackups();
        if (archives.Count == 0)
        {
            StatusText.Text = "No INI backups found";
            StatusText.Visibility = Visibility.Visible;
            BackupList.ItemsSource = null;
            return;
        }
        StatusText.Visibility = Visibility.Collapsed;
        var items = archives.Select(a =>
        {
            var fi = new System.IO.FileInfo(a);
            return new BackupArchive(a, fi.Length, fi.LastWriteTime);
        }).ToList();
        BackupList.ItemsSource = items;
    }

    private async void OnCreateBackup(object sender, RoutedEventArgs e)
    {
        try
        {
            var configPath = GetConfigPath();
            var result = SaveOps.BackupIniFiles(configPath);
            if (XamlRoot != null)
            {
                var dlg = new ContentDialog
                {
                    XamlRoot = XamlRoot,
                    Title = "INI Backup Created",
                    Content = $"{result.FilesCopied} config files archived",
                    PrimaryButtonText = "OK",
                    DefaultButton = ContentDialogButton.Primary
                };
                await dlg.ShowAsync();
            }
        }
        catch (System.Exception ex)
        {
            if (XamlRoot != null)
            {
                var dlg = new ContentDialog
                {
                    XamlRoot = XamlRoot,
                    Title = "Backup Failed",
                    Content = $"Path: {GetConfigPath()}\n{ex.Message}",
                    PrimaryButtonText = "OK",
                    DefaultButton = ContentDialogButton.Primary
                };
                await dlg.ShowAsync();
            }
        }
        LoadBackupList();
    }

    private async void OnDeleteIni(object sender, RoutedEventArgs e)
    {
        if (!await MainWindow.CheckGameGuard(this.XamlRoot)) return;
        try
        {
            var configPath = GetConfigPath();
            var hasBackup = SaveOps.ListIniBackups().Count > 0;
            if (!hasBackup)
            {
                if (XamlRoot != null)
                {
                    var warnDlg = new ContentDialog
                    {
                        XamlRoot = XamlRoot,
                        Title = "No Backup Found",
                        Content = "INI Backup will be created before proceeding.",
                        PrimaryButtonText = "OK",
                        DefaultButton = ContentDialogButton.Primary
                    };
                    await warnDlg.ShowAsync();
                }
                SaveOps.BackupIniFiles(configPath);
            }

            if (XamlRoot != null)
            {
                var confirmDlg = new ContentDialog
                {
                    XamlRoot = XamlRoot,
                    Title = "Delete INI Files",
                    Content = "This will delete your Subnautica 2 UE5 config files so the game can regenerate them on next launch. Use this to fix mod conflicts or reset settings. A backup exists so you can restore later.\n\nContinue?",
                    PrimaryButtonText = "Delete",
                    CloseButtonText = "Cancel",
                    DefaultButton = ContentDialogButton.Close
                };
                var result = await confirmDlg.ShowAsync();
                if (result != ContentDialogResult.Primary) return;

                var count = SaveOps.DeleteIniFiles(configPath);
                var doneDlg = new ContentDialog
                {
                    XamlRoot = XamlRoot,
                    Title = "INI Files Deleted",
                    Content = $"{count} config files removed",
                    PrimaryButtonText = "OK",
                    DefaultButton = ContentDialogButton.Primary
                };
                await doneDlg.ShowAsync();
            }
        }
        catch (System.Exception ex)
        {
            if (XamlRoot != null)
            {
                var dlg = new ContentDialog
                {
                    XamlRoot = XamlRoot,
                    Title = "Delete Failed",
                    Content = $"Path: {GetConfigPath()}\n{ex.Message}",
                    PrimaryButtonText = "OK",
                    DefaultButton = ContentDialogButton.Primary
                };
                await dlg.ShowAsync();
            }
        }
    }

    private async void OnRestoreArchive(object sender, RoutedEventArgs e)
    {
        if (sender is not Button btn || btn.Tag is not string path) return;
        if (!await MainWindow.CheckGameGuard(this.XamlRoot)) return;
        try
        {
            var configPath = GetConfigPath();
            if (btn.XamlRoot != null)
            {
                var fi = new System.IO.FileInfo(path);
                var confirmDlg = new ContentDialog
                {
                    XamlRoot = btn.XamlRoot,
                    Title = "Restore INI Backup",
                    Content = $"Restore {fi.Name}?\n\nCurrent config will be backed up first.",
                    PrimaryButtonText = "Restore",
                    CloseButtonText = "Cancel",
                    DefaultButton = ContentDialogButton.Close
                };
                var result = await confirmDlg.ShowAsync();
                if (result != ContentDialogResult.Primary) return;

                var count = SaveOps.RestoreIniFiles(path, configPath);
                var doneDlg = new ContentDialog
                {
                    XamlRoot = btn.XamlRoot,
                    Title = "Restore Complete",
                    Content = $"{count} files restored from {fi.Name}",
                    PrimaryButtonText = "OK",
                    DefaultButton = ContentDialogButton.Primary
                };
                await doneDlg.ShowAsync();
            }
        }
        catch (System.Exception ex)
        {
            if (btn?.XamlRoot != null)
            {
                var dlg = new ContentDialog
                {
                    XamlRoot = btn.XamlRoot,
                    Title = "Restore Failed",
                    Content = $"Path: {GetConfigPath()}\n{ex.Message}",
                    PrimaryButtonText = "OK",
                    DefaultButton = ContentDialogButton.Primary
                };
                await dlg.ShowAsync();
            }
        }
        LoadBackupList();
    }

    private async void OnDeleteArchive(object sender, RoutedEventArgs e)
    {
        if (sender is not Button btn || btn.Tag is not string path) return;
        try
        {
            if (btn.XamlRoot != null)
            {
                var fi = new System.IO.FileInfo(path);
                var confirmDlg = new ContentDialog
                {
                    XamlRoot = btn.XamlRoot,
                    Title = "Delete Backup",
                    Content = $"Delete {fi.Name} permanently?",
                    PrimaryButtonText = "Delete",
                    CloseButtonText = "Cancel",
                    DefaultButton = ContentDialogButton.Close
                };
                var result = await confirmDlg.ShowAsync();
                if (result != ContentDialogResult.Primary) return;

                System.IO.File.Delete(path);
            }
        }
        catch (System.Exception ex)
        {
            if (btn?.XamlRoot != null)
            {
                var dlg = new ContentDialog
                {
                    XamlRoot = btn.XamlRoot,
                    Title = "Delete Failed",
                    Content = $"Path: {GetConfigPath()}\n{ex.Message}",
                    PrimaryButtonText = "OK",
                    DefaultButton = ContentDialogButton.Primary
                };
                await dlg.ShowAsync();
            }
        }
        LoadBackupList();
    }
}
