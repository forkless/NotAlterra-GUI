using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using NotAlterra.Services;

namespace NotAlterra_UI;

public partial class App : Application
{
    private NativeSplash? _splash;
    public static Window? MainWindow => (Current as App)?._window;
    private Window? _window;

    public App()
    {
        InitializeComponent();
    }

    protected override async void OnLaunched(LaunchActivatedEventArgs args)
    {
        _splash = new NativeSplash();
        var pngPath = System.IO.Path.Combine(AppContext.BaseDirectory, "Assets", "splash.png");
        _splash.Show(pngPath, 3000);

        var tcs = new TaskCompletionSource();
        _splash.Closed += () => tcs.TrySetResult();
        await tcs.Task;

        await Task.Delay(300);

        _window = new MainWindow();
        _window.Activate();
        
    }
}
