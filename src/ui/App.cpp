// NotAlterra — WinUI 3 Desktop application — debug msgbox

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

int WINAPI WinMain(HINSTANCE, HINSTANCE, LPSTR, int) {
    MessageBoxA(nullptr, "WinMain started", "NotAlterra", MB_OK);
    return 0;
}
