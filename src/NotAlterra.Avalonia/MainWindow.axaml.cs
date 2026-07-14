using Avalonia.Controls;
using Avalonia.Interactivity;

namespace NotAlterra.Avalonia;

public partial class MainWindow : Window
{
    private readonly ContentControl _content;

    public MainWindow()
    {
        InitializeComponent();
        Title = "NotAlterra — Subnautica 2 Save Manager";
        Width = 1200;
        Height = 800;
        MinWidth = 900;
        MinHeight = 600;

        _content = this.FindControl<ContentControl>("ContentArea")!;
        _content.Content = new Pages.HomePage();
    }

    private void NavigateHome(object? sender, RoutedEventArgs e)
        => _content.Content = new Pages.HomePage();

    private void NavigateSaves(object? sender, RoutedEventArgs e)
        => _content.Content = new Pages.SaveSlotsPage();

    private void NavigateAbout(object? sender, RoutedEventArgs e)
        => _content.Content = new Pages.AboutPage();

    private void NavigateSettings(object? sender, RoutedEventArgs e)
        => _content.Content = new Pages.SettingsPage();
}
