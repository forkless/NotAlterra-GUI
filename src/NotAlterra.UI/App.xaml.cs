using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using NotAlterra.Services;

namespace NotAlterra_UI;

public partial class App : Application
{
    private Window? _window;
    private NativeSplash? _splash;
    public static Window? MainWindow => (Current as App)?._window;

    public App()
    {
        InitializeComponent();
    }

    protected override async void OnLaunched(LaunchActivatedEventArgs args)
    {
        // Native layered splash before any WinUI content
        _splash = new NativeSplash();
        var pngPath = System.IO.Path.Combine(AppContext.BaseDirectory, "Assets", "splash.png");
        _splash.Show(pngPath, 3000);

        // Wait for splash Closed event
        var tcs = new TaskCompletionSource();
        _splash.Closed += () => tcs.TrySetResult();
        await tcs.Task;

        // Brief pause so splash disappearing is visible before app appears
        await Task.Delay(300);

        _window = new MainWindow();
        _window.Activate();

        if (!AppConfig.DisclaimerAccepted())
            _window.DispatcherQueue.TryEnqueue(async () =>
            {
                await Task.Delay(200);
                await ShowDisclaimerAsync();
            });
    }

    private async Task ShowDisclaimerAsync()
    {
        while (_window!.Content?.XamlRoot is null)
            await Task.Delay(100);

        var dialog = new ContentDialog
        {
            XamlRoot = _window.Content.XamlRoot,
            Title = "NotAlterra — Disclaimer",
            CloseButtonText = "Decline",
            PrimaryButtonText = "Accept",
            DefaultButton = ContentDialogButton.Primary,
            Content = new StackPanel
            {
                Spacing = 12,
                MaxWidth = 500,
                Children =
                {
                    new TextBlock { TextWrapping = TextWrapping.Wrap, Text = "NotAlterra is an unofficial fan-made utility for managing Subnautica 2 save files. It is not affiliated with or endorsed by KRAFTON or Unknown Worlds Entertainment." },
                    new TextBlock { TextWrapping = TextWrapping.Wrap, Text = "This tool is a read-only metadata inspector and backup manager. It does not edit .sav files in-place." },
                    new TextBlock { TextWrapping = TextWrapping.Wrap, Text = "NotAlterra makes no network connections. No telemetry. No data leaves your machine." },
                    new TextBlock { TextWrapping = TextWrapping.Wrap, FontWeight = Microsoft.UI.Text.FontWeights.SemiBold, Text = "This software is provided 'as is', without warranty of any kind. Use at your own risk." },
                    new TextBlock { TextWrapping = TextWrapping.Wrap, Opacity = 0.6, Text = "MIT License - see LICENSE.md for full terms." }
                }
            }
        };

        var result = await dialog.ShowAsync();
        if (result == ContentDialogResult.Primary)
            AppConfig.AcceptDisclaimer();
        else
            Environment.Exit(0);
    }
}
