using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;

namespace NotAlterra_UI.Pages;

public sealed partial class SaveManagerPage : Page
{
    public SaveManagerPage()
    {
        InitializeComponent();
    }

    private void OnSaveSlots(object sender, RoutedEventArgs e)
    {
        Frame.Navigate(typeof(SaveSlotsPage));
    }

    private void OnBackups(object sender, RoutedEventArgs e)
    {
        Frame.Navigate(typeof(BackupsPage));
    }

    private void OnIniConfig(object sender, RoutedEventArgs e)
    {
        Frame.Navigate(typeof(IniConfigPage));
    }
}
