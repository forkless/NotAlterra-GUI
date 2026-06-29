using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using Microsoft.UI.Xaml.Media.Imaging;

namespace NotAlterra_UI.Pages;

public sealed partial class HomePage : Page
{
    public HomePage()
    {
        InitializeComponent();
        Loaded += (_, _) =>
        {
            var path = System.IO.Path.Combine(AppContext.BaseDirectory, "Assets", "AppIcon256.png");
            LogoImage.Source = new BitmapImage(new Uri(path));
        };
    }
}
