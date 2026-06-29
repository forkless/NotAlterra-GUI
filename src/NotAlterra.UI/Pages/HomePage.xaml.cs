using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using Microsoft.UI.Xaml.Media.Imaging;
using NotAlterra.Services;

namespace NotAlterra_UI.Pages;

public sealed partial class HomePage : Page
{
    public HomePage()
    {
        var cfg = AppConfig.LoadAppConfig();
        var saveFolder = cfg.SaveFolder
            ?? System.IO.Path.Combine(
                Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData),
                "Subnautica2", "Saved", "SaveGames");
        var backupRoot = AppConfig.GetBackupRoot();

        InitializeComponent();
        var localAppData = Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData);
        var userProfile = Environment.GetFolderPath(Environment.SpecialFolder.UserProfile);
        SavePathRun.Text = saveFolder.Replace(localAppData, "%LOCALAPPDATA%").Replace(userProfile, "%USERPROFILE%");
        BackupPathRun.Text = backupRoot.Replace(localAppData, "%LOCALAPPDATA%").Replace(userProfile, "%USERPROFILE%");

        Loaded += (_, _) =>
        {
            var path = System.IO.Path.Combine(AppContext.BaseDirectory, "Assets", "AppIcon256.png");
            LogoImage.Source = new BitmapImage(new Uri(path));
        };
    }
}
