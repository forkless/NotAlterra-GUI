using Microsoft.UI;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using Microsoft.UI.Xaml.Media;
using Microsoft.UI.Xaml.Shapes;
using Microsoft.UI.Windowing;
using NotAlterra_UI.Pages;
using NotAlterra.Services;
using System.Runtime.InteropServices;
using Windows.Graphics;
using Windows.Media.Core;
using Windows.Media.Playback;
using Windows.UI;
using WinRT.Interop;

namespace NotAlterra_UI;

public sealed partial class MainWindow : Window
{
    private Button? _activeBtn;
    private IntPtr _oldWndProc;
    private WndProcDelegate? _wndProcRef;
    private int _minW = 1200, _minH = 800;
    private delegate IntPtr WndProcDelegate(IntPtr hWnd, uint msg, IntPtr wParam, IntPtr lParam);

    [DllImport("user32.dll", EntryPoint = "SetWindowLongPtr")]
    private static extern IntPtr SetWindowLongPtr64(IntPtr hWnd, int nIndex, IntPtr dwNewLong);

    [DllImport("user32.dll")]
    private static extern IntPtr CallWindowProc(IntPtr lpPrevWndFunc, IntPtr hWnd, uint msg, IntPtr wParam, IntPtr lParam);

    [StructLayout(LayoutKind.Sequential)]
    private struct POINT { public int X; public int Y; }

    [StructLayout(LayoutKind.Sequential)]
    private struct MINMAXINFO
    {
        public POINT ptReserved;
        public POINT ptMaxSize;
        public POINT ptMaxPosition;
        public POINT ptMinTrackSize;
        public POINT ptMaxTrackSize;
    }

    private const int GWLP_WNDPROC = -4;
    private const uint WM_GETMINMAXINFO = 0x24;

    /// Position and size the window. Must be called after Activate().
    public void PositionWindow()
    {
        var area = DisplayArea.Primary.WorkArea;
        _minW = (int)(area.Width * 0.65);
        _minH = (int)(area.Height * 0.75);
        int initW = Math.Max(_minW, (int)(area.Width * 0.8));
        int initH = Math.Max(_minH, (int)(area.Height * 0.85));
        AppWindow.Move(new PointInt32((area.Width - initW) / 2, (area.Height - initH) / 2));
    }

    /// Check if game is running and warn before destructive operations.
    /// Returns true if safe to proceed, false if user should stop.
    public static async Task<bool> CheckGameGuard(XamlRoot? xamlRoot)
    {
        if (!Guard.GameRunning()) return true;
        var d = new ContentDialog
        {
            XamlRoot = xamlRoot,
            Title = "Subnautica 2 is running",
            Content = "Please close Subnautica 2 before making changes to save files to avoid corruption.",
            PrimaryButtonText = "Retry",
            SecondaryButtonText = "Close App",
            DefaultButton = ContentDialogButton.Primary
        };
        while (true)
        {
            var r = await d.ShowAsync();
            if (r == ContentDialogResult.Secondary) { Environment.Exit(0); return false; }
            if (!Guard.GameRunning()) return true;
        }
    }

    public MainWindow()
    {
        InitializeComponent();
        ExtendsContentIntoTitleBar = true;
        SetTitleBar(AppTitleBar);
        AppWindow.TitleBar.PreferredHeightOption = TitleBarHeightOption.Tall;
        var exeDir = System.IO.Path.GetDirectoryName(System.Reflection.Assembly.GetExecutingAssembly().Location);
        AppWindow.SetIcon(System.IO.Path.Combine(AppContext.BaseDirectory, "Assets/AppIcon.ico"));
        var area = DisplayArea.Primary.WorkArea;
        _minW = (int)(area.Width * 0.65);
        _minH = (int)(area.Height * 0.75);
        int initW = Math.Max(_minW, (int)(area.Width * 0.8));
        int initH = Math.Max(_minH, (int)(area.Height * 0.85));
        AppWindow.Resize(new SizeInt32(initW, initH));

        ((FrameworkElement)Content).Loaded += async (_, _) =>
        {
            var ver = System.Reflection.Assembly.GetExecutingAssembly().GetName().Version;
            Title = $"NotAlterra v{ver.Major}.{ver.Minor}.{ver.Build}";
            AppTitleBar.Title = Title;
            var iconPath = System.IO.Path.Combine(AppContext.BaseDirectory, "Assets/AppIcon.ico");
            var iconUri = new System.Uri(iconPath);
            var iconSource = new Microsoft.UI.Xaml.Media.Imaging.BitmapImage(iconUri);
            AppTitleBar.IconSource = new Microsoft.UI.Xaml.Controls.ImageIconSource { ImageSource = iconSource };
            PositionWindow();
            var hwnd = WindowNative.GetWindowHandle(this);
            _wndProcRef = WndProc;
            _oldWndProc = SetWindowLongPtr64(hwnd, GWLP_WNDPROC, Marshal.GetFunctionPointerForDelegate(_wndProcRef));

            bool isPackaged = false;
            try { var _ = Windows.ApplicationModel.Package.Current; isPackaged = true; } catch { }
            var webmUri = isPackaged
                ? new Uri("ms-appx:///Assets/loop.webm")
                : new Uri(System.IO.Path.Combine(AppContext.BaseDirectory, "Assets", "loop.webm"));
            SidebarVideo.SetMediaPlayer(new MediaPlayer
            {
                IsLoopingEnabled = true, AutoPlay = true,
                Source = MediaSource.CreateFromUri(webmUri)
            });

            bool _wide = true;
            MainGrid.SizeChanged += (_, _) =>
            {
                bool now = MainGrid.ActualWidth >= 1000;
                if (now == _wide) return;
                _wide = now;
                MainGrid.ColumnDefinitions[0].Width = new GridLength(now ? 360 : 280);
            };

            SetActive(HomeBtn);
            NavFrame.Navigate(typeof(HomePage));

            // Launch-time game guard
            while (Guard.GameRunning())
            {
                var d = new ContentDialog
                {
                    XamlRoot = Content.XamlRoot,
                    Title = "Subnautica 2 is running",
                    Content = "Please save and close Subnautica 2 before using this tool.",
                    PrimaryButtonText = "Retry",
                    SecondaryButtonText = "Close App",
                    DefaultButton = ContentDialogButton.Primary
                };
                var r = await d.ShowAsync();
                if (r == ContentDialogResult.Secondary) { Environment.Exit(0); return; }
                if (!Guard.GameRunning()) break;
            }

        };
    }

    private IntPtr WndProc(IntPtr hWnd, uint msg, IntPtr wParam, IntPtr lParam)
    {
        if (msg == WM_GETMINMAXINFO)
        {
            var mmi = Marshal.PtrToStructure<MINMAXINFO>(lParam);
            mmi.ptMinTrackSize.X = _minW;
            mmi.ptMinTrackSize.Y = _minH;
            Marshal.StructureToPtr(mmi, lParam, false);
            return IntPtr.Zero;
        }
        return CallWindowProc(_oldWndProc, hWnd, msg, wParam, lParam);
    }

    private void NavBtn_Click(object sender, RoutedEventArgs e)
    {
        if (sender is not Button btn || btn.Tag is not string tag) return;
        SetActive(btn);
        switch (tag) { case "home": NavFrame.Navigate(typeof(HomePage)); break; case "saves": NavFrame.Navigate(typeof(SaveManagerPage)); break; case "about": NavFrame.Navigate(typeof(AboutPage)); break; default: NavFrame.Navigate(typeof(SettingsPage)); break; }
    }

    private void SetActive(Button btn)
    {
        if (_activeBtn != null && _activeBtn.Content is Grid pg && pg.Children.Count > 0 && pg.Children[0] is Rectangle pr)
            pr.Visibility = Visibility.Collapsed;
        if (btn.Content is Grid g && g.Children.Count > 0 && g.Children[0] is Rectangle r)
            r.Visibility = Visibility.Visible;
        _activeBtn = btn;
    }

    private void TitleBar_BackRequested(TitleBar s, object a) => NavFrame.GoBack();
}
