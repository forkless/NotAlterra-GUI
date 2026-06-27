using Microsoft.UI.Composition;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using Microsoft.UI.Xaml.Hosting;
using Microsoft.UI.Xaml.Media;
using Microsoft.UI.Windowing;
using NotAlterra_UI.Pages;
using NotAlterra.Services;
using System.Numerics;
using Windows.UI;

namespace NotAlterra_UI;

public sealed partial class MainWindow : Window
{
    private ImageBrush? _bg;
    private Visual? _v;
    private Grid? _r;
    private Compositor? _comp;
    private float _opacity = 0.85f;
    private float _bandWidth = 0.45f;
    private float _speedFactor = 0.5f;
    private bool _debug;

    float Norm(double v) => (float)(v - 50) / 100f;

    public MainWindow()
    {
        InitializeComponent();
        ExtendsContentIntoTitleBar = true;
        SetTitleBar(AppTitleBar);
        AppWindow.TitleBar.PreferredHeightOption = TitleBarHeightOption.Tall;
        AppWindow.SetIcon("Assets/AppIcon.ico");

        StrengthSlider.StepFrequency = 1;
        SpeedSlider.StepFrequency = 1;
        OpacitySlider.StepFrequency = 1;
        WidthSlider.StepFrequency = 1;
        StrengthSlider.ValueChanged += (_, e) => { float s = Norm(e.NewValue); if (_v != null) _v.Opacity = _opacity * (0.15f + 0.85f * s); };
        SpeedSlider.ValueChanged += (_, e) => { _speedFactor = Norm(e.NewValue); RebuildFx(); };
        OpacitySlider.ValueChanged += (_, e) => { _opacity = Norm(e.NewValue); if (_v != null) _v.Opacity = _opacity; };
        WidthSlider.ValueChanged += (_, e) => { _bandWidth = 0.1f + Norm(e.NewValue) * 0.6f; RebuildFx(); };

        NavView.Loaded += (_, _) => { _r = F<Grid>(NavView, "PaneContentGrid"); Bg(); Fx(); Font(); };
        ((FrameworkElement)Content).Loaded += async (_, _) => { while (Guard.GameRunning()) { var d = new ContentDialog { XamlRoot = Content.XamlRoot, Title = "Subnautica 2 is running", Content = "Please save and close Subnautica 2 before using this tool.", PrimaryButtonText = "Retry", CloseButtonText = "Continue anyway", DefaultButton = ContentDialogButton.Primary }; var r = await d.ShowAsync(); if (r == ContentDialogResult.None) { Guard.LogAction("GUARD", "Proceeded while game running", "WARN"); break; } } };
    }

    private void Bg()
    {
        if (_r == null) return;
        _bg = new ImageBrush { ImageSource = new Microsoft.UI.Xaml.Media.Imaging.BitmapImage(new Uri("ms-appx:///Assets/sn2_sidepanel.jpg")), Stretch = Stretch.UniformToFill, AlignmentX = AlignmentX.Left };
        _r.Background = _bg;
    }

    private void Fx()
    {
        if (_r == null) return;
        _comp = ElementCompositionPreview.GetElementVisual(NavView).Compositor;
        BuildGradient(_bandWidth);
    }

    private void BuildGradient(float bandWidth)
    {
        if (_r == null || _comp == null) return;
        var c = _comp;

        var root = c.CreateContainerVisual();
        root.Size = new Vector2((float)_r.ActualWidth, (float)_r.ActualHeight);
        root.IsVisible = _v?.IsVisible ?? false;
        root.Opacity = _opacity;

        float baseDur = 5f + MathF.Pow(1f - _speedFactor, 3f) * 55f;
        float rx = 0.12f + bandWidth * 0.35f;
        float ry = 0.1f + bandWidth * 0.28f;
        float range = 0.04f + bandWidth * 0.06f;
        var ease = c.CreateCubicBezierEasingFunction(new Vector2(0.42f, 0), new Vector2(0.58f, 1));

        // Spots 1 & 3 go right, spot 2 goes left
        for (int n = 0; n < 3; n++)
        {
            float cx = 0.25f + n * 0.25f, cy = 0.35f + n * 0.15f;
            float delay = (float)Random.Shared.NextDouble() * baseDur * (1f + n * 0.15f);
            bool invert = n == 1; // spot 2 goes opposite direction

            var g = c.CreateRadialGradientBrush();
            g.EllipseRadius = new Vector2(rx, ry);
            g.EllipseCenter = new Vector2(cx, cy);
            byte pa = (byte)(_debug ? 200 : 90);
            if (_debug)
            {
                g.ColorStops.Add(c.CreateColorGradientStop(0f, Color.FromArgb(pa, 255, 100, 100)));
                g.ColorStops.Add(c.CreateColorGradientStop(0.05f, Color.FromArgb(pa, 255, 100, 100)));
                g.ColorStops.Add(c.CreateColorGradientStop(0.08f, Color.FromArgb(0, 0, 0, 0)));
            }
            else
            {
                g.ColorStops.Add(c.CreateColorGradientStop(0f, Color.FromArgb(pa, 100, 200, 255)));
                g.ColorStops.Add(c.CreateColorGradientStop(0.3f, Color.FromArgb((byte)(pa / 3), 100, 200, 255)));
                g.ColorStops.Add(c.CreateColorGradientStop(0.65f, Color.FromArgb(0, 0, 0, 0)));
            }

            var sv = c.CreateSpriteVisual();
            sv.Size = root.Size; sv.Brush = g;
            root.Children.InsertAtTop(sv);

            // Varying-width horizontal sway — seamless loop
            float[] peaks = { 1.0f, 0.8f, 0.6f, 0.9f, 1.3f, 0.7f, 1.0f };
            int k = peaks.Length;
            var ax = c.CreateScalarKeyFrameAnimation();
            for (int p = 0; p < k; p++)
            {
                float t = (float)p / (k - 1);
                float dir = (p % 2 == 0) ? 1f : -1f;
                float offset = dir * range * peaks[p] * (invert ? -1f : 1f);
                ax.InsertKeyFrame(t, cx + offset, ease);
            }
            // keyframe 1 = same as keyframe 0 → seamless loop
            ax.Duration = TimeSpan.FromSeconds(baseDur * (1f + n * 0.15f));
            ax.IterationBehavior = AnimationIterationBehavior.Forever;
            ax.DelayTime = TimeSpan.FromSeconds(delay);
            g.StartAnimation("EllipseCenter.X", ax);

            // No vertical animation — pure horizontal
        }

        _v = root;
        ElementCompositionPreview.SetElementChildVisual(_r, root);
    }

    private void RebuildFx()
    {
        if (_r != null && _comp != null) BuildGradient(_bandWidth);
    }

    private void Tgl(bool on) { if (_v != null) _v.IsVisible = on; }
    private void WaterRippleToggle_Toggled(object s, RoutedEventArgs e) => Tgl(WaterRippleToggle.IsOn);
    private void DebugToggle_Toggled(object s, RoutedEventArgs e) { _debug = DebugToggle.IsOn; RebuildFx(); }
    private void FontColorCombo_SelectionChanged(object s, SelectionChangedEventArgs e) => Font();

    private Color C() => FontColorCombo.SelectedIndex switch
    {
        0 => Color.FromArgb(255, 255, 255, 255), 1 => Color.FromArgb(255, 0, 0, 0),
        2 => Color.FromArgb(255, 173, 216, 230), 3 => Color.FromArgb(255, 0, 128, 128),
        4 => Color.FromArgb(255, 0, 128, 0), 5 => Color.FromArgb(255, 255, 165, 0),
        6 => Color.FromArgb(255, 255, 0, 0), 7 => Color.FromArgb(255, 255, 255, 0),
        _ => Color.FromArgb(255, 255, 255, 255),
    };

    private void Font() { var c = C(); foreach (var i in NavView.MenuItems) if (i is NavigationViewItem nvi) { nvi.FontSize = 14; nvi.Foreground = new SolidColorBrush(c); } }

    private static T? F<T>(DependencyObject p, string n) where T : DependencyObject
    {
        for (int i = 0; i < VisualTreeHelper.GetChildrenCount(p); i++) { var c = VisualTreeHelper.GetChild(p, i); if (c is T t && c is FrameworkElement fe && fe.Name == n) return t; var f = F<T>(c, n); if (f != null) return f; }
        return null;
    }

    private void TitleBar_PaneToggleRequested(TitleBar s, object a) => NavView.IsPaneOpen = !NavView.IsPaneOpen;
    private void TitleBar_BackRequested(TitleBar s, object a) => NavFrame.GoBack();
    private void NavView_SelectionChanged(NavigationView s, NavigationViewSelectionChangedEventArgs a) { if (a.IsSettingsSelected) NavFrame.Navigate(typeof(SettingsPage)); else if (a.SelectedItem is NavigationViewItem i) switch (i.Tag) { case "home": NavFrame.Navigate(typeof(HomePage)); break; case "about": NavFrame.Navigate(typeof(AboutPage)); break; } }
}
