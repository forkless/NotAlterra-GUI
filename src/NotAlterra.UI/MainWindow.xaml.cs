using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using Microsoft.UI.Xaml.Media;
using Microsoft.UI.Xaml.Shapes;
using Microsoft.UI.Windowing;
using NotAlterra_UI.Pages;
using NotAlterra.Services;
using Windows.Media.Core;
using Windows.Media.Playback;
using Windows.UI;

namespace NotAlterra_UI;

public sealed partial class MainWindow : Window
{
    private Button? _activeBtn;

    public MainWindow()
    {
        InitializeComponent();
        ExtendsContentIntoTitleBar = true;
        SetTitleBar(AppTitleBar);
        AppWindow.TitleBar.PreferredHeightOption = TitleBarHeightOption.Tall;
        AppWindow.SetIcon("Assets/AppIcon.ico");

        ((FrameworkElement)Content).Loaded += async (_, _) =>
        {
            // Set up sidebar video
            SidebarVideo.SetMediaPlayer(new MediaPlayer
            {
                IsLoopingEnabled = true,
                AutoPlay = true,
                Source = MediaSource.CreateFromUri(new Uri("ms-appx:///Assets/loop.webm"))
            });

            // Sidebar breakpoints — only at threshold
            bool _wide = true;
            MainGrid.SizeChanged += (_, _) =>
            {
                bool now = MainGrid.ActualWidth >= 1000;
                if (now == _wide) return;
                _wide = now;
                MainGrid.ColumnDefinitions[0].Width = new GridLength(now ? 360 : 280);
            };

            // Default to Home
            SetActive(HomeBtn);
            NavFrame.Navigate(typeof(HomePage));

            // Game guard
            while (Guard.GameRunning()) { var d = new ContentDialog { XamlRoot = Content.XamlRoot, Title = "Subnautica 2 is running", Content = "Please save and close Subnautica 2 before using this tool.", PrimaryButtonText = "Retry", CloseButtonText = "Continue anyway", DefaultButton = ContentDialogButton.Primary }; var r = await d.ShowAsync(); if (r == ContentDialogResult.None) { Guard.LogAction("GUARD", "Proceeded while game running", "WARN"); break; } } };
    }

    private void NavBtn_Click(object sender, RoutedEventArgs e)
    {
        if (sender is not Button btn || btn.Tag is not string tag) return;
        SetActive(btn);
        switch (tag)
        {
            case "home": NavFrame.Navigate(typeof(HomePage)); break;
            case "about": NavFrame.Navigate(typeof(AboutPage)); break;
            default: NavFrame.Navigate(typeof(SettingsPage)); break;
        }
    }

    private void SetActive(Button btn)
    {
        if (_activeBtn != null)
        {
            if (_activeBtn.Content is Grid pg && pg.Children.Count > 0 && pg.Children[0] is Rectangle pr)
                pr.Visibility = Visibility.Collapsed;
        }
        if (btn.Content is Grid g && g.Children.Count > 0 && g.Children[0] is Rectangle r)
            r.Visibility = Visibility.Visible;
        _activeBtn = btn;
    }

    private void TitleBar_BackRequested(TitleBar s, object a) => NavFrame.GoBack();
}
