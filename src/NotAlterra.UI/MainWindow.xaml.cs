using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using Microsoft.UI.Xaml.Media;
using Microsoft.UI.Windowing;
using NotAlterra_UI.Pages;
using NotAlterra.Services;
using Windows.Media.Core;
using Windows.Media.Playback;
using Windows.UI;

namespace NotAlterra_UI;

public sealed partial class MainWindow : Window
{
    public MainWindow()
    {
        InitializeComponent();
        ExtendsContentIntoTitleBar = true;
        SetTitleBar(AppTitleBar);
        AppWindow.TitleBar.PreferredHeightOption = TitleBarHeightOption.Tall;
        AppWindow.SetIcon("Assets/AppIcon.ico");

        NavView.Loaded += (_, _) =>
        {
            var pane = F<Grid>(NavView, "PaneContentGrid");
            if (pane == null) return;
            var src = MediaSource.CreateFromUri(new Uri("ms-appx:///Assets/loop.webm"));
            var player = new MediaPlayer { IsLoopingEnabled = true, AutoPlay = true, Source = src };
            var mpe = new MediaPlayerElement { Stretch = Stretch.UniformToFill, AreTransportControlsEnabled = false };
            mpe.SetMediaPlayer(player);
            Grid.SetRow(mpe, 0);
            Grid.SetRowSpan(mpe, 10);
            Canvas.SetZIndex(mpe, -1);
            pane.Children.Add(mpe);

            // Hide selection highlight on menu items
            var clear = new SolidColorBrush(Color.FromArgb(0, 0, 0, 0));
            foreach (var i in NavView.MenuItems)
                if (i is NavigationViewItem nvi) { nvi.Resources["NavigationViewItemBackgroundSelected"] = clear; nvi.Resources["NavigationViewItemBackgroundSelectedPointerOver"] = clear; }
            if (NavView.SettingsItem is NavigationViewItem svi) { svi.Resources["NavigationViewItemBackgroundSelected"] = clear; svi.Resources["NavigationViewItemBackgroundSelectedPointerOver"] = clear; }
        };

        ((FrameworkElement)Content).Loaded += async (_, _) => { while (Guard.GameRunning()) { var d = new ContentDialog { XamlRoot = Content.XamlRoot, Title = "Subnautica 2 is running", Content = "Please save and close Subnautica 2 before using this tool.", PrimaryButtonText = "Retry", CloseButtonText = "Continue anyway", DefaultButton = ContentDialogButton.Primary }; var r = await d.ShowAsync(); if (r == ContentDialogResult.None) { Guard.LogAction("GUARD", "Proceeded while game running", "WARN"); break; } } };
    }

    private static T? F<T>(DependencyObject p, string n) where T : DependencyObject
    {
        for (int i = 0; i < VisualTreeHelper.GetChildrenCount(p); i++) { var c = VisualTreeHelper.GetChild(p, i); if (c is T t && c is FrameworkElement fe && fe.Name == n) return t; var f = F<T>(c, n); if (f != null) return f; }
        return null;
    }

    private void TitleBar_PaneToggleRequested(TitleBar s, object a) => NavView.IsPaneOpen = !NavView.IsPaneOpen;
    private void TitleBar_BackRequested(TitleBar s, object a) => NavFrame.GoBack();
    private void NavView_SelectionChanged(NavigationView s, NavigationViewSelectionChangedEventArgs a) { if (a.IsSettingsSelected) NavFrame.Navigate(typeof(SettingsPage)); else if (a.SelectedItem is NavigationViewItem i) switch (i.Tag) { case "home": NavFrame.Navigate(typeof(HomePage)); break; case "about": NavFrame.Navigate(typeof(AboutPage)); break; } }
}
