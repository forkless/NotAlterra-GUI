// NotAlterra — WinUI 3 Desktop application (XAML entry point)

#define _WIN32_WINNT 0x0A00
#include <windows.h>
#include <shellapi.h>
#include <string>
#include <memory>

typedef struct PACKAGE_VERSION { USHORT Rev; USHORT Build; USHORT Minor; USHORT Major; } PACKAGE_VERSION;
typedef enum MddBootstrapInitializeOptions { MddBootstrapInitializeOptions_None = 0 } MddBootstrapInitializeOptions;

#include <winrt/Windows.Foundation.h>
#include <winrt/Windows.Foundation.Collections.h>
#include <winrt/Microsoft.UI.Xaml.h>
#include <winrt/Microsoft.UI.Xaml.Controls.h>

#include "App.xaml.h"
#include "MainWindow.xaml.h"

using namespace winrt;
using namespace Microsoft::UI::Xaml;

namespace winrt::NotAlterra::implementation
{
    void App::OnLaunched(LaunchActivatedEventArgs const&)
    {
        auto window = make<winrt::NotAlterra::implementation::MainWindow>();
        window.Activate();
    }
}

typedef HRESULT(WINAPI* MddBootstrapInitialize2Fn)(UINT32, PCWSTR, PACKAGE_VERSION, int);

int WINAPI WinMain(HINSTANCE, HINSTANCE, LPSTR, int)
{
    // ── Locate the Windows App Runtime Bootstrap DLL alongside the EXE ──
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

    auto init = reinterpret_cast<MddBootstrapInitialize2Fn>(
        GetProcAddress(h, "MddBootstrapInitialize2"));
    if (!init) {
        MessageBoxA(nullptr, "MddBootstrapInitialize2 not found", "NotAlterra", MB_OK);
        return 1;
    }

    PACKAGE_VERSION zero{};
    HRESULT hr = init(0x00010008, L"", zero, 0);
    if (FAILED(hr)) hr = init(0x00010008, L"stable", zero, 0);
    if (FAILED(hr)) hr = init(0x00010006, L"", zero, 0);
    if (FAILED(hr)) hr = init(0x00010006, L"stable", zero, 0);
    if (FAILED(hr)) {
        char buf[64];
        sprintf_s(buf, "Bootstrap failed: 0x%08X", static_cast<unsigned>(hr));
        MessageBoxA(nullptr, buf, "NotAlterra", MB_OK);
        return 1;
    }

    // ── Start the WinUI 3 application ──
    try {
        Application::Start([](auto const&) { make<winrt::NotAlterra::implementation::App>(); });
    } catch (winrt::hresult_error const& e) {
        std::string m = "Start: " + winrt::to_string(e.message());
        MessageBoxA(nullptr, m.c_str(), "NotAlterra", MB_OK);
    } catch (std::exception const& e) {
        MessageBoxA(nullptr, e.what(), "NotAlterra", MB_OK);
    } catch (...) {
        MessageBoxA(nullptr, "Unknown error in Start", "NotAlterra", MB_OK);
    }

    return 0;
}
