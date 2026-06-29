// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using Microsoft.Win32;
using NotAlterra.Services;

namespace NotAlterra_UI.Pages;

public sealed partial class SettingsPage : Page
{
    public SettingsPage()
    {
        InitializeComponent();
        Loaded += OnLoaded;
    }

    private void OnLoaded(object sender, RoutedEventArgs e)
    {
        var cfg = AppConfig.LoadAppConfig();
        if (cfg.SaveFolder != null)
            SaveFolderBox.Text = cfg.SaveFolder;
        if (cfg.BackupRoot != null)
            BackupFolderBox.Text = cfg.BackupRoot;
    }

    private async void OnBrowseFolder(object sender, RoutedEventArgs e)
    {
        var picker = new Windows.Storage.Pickers.FolderPicker();
        picker.FileTypeFilter.Add("*");

        var hwnd = WinRT.Interop.WindowNative.GetWindowHandle(App.MainWindow!);
        WinRT.Interop.InitializeWithWindow.Initialize(picker, hwnd);

        var folder = await picker.PickSingleFolderAsync();
        if (folder != null)
        {
            SaveFolderBox.Text = folder.Path;
            AppConfig.SaveAppConfig(folder.Path, null);
        }
    }

    private async void OnBrowseBackupFolder(object sender, RoutedEventArgs e)
    {
        var picker = new Windows.Storage.Pickers.FolderPicker();
        picker.FileTypeFilter.Add("*");

        var hwnd = WinRT.Interop.WindowNative.GetWindowHandle(App.MainWindow!);
        WinRT.Interop.InitializeWithWindow.Initialize(picker, hwnd);

        var folder = await picker.PickSingleFolderAsync();
        if (folder != null)
        {
            BackupFolderBox.Text = folder.Path;
            AppConfig.SaveAppConfig(null, folder.Path);
        }
    }

    private void OnResetSaveFolder(object sender, RoutedEventArgs e)
    {
        using var key = Registry.CurrentUser.OpenSubKey(@"Software\NotAlterra", writable: true);
        key?.DeleteValue("SaveFolder", throwOnMissingValue: false);
        SaveFolderBox.Text = "";
    }

    private void OnResetBackupFolder(object sender, RoutedEventArgs e)
    {
        using var key = Registry.CurrentUser.OpenSubKey(@"Software\NotAlterra", writable: true);
        key?.DeleteValue("BackupRoot", throwOnMissingValue: false);
        BackupFolderBox.Text = "";
    }
}
