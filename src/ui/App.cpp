// NotAlterra — WinUI 3 Desktop application
// Uses LoadLibrary/GetProcAddress for the bootstrap DLL
// so we can show a friendly dialog if the runtime is missing.

#define _WIN32_WINNT 0x0A00
#include <windows.h>
#include <shellapi.h>
#include <commctrl.h>
#include <string>

// Types normally from MddBootstrap.h
typedef struct PACKAGE_VERSION {
    USHORT Revision;
    USHORT Build;
    USHORT Minor;
    USHORT Major;
} PACKAGE_VERSION;

typedef enum MddBootstrapInitializeOptions {
    MddBootstrapInitializeOptions_None = 0,
} MddBootstrapInitializeOptions;

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

// Function pointer types for the bootstrap API
typedef HRESULT (WINAPI *MddInit2Fn)(UINT32, PCWSTR, PACKAGE_VERSION, MddBootstrapInitializeOptions);
typedef void (WINAPI *MddShutdownFn)();

static MddInit2Fn g_MddInit2 = nullptr;
static MddShutdownFn g_MddShutdown = nullptr;

static bool load_bootstrap() {
    // Look for the bootstrap DLL in the same directory as the .exe
    wchar_t exePath[MAX_PATH];
    GetModuleFileNameW(nullptr, exePath, MAX_PATH);
    std::wstring dllPath(exePath);
    auto pos = dllPath.rfind(L'\\');
    dllPath = dllPath.substr(0, pos) + L"\\Microsoft.WindowsAppRuntime.Bootstrap.dll";

    HMODULE hMod = LoadLibraryW(dllPath.c_str());
    if (!hMod) return false;

    g_MddInit2 = (MddInit2Fn)GetProcAddress(hMod, "MddBootstrapInitialize2");
    g_MddShutdown = (MddShutdownFn)GetProcAddress(hMod, "MddBootstrapShutdown");
    return (g_MddInit2 != nullptr && g_MddShutdown != nullptr);
}

static void show_download_prompt() {
    TASKDIALOGCONFIG tdc = {};
    tdc.cbSize = sizeof(tdc);
    tdc.dwFlags = TDF_USE_HICON_MAIN | TDF_ALLOW_DIALOG_CANCELLATION;
    tdc.hMainIcon = LoadIcon(nullptr, IDI_INFORMATION);
    tdc.pszWindowTitle = L"NotAlterra";
    tdc.pszMainInstruction = L"Windows App SDK runtime required";
    tdc.pszContent = L"NotAlterra needs a free Microsoft component to run.\n\n"
                     L"Click Download to get it, then install and run NotAlterra again.";
    const int DL = 100;
    TASKDIALOG_BUTTON btns[] = { { DL, L"Download runtime" }, { IDCANCEL, L"Exit" } };
    tdc.cButtons = 2; tdc.pButtons = btns; tdc.nDefaultButton = DL;
    int r = 0;
    TaskDialogIndirect(&tdc, &r, nullptr, nullptr);
    if (r == DL)
        ShellExecuteW(nullptr, L"open",
            L"https://aka.ms/windowsappsdk/1.6/WindowsAppRuntimeInstall-x64.exe",
            nullptr, nullptr, SW_SHOWNORMAL);
}

int WINAPI WinMain(HINSTANCE, HINSTANCE, LPSTR, int) {
    if (!load_bootstrap()) {
        show_download_prompt();
        return 1;
    }

    PACKAGE_VERSION ver{};
    ver.Major = 1;
    ver.Minor = 6;

    HRESULT hr = g_MddInit2(0x00010006, L"stable", ver,
                            MddBootstrapInitializeOptions_None);
    if (FAILED(hr)) {
        show_download_prompt();
        return 1;
    }

    Application::Start([](auto const&) { make<App>(); });
    g_MddShutdown();
    return 0;
}
