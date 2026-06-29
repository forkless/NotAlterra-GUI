using Microsoft.UI.Xaml.Controls;
using System.Runtime.InteropServices;
using Windows.ApplicationModel;

namespace NotAlterra_UI.Pages;

public sealed partial class AboutPage : Page
{
    public string AppVersion { get; }
    public string BuildInfo { get; }
    public string ParsingEngine { get; }
    public int SupportedProperties { get; }
    public int TestFiles { get; }
    public string TestCoverage { get; }
    public string FuzzStats { get; }
    public string Dependencies { get; }
    public string BinarySize { get; }
    public string Arch { get; }

    public AboutPage()
    {
        InitializeComponent();

        var ver = System.Reflection.Assembly.GetExecutingAssembly().GetName().Version;
        AppVersion = ver != null ? $"{ver.Major}.{ver.Minor}.{ver.Build}" : "0.5.0";

        Arch = RuntimeInformation.ProcessArchitecture switch
        {
            Architecture.X64 => "x64",
            Architecture.Arm64 => "ARM64",
            _ => RuntimeInformation.ProcessArchitecture.ToString()
        };
        BuildInfo = $"{DateTime.Now:MMM dd, yyyy} · {Arch}";

        ParsingEngine = "100% hand-rolled heuristic byte-scan\n(GVAS didn't stand a chance)";
        SupportedProperties = 13;
        TestFiles = 8;
        TestCoverage = "102 tests · 85 unit + 17 property-based\n(we test everything. twice.)";
        FuzzStats = "2,700 random inputs per test run\nFsCheck tried to break it. It couldn't. We scared it.";
        Dependencies = "4 NuGet packages\n(no node_modules in sight. we have standards.)";
        BinarySize = "~19 MB installer\n(The app itself is ~8 MB. The rest is Microsoft runtime tax.)";
    }
}
