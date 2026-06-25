// NotAlterra — WinUI 3 Desktop application
// Self-contained .exe with runtime dependency check.

#include <windows.h>
#include <winrt/Windows.Foundation.h>
#include <winrt/Windows.Foundation.Collections.h>
#include <winrt/Microsoft.UI.Xaml.h>
#include <winrt/Microsoft.UI.Xaml.Controls.h>
#include <MddBootstrap.h>

using namespace winrt;
using namespace Microsoft::UI::Xaml;
using namespace Microsoft::UI::Xaml::Controls;

struct App : ApplicationT<App> {
    void OnLaunched(LaunchActivatedEventArgs const&) {
        auto window = Window();
        window.Title(L"NotAlterra");

        auto nav = NavigationView();
        nav.PaneDisplayMode(NavigationViewPaneDisplayMode::LeftCompact);

        auto item = NavigationViewItem();
        item.Content(box_value(L"Dashboard"));
        item.Icon(SymbolIcon(Symbol::Home));
        nav.MenuItems().Append(item);

        auto text = TextBlock();
        text.Text(L"Subnautica 2 Save Manager");
        text.FontSize(24);

        auto stack = StackPanel();
        stack.Children().Append(text);

        nav.Content(stack);
        window.Content(nav);
        window.Activate();
    }
};

int WINAPI WinMain(HINSTANCE, HINSTANCE, LPSTR, int) {
    PACKAGE_VERSION ver{};
    ver.Major = 1;
    ver.Minor = 6;
    HRESULT hr = ::MddBootstrapInitialize2(0x00010006, L"stable", ver,
        MddBootstrapInitializeOptions_None);

    if (FAILED(hr)) {
        MessageBoxW(nullptr,
            L"NotAlterra requires the Windows App SDK runtime.\n\n"
            L"Download it from:\n"
            L"https://aka.ms/windowsappsdk/1.6/windowsappruntimeinstall-x64.exe\n\n"
            L"Install it, then run NotAlterra again.",
            L"Runtime Required", MB_OK | MB_ICONINFORMATION);
        return 1;
    }

    Application::Start([](auto const&) { make<App>(); });
    MddBootstrapShutdown();
    return 0;
}
