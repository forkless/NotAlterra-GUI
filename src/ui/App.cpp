// NotAlterra — WinUI 3 Desktop application

#define _WIN32_WINNT 0x0A00
#include <windows.h>
#include <shellapi.h>
#include <commctrl.h>
#include <string>

typedef struct PACKAGE_VERSION { USHORT Rev; USHORT Build; USHORT Minor; USHORT Major; } PACKAGE_VERSION;
typedef enum MddBootstrapInitializeOptions { MddBootstrapInitializeOptions_None = 0 } MddBootstrapInitializeOptions;

#include <winrt/Windows.Foundation.h>
#include <winrt/Windows.Foundation.Collections.h>
#include <winrt/Microsoft.UI.Xaml.h>
#include <winrt/Microsoft.UI.Xaml.Controls.h>

using namespace winrt;
using namespace Microsoft::UI::Xaml;
using namespace Microsoft::UI::Xaml::Controls;

struct App : ApplicationT<App> {
    void OnLaunched(LaunchActivatedEventArgs const&) try {
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
    } catch (winrt::hresult_error const& e) {
        std::string m = "OnLaunched: " + winrt::to_string(e.message());
        MessageBoxA(nullptr, m.c_str(), "NotAlterra", MB_OK);
    }
};

typedef HRESULT(WINAPI* FN)(UINT32, PCWSTR, PACKAGE_VERSION, int);

int WINAPI WinMain(HINSTANCE, HINSTANCE, LPSTR, int) {
    wchar_t path[MAX_PATH];
    GetModuleFileNameW(nullptr, path, MAX_PATH);
    std::wstring dll(path);
    auto pos = dll.rfind(L'\\');
    dll = dll.substr(0, pos) + L"\\Microsoft.WindowsAppRuntime.Bootstrap.dll";

    HMODULE h = LoadLibraryW(dll.c_str());
    if (!h) {
        MessageBoxA(nullptr, "Bootstrap DLL not found", "NotAlterra", MB_OK);
        return 1;
    }

    FN init = (FN)GetProcAddress(h, "MddBootstrapInitialize2");
    if (!init) {
        MessageBoxA(nullptr, "MddBootstrapInitialize2 not found", "NotAlterra", MB_OK);
        return 1;
    }

    PACKAGE_VERSION v{}; v.Major = 1; v.Minor = 6;
    HRESULT hr = init(0x00010006, L"stable", v, 0);
    if (FAILED(hr)) {
        char buf[64];
        sprintf_s(buf, "Bootstrap failed: 0x%08X", (unsigned)hr);
        MessageBoxA(nullptr, buf, "NotAlterra", MB_OK);
        return 1;
    }

    Application::Start([](auto const&) { make<App>(); });
    return 0;
}
