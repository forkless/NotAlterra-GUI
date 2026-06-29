using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using NotAlterra.Services;

namespace NotAlterra_UI.Pages;

public record BackupArchive(string Path, long Size, DateTime Date)
{
    public string Name => System.IO.Path.GetFileName(Path);
    public string SizeDisplay => Size switch
    {
        > 100_000_000 => $"{Size / 1_000_000.0:F1} MB",
        > 1_000_000 => $"{Size / 1_000_000.0:F1} MB",
        > 1_000 => $"{Size / 1_000.0:F1} KB",
        _ => $"{Size} B"
    };
    public string DateDisplay => Date.ToString("MMMM dd, yyyy HH:mm");
}

public sealed partial class BackupsPage : Page
{
    public BackupsPage()
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

    private void LoadBackupList()
    {
        var archives = SaveOps.ListFullBackups();
        if (archives.Count == 0)
        {
            StatusText.Text = "No backups found";
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
            var saveFolder = AppConfig.LoadAppConfig().SaveFolder
                ?? SaveDiscovery.QuickDiscover()
                ?? @"D:\Development\NotAlterra-GUI\gvas-files";

            var result = SaveOps.CreateFullBackup(saveFolder);
            if (XamlRoot != null)
            {
                var dlg = new ContentDialog
                {
                    XamlRoot = XamlRoot,
                    Title = "Backup Created",
                    Content = $"{result.FilesCopied} save files archived\n{result.TotalSize / 1000} KB",
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
                    Content = ex.Message,
                    PrimaryButtonText = "OK",
                    DefaultButton = ContentDialogButton.Primary
                };
                await dlg.ShowAsync();
            }
        }
        LoadBackupList();
    }

    private async void OnRestoreBackup(object sender, RoutedEventArgs e)
    {
        var picker = new Windows.Storage.Pickers.FileOpenPicker
        {
            ViewMode = Windows.Storage.Pickers.PickerViewMode.List,
            SuggestedStartLocation = Windows.Storage.Pickers.PickerLocationId.Downloads
        };
        picker.FileTypeFilter.Add(".tar.gz");

        var hwnd = WinRT.Interop.WindowNative.GetWindowHandle(App.MainWindow);
        WinRT.Interop.InitializeWithWindow.Initialize(picker, hwnd);

        var file = await picker.PickSingleFileAsync();
        if (file == null) return;

        try
        {
            var saveFolder = AppConfig.LoadAppConfig().SaveFolder
                ?? SaveDiscovery.QuickDiscover()
                ?? @"D:\Development\NotAlterra-GUI\gvas-files";

            if (XamlRoot != null)
            {
                var confirmDlg = new ContentDialog
                {
                    XamlRoot = XamlRoot,
                    Title = "Restore Backup",
                    Content = $"This will overwrite your current save files.\nA pre-restore backup will be created first.\n\nContinue?",
                    PrimaryButtonText = "Restore",
                    CloseButtonText = "Cancel",
                    DefaultButton = ContentDialogButton.Close
                };
                var result = await confirmDlg.ShowAsync();
                if (result != ContentDialogResult.Primary) return;

                var count = SaveOps.RestoreFullBackup(file.Path, saveFolder);
                var doneDlg = new ContentDialog
                {
                    XamlRoot = XamlRoot,
                    Title = "Restore Complete",
                    Content = $"{count} files restored from {file.Name}",
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
                    Title = "Restore Failed",
                    Content = ex.Message,
                    PrimaryButtonText = "OK",
                    DefaultButton = ContentDialogButton.Primary
                };
                await dlg.ShowAsync();
            }
        }
        LoadBackupList();
    }

    private async void OnRestoreArchive(object sender, RoutedEventArgs e)
    {
        if (sender is not Button btn || btn.Tag is not string path) return;
        if (!await MainWindow.CheckGameGuard(this.XamlRoot)) return;
        try
        {
            var saveFolder = AppConfig.LoadAppConfig().SaveFolder
                ?? SaveDiscovery.QuickDiscover()
                ?? @"D:\Development\NotAlterra-GUI\gvas-files";

            if (btn.XamlRoot != null)
            {
                var fi = new System.IO.FileInfo(path);
                var confirmDlg = new ContentDialog
                {
                    XamlRoot = btn.XamlRoot,
                    Title = "Restore Backup",
                    Content = $"Restore {fi.Name}?\n\nThis will overwrite your current save files.\nA pre-restore backup will be created first.",
                    PrimaryButtonText = "Restore",
                    CloseButtonText = "Cancel",
                    DefaultButton = ContentDialogButton.Close
                };
                var result = await confirmDlg.ShowAsync();
                if (result != ContentDialogResult.Primary) return;

                var count = SaveOps.RestoreFullBackup(path, saveFolder);
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
            if (XamlRoot != null)
            {
                var dlg = new ContentDialog
                {
                    XamlRoot = btn.XamlRoot,
                    Title = "Restore Failed",
                    Content = ex.Message,
                    PrimaryButtonText = "OK",
                    DefaultButton = ContentDialogButton.Primary
                };
                await dlg.ShowAsync();
            }
        }
        LoadBackupList();
    }

    private async void OnCheckIntegrity(object sender, RoutedEventArgs e)
    {
        if (sender is not Button btn || btn.Tag is not string path) return;
        try
        {
            var (ok, details) = SaveOps.VerifyTarGzIntegrity(path);
            var fi = new System.IO.FileInfo(path);
            if (btn.XamlRoot != null)
            {
                var dlg = new ContentDialog
                {
                    XamlRoot = btn.XamlRoot,
                    Title = ok ? "Integrity Check Passed" : "Integrity Check Failed",
                    Content = $"{fi.Name}: {details}",
                    PrimaryButtonText = "OK",
                    DefaultButton = ContentDialogButton.Primary
                };
                await dlg.ShowAsync();
            }
        }
        catch (System.Exception ex)
        {
            if (btn?.XamlRoot != null)
            {
                var dlg = new ContentDialog
                {
                    XamlRoot = btn.XamlRoot,
                    Title = "Check Failed",
                    Content = ex.Message,
                    PrimaryButtonText = "OK",
                    DefaultButton = ContentDialogButton.Primary
                };
                await dlg.ShowAsync();
            }
        }
    }

    private async void OnDeleteArchive(object sender, RoutedEventArgs e)
    {
        if (sender is not Button btn || btn.Tag is not string path) return;
        if (!await MainWindow.CheckGameGuard(this.XamlRoot)) return;
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
            if (XamlRoot != null)
            {
                var dlg = new ContentDialog
                {
                    XamlRoot = btn.XamlRoot,
                    Title = "Delete Failed",
                    Content = ex.Message,
                    PrimaryButtonText = "OK",
                    DefaultButton = ContentDialogButton.Primary
                };
                await dlg.ShowAsync();
            }
        }
        LoadBackupList();
    }
}
