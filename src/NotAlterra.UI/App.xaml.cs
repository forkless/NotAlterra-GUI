using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using NotAlterra.Services;

namespace NotAlterra_UI;

public partial class App : Application
{
    private Window? _window;
    public static Window? MainWindow => (Current as App)?._window;

    public App()
    {
        InitializeComponent();
    }

    protected override void OnLaunched(LaunchActivatedEventArgs args)
    {
        _window = new MainWindow();
        _window.Activate();

        if (!Services.AppConfig.DisclaimerAccepted())
            _ = ShowDisclaimerAsync();
    }

    private async Task ShowDisclaimerAsync()
    {
        var dialog = new ContentDialog
        {
            XamlRoot = _window!.Content.XamlRoot,
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
                    new TextBlock
                    {
                        TextWrapping = TextWrapping.Wrap,
                        Text = "NotAlterra is an unofficial fan-made utility for managing Subnautica 2 save files. It is not affiliated with or endorsed by KRAFTON or Unknown Worlds Entertainment."
                    },
                    new TextBlock
                    {
                        TextWrapping = TextWrapping.Wrap,
                        Text = "This tool is a read-only metadata inspector and backup manager. It does not edit .sav files in-place."
                    },
                    new TextBlock
                    {
                        TextWrapping = TextWrapping.Wrap,
                        Text = "NotAlterra makes no network connections. No telemetry. No data leaves your machine. Configuration files are stored locally in plain text."
                    },
                    new TextBlock
                    {
                        TextWrapping = TextWrapping.Wrap,
                        FontWeight = Microsoft.UI.Text.FontWeights.SemiBold,
                        Text = "This software is provided 'as is', without warranty of any kind. Use at your own risk."
                    },
                    new TextBlock
                    {
                        TextWrapping = TextWrapping.Wrap,
                        Opacity = 0.6,
                        Text = "MIT License — see LICENSE.md for full terms."
                    }
                }
            }
        };

        var result = await dialog.ShowAsync();

        if (result == ContentDialogResult.Primary)
        {
            AppConfig.AcceptDisclaimer();
        }
        else
        {
            Environment.Exit(0);
        }
    }
}
