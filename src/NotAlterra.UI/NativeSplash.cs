/// Native Win32 layered transparent splash window.
/// No WinUI dependencies — shows instantly, per-pixel alpha.
using System.Runtime.InteropServices;
using Windows.Graphics.Imaging;
using Windows.Storage;

namespace NotAlterra_UI;

public sealed class NativeSplash : IDisposable
{
    // ── Win32 ────────────────────────────────────────────────────────

    [DllImport("user32.dll")]
    private static extern IntPtr CreateWindowEx(int dwExStyle, string lpClassName, string lpWindowName,
        int dwStyle, int x, int y, int nWidth, int nHeight, IntPtr hWndParent, IntPtr hMenu,
        IntPtr hInstance, IntPtr lpParam);

    [DllImport("user32.dll")] private static extern bool DestroyWindow(IntPtr hWnd);
    [DllImport("user32.dll")] private static extern bool ShowWindow(IntPtr hWnd, int nCmdShow);
    [DllImport("user32.dll")] private static extern bool SetWindowPos(IntPtr hWnd, IntPtr hWndInsertAfter, int X, int Y, int cx, int cy, uint uFlags);
    private const uint SWP_NOSIZE = 0x0001;
    private const uint SWP_NOZORDER = 0x0004;
    private const uint SWP_SHOWWINDOW = 0x0040;
    [DllImport("gdi32.dll")] private static extern IntPtr CreateDIBSection(IntPtr hdc, ref BITMAPINFOHEADER pbmi, uint iUsage, out IntPtr ppvBits, IntPtr hSection, uint dwOffset);
    [DllImport("gdi32.dll")] private static extern IntPtr CreateCompatibleDC(IntPtr hDC);
    [DllImport("gdi32.dll")] private static extern bool DeleteDC(IntPtr hDC);
    [DllImport("gdi32.dll")] private static extern IntPtr SelectObject(IntPtr hDC, IntPtr hObject);
    [DllImport("gdi32.dll")] private static extern bool DeleteObject(IntPtr hObject);
    [DllImport("user32.dll")] private static extern bool UpdateLayeredWindow(IntPtr hWnd, IntPtr hdcDst, ref POINT pptDst, ref SIZE psize, IntPtr hdcSrc, ref POINT pptSrc, int crKey, ref BLENDFUNCTION pblend, int dwFlags);
    [DllImport("kernel32.dll")] private static extern IntPtr GetModuleHandle(string? lpModuleName);
    [DllImport("user32.dll")] private static extern bool GetWindowRect(IntPtr hWnd, out RECT lpRect);
    [DllImport("user32.dll")] private static extern IntPtr MonitorFromWindow(IntPtr hwnd, uint dwFlags);
    [DllImport("user32.dll")] private static extern bool GetMonitorInfoW(IntPtr hMonitor, ref MONITORINFO lpmi);
    private const uint MONITOR_DEFAULTTONEAREST = 2;
    [StructLayout(LayoutKind.Sequential)] private struct RECT { public int left, top, right, bottom; }
    [StructLayout(LayoutKind.Sequential, CharSet = CharSet.Auto)] private struct MONITORINFO
    {
        public int cbSize;
        public RECT rcMonitor, rcWork;
        public uint dwFlags;
    }

    [StructLayout(LayoutKind.Sequential)]
    private struct POINT { public int x, y; }
    [StructLayout(LayoutKind.Sequential)]
    private struct SIZE { public int cx, cy; }
    [StructLayout(LayoutKind.Sequential)]
    private struct BLENDFUNCTION { public byte BlendOp, BlendFlags, SourceConstantAlpha, AlphaFormat; }

    [StructLayout(LayoutKind.Sequential, Pack = 1)]
    private struct BITMAPINFOHEADER
    {
        public uint biSize;
        public int biWidth, biHeight;
        public ushort biPlanes, biBitCount;
        public uint biCompression, biSizeImage;
        public int biXPelsPerMeter, biYPelsPerMeter;
        public uint biClrUsed, biClrImportant;
    }

    private const int WS_EX_LAYERED = 0x80000;
    private const int WS_EX_TOOLWINDOW = 0x80;
    private const int WS_POPUP = unchecked((int)0x80000000);
    private const int AC_SRC_OVER = 0x00;
    private const int AC_SRC_ALPHA = 0x01;
    private const int ULW_ALPHA = 0x02;
    private const int DIB_RGB_COLORS = 0;

    private IntPtr _hwnd;

    /// Fires when the splash window is hidden/closed.
    public event Action? Closed;

    /// Show the splash PNG as a native layered window.
    /// Closes automatically after `durationMs`.
    public void Show(string pngPath, int durationMs = 1500)
    {
        if (!System.IO.File.Exists(pngPath)) return;

        var (pixels, w, h) = DecodePng(pngPath);
        if (pixels == null || w <= 0) return;

        int x = 0, y = 0;

        _hwnd = CreateWindowEx(WS_EX_LAYERED | WS_EX_TOOLWINDOW, "static", "",
            WS_POPUP, x, y, w, h, IntPtr.Zero, IntPtr.Zero,
            GetModuleHandle(null), IntPtr.Zero);
        if (_hwnd == IntPtr.Zero) return;

        var hdcScreen = CreateCompatibleDC(IntPtr.Zero);
        var hdcMem = CreateCompatibleDC(hdcScreen);

        // Build DIB section with bottom-up BGRA data
        var bmi = new BITMAPINFOHEADER
        {
            biSize = (uint)Marshal.SizeOf<BITMAPINFOHEADER>(),
            biWidth = w,
            biHeight = -h,  // negative = top-down (no flip needed)
            biPlanes = 1,
            biBitCount = 32,
            biCompression = 0
        };
        var hDib = CreateDIBSection(hdcScreen, ref bmi, DIB_RGB_COLORS, out var bits, IntPtr.Zero, 0);
        if (hDib == IntPtr.Zero) { Cleanup(hdcScreen, hdcMem, IntPtr.Zero); return; }

        // Copy BGRA pixels
        Marshal.Copy(pixels, 0, bits, pixels.Length);

        var old = SelectObject(hdcMem, hDib);
        var ptDst = new POINT { x = 0, y = 0 };
        var ptSrc = new POINT { x = 0, y = 0 };
        var sz = new SIZE { cx = w, cy = h };
        var blend = new BLENDFUNCTION { BlendOp = AC_SRC_OVER, SourceConstantAlpha = 255, AlphaFormat = AC_SRC_ALPHA };

        UpdateLayeredWindow(_hwnd, hdcScreen, ref ptDst, ref sz, hdcMem, ref ptSrc, 0, ref blend, ULW_ALPHA);

        SelectObject(hdcMem, old);
        DeleteObject(hDib);
        DeleteDC(hdcMem);
        DeleteDC(hdcScreen);

        ShowWindow(_hwnd, 5); // SW_SHOW

        // Recenter after show (screen metrics may have been stale during CreateWindowEx)
        // Center on monitor
        var mi = new MONITORINFO { cbSize = System.Runtime.InteropServices.Marshal.SizeOf<MONITORINFO>() };
        var hMon = MonitorFromWindow(_hwnd, MONITOR_DEFAULTTONEAREST);
        if (GetMonitorInfoW(hMon, ref mi))
        {
            int mw = mi.rcWork.right - mi.rcWork.left;
            int mh = mi.rcWork.bottom - mi.rcWork.top;
            SetWindowPos(_hwnd, IntPtr.Zero,
                mi.rcWork.left + (mw - w) / 2,
                mi.rcWork.top + (mh - h) / 2,
                0, 0, SWP_NOSIZE | SWP_NOZORDER);
        }

        if (durationMs > 0)
            _ = Task.Run(async () =>
            {
                await Task.Delay(durationMs);
                Close();
            });
    }

    public void Close()
    {
        if (_hwnd != IntPtr.Zero) { ShowWindow(_hwnd, 0); DestroyWindow(_hwnd); _hwnd = IntPtr.Zero; Closed?.Invoke(); }
    }

    public void Dispose() => Close();

    private static void Cleanup(IntPtr hdc1, IntPtr hdc2, IntPtr obj)
    {
        if (obj != IntPtr.Zero) DeleteObject(obj);
        if (hdc2 != IntPtr.Zero) DeleteDC(hdc2);
        if (hdc1 != IntPtr.Zero) DeleteDC(hdc1);
    }

    private static (byte[]? pixels, int w, int h) DecodePng(string path)
    {
        try
        {
            var file = StorageFile.GetFileFromPathAsync(path).GetAwaiter().GetResult();
            using var stream = file.OpenReadAsync().GetAwaiter().GetResult();
            var decoder = BitmapDecoder.CreateAsync(stream).GetAwaiter().GetResult();
            var pd = decoder.GetPixelDataAsync(BitmapPixelFormat.Bgra8, BitmapAlphaMode.Premultiplied,
                new BitmapTransform(), ExifOrientationMode.IgnoreExifOrientation,
                ColorManagementMode.DoNotColorManage).GetAwaiter().GetResult();
            return (pd.DetachPixelData(), (int)decoder.PixelWidth, (int)decoder.PixelHeight);
        }
        catch { return (null, 0, 0); }
    }
}
