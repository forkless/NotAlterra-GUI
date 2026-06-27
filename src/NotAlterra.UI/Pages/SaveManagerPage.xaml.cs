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
        Frame.Navigate(typeof(AboutPage));
    }

    private void OnBackups(object sender, RoutedEventArgs e)
    {
        Frame.Navigate(typeof(AboutPage));
    }

    private void OnIniConfig(object sender, RoutedEventArgs e)
    {
        Frame.Navigate(typeof(AboutPage));
    }
}
