// NotAlterra — WinUI 3 Desktop application
// No direct WindowsApp.lib import — C++/WinRT handles lazy resolution.

#define _WIN32_WINNT 0x0A00
#include <windows.h>
#include <shellapi.h>
#include <commctrl.h>
#include <string>

typedef struct PACKAGE_VERSION { USHORT Revision; USHORT Build; USHORT Minor; USHORT Major; } PACKAGE_VERSION;
typedef enum MddBootstrapInitializeOptions { MddBootstrapInitializeOptions_None = 0 } MddBootstrapInitializeOptions;

#include <winrt/Windows.Foundation.h>
#include <winrt/Windows.Foundation.Collections.h>
#include <winrt/Microsoft.UI.Xaml.h>
#include <winrt/Microsoft.UI.Xaml.Controls.h>

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

typedef HRESULT (WINAPI *MddInit2Fn)(UINT32, PCWSTR, PACKAGE_VERSION, int);
typedef void (WINAPI *MddShutdownFn)();

static bool load_bootstrap() {
    wchar_t path[MAX_PATH];
    GetModuleFileNameW(nullptr, path, MAX_PATH);
    std::wstring dll(path);
    auto pos = dll.rfind(L'\\');
    dll = dll.substr(0, pos) + L"\\Microsoft.WindowsAppRuntime.Bootstrap.dll";
    HMODULE h = LoadLibraryW(dll.c_str());
    if (!h) return false;
    auto init = (MddInit2Fn)GetProcAddress(h, "MddBootstrapInitialize2");
    auto shutdown = (MddShutdownFn)GetProcAddress(h, "MddBootstrapShutdown");
    if (!init || !shutdown) return false;
    PACKAGE_VERSION v{}; v.Major = 1; v.Minor = 6;
    if (FAILED(init(0x00010006, L"stable", v, 0))) return false;
    return true;
}

static void log_error(const char* msg) {
    wchar_t path[MAX_PATH];
    GetModuleFileNameW(nullptr, path, MAX_PATH);
    std::wstring log(path);
    auto pos = log.rfind(L'\\');
    log = log.substr(0, pos) + L"\\notalterra.log";
    HANDLE f = CreateFileW(log.c_str(), GENERIC_WRITE, FILE_SHARE_READ, nullptr,
                           OPEN_ALWAYS, FILE_ATTRIBUTE_NORMAL, nullptr);
    if (f != INVALID_HANDLE_VALUE) {
        SetFilePointer(f, 0, nullptr, FILE_END);
        DWORD written;
        WriteFile(f, msg, (DWORD)strlen(msg), &written, nullptr);
        WriteFile(f, "\r\n", 2, &written, nullptr);
        CloseHandle(f);
    }
}

int WINAPI WinMain(HINSTANCE, HINSTANCE, LPSTR, int) {
    log_error("NotAlterra starting");
    if (!load_bootstrap()) {
        log_error("Bootstrap DLL failed to load");
        MessageBoxW(nullptr,
            L"NotAlterra requires the Windows App SDK runtime.\n\n"
            L"Download and install it from:\n"
            L"https://aka.ms/windowsappsdk/1.6/WindowsAppRuntimeInstall-x64.exe",
            L"Runtime Required", MB_OK | MB_ICONINFORMATION);
        return 1;
    }

    Application::Start([](auto const&) { make<App>(); });
    return 0;
}
