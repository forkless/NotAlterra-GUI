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

        try
        {
            var ver = Package.Current.Id.Version;
            AppVersion = $"{ver.Major}.{ver.Minor}.{ver.Build}";
        }
        catch
        {
            AppVersion = "0.5.0";
        }

        Arch = RuntimeInformation.ProcessArchitecture switch
        {
            Architecture.X64 => "AMD64",
            Architecture.Arm64 => "ARM64",
            _ => RuntimeInformation.ProcessArchitecture.ToString()
        };
        BuildInfo = $"{DateTime.Now:MMM dd, yyyy} · {Arch}";

        ParsingEngine = "100% hand-rolled heuristic byte-scan\n(GVAS didn't stand a chance)";
        SupportedProperties = 13;
        TestFiles = 8;
        TestCoverage = "102 tests · 85 unit + 17 property-based\n(we test everything. twice.)";
        FuzzStats = "2,700 random inputs per test run\nFsCheck tried to break it. It couldn't. We scared it.";
        Dependencies = "6 NuGet packages\n(no node_modules in sight)";
        BinarySize = "~30 MB MSIX\n(22 MB of that is .NET runtime tax.\nThe actual app is 8 MB of pure spite.)";
    }
}
