// NotAlterra — Win32 Desktop application
// Zero dependencies, single .exe, Windows 11 native.

#define WIN32_LEAN_AND_MEAN

#include <windows.h>
#include <commctrl.h>
#include <string>

#pragma comment(linker,"\"/manifestdependency:type='win32' name='Microsoft.Windows.Common-Controls' version='6.0.0.0' processorArchitecture='*' publicKeyToken='6595b64144ccf1df' language='*'\"")

static constexpr int IDC_NAV_DASHBOARD = 101;
static constexpr int IDC_NAV_SAVES    = 102;
static constexpr int IDC_NAV_BACKUPS  = 103;
static constexpr int IDC_NAV_CONFIG   = 104;
static constexpr int IDC_LIST_VIEW    = 201;
static constexpr int IDC_INFO_TEXT    = 301;

static HWND g_mainStatus = nullptr;
static HWND g_contentArea = nullptr;
static HWND g_listView = nullptr;
static HWND g_infoText = nullptr;

static std::wstring g_status = L"Ready";

static void update_status(HWND hwnd, std::wstring const& text) {
    g_status = text;
    if (g_mainStatus) SetWindowTextW(g_mainStatus, g_status.c_str());
}

static void navigate_to(HWND hwnd, int page) {
    std::wstring title, info;
    switch (page) {
        case IDC_NAV_DASHBOARD:
            title = L"NotAlterra - Dashboard";
            info = L"Dashboard\n\nSet your save folder in Settings to get started.\n\n"
                   L"Saves: 5    Backups: 12    Playtime: 32.3h";
            break;
        case IDC_NAV_SAVES:
            title = L"NotAlterra - Saves";
            info = L"Save Files\n\nSelect a save file to inspect its metadata.";
            break;
        case IDC_NAV_BACKUPS:
            title = L"NotAlterra - Backups";
            info = L"Backups\n\nCreate a full backup or restore from a previous one.";
            break;
        case IDC_NAV_CONFIG:
            title = L"NotAlterra - Config";
            info = L"Config (.ini) Files\n\nManage UE5 Config files.";
            break;
    }
    SetWindowTextW(hwnd, title.c_str());
    SetWindowTextW(g_infoText, info.c_str());
}

static LRESULT CALLBACK WndProc(HWND hwnd, UINT msg, WPARAM wParam, LPARAM lParam) {
    switch (msg) {
        case WM_CREATE: {
            // Navigation buttons
            CreateWindowW(L"BUTTON", L"  Dashboard  ", WS_CHILD | WS_VISIBLE | BS_PUSHLIKE,
                          10, 10, 100, 30, hwnd, (HMENU)IDC_NAV_DASHBOARD, nullptr, nullptr);
            CreateWindowW(L"BUTTON", L"  Saves  ", WS_CHILD | WS_VISIBLE | BS_PUSHLIKE,
                          120, 10, 80, 30, hwnd, (HMENU)IDC_NAV_SAVES, nullptr, nullptr);
            CreateWindowW(L"BUTTON", L"  Backups  ", WS_CHILD | WS_VISIBLE | BS_PUSHLIKE,
                          210, 10, 90, 30, hwnd, (HMENU)IDC_NAV_BACKUPS, nullptr, nullptr);
            CreateWindowW(L"BUTTON", L"  Config  ", WS_CHILD | WS_VISIBLE | BS_PUSHLIKE,
                          310, 10, 80, 30, hwnd, (HMENU)IDC_NAV_CONFIG, nullptr, nullptr);

            // Content area
            g_infoText = CreateWindowW(L"STATIC", L"", WS_CHILD | WS_VISIBLE,
                                       10, 50, 560, 200, hwnd, (HMENU)IDC_INFO_TEXT, nullptr, nullptr);

            // Status bar
            g_mainStatus = CreateWindowW(STATUSCLASSNAMEW, L"Ready",
                                          WS_CHILD | WS_VISIBLE | SBARS_SIZEGRIP,
                                          0, 0, 0, 0, hwnd, nullptr, nullptr, nullptr);

            navigate_to(hwnd, IDC_NAV_DASHBOARD);
            update_status(hwnd, L"Save folder not set");
            break;
        }

        case WM_COMMAND: {
            int id = LOWORD(wParam);
            if (id >= IDC_NAV_DASHBOARD && id <= IDC_NAV_CONFIG) {
                navigate_to(hwnd, id);
            }
            break;
        }

        case WM_SIZE: {
            RECT rc;
            GetClientRect(hwnd, &rc);
            if (g_mainStatus)
                SendMessageW(g_mainStatus, WM_SIZE, 0, 0);
            if (g_infoText)
                SetWindowPos(g_infoText, nullptr, 10, 50,
                             rc.right - 20, rc.bottom - 100, SWP_NOZORDER);
            break;
        }

        case WM_DESTROY:
            PostQuitMessage(0);
            break;

        default:
            return DefWindowProcW(hwnd, msg, wParam, lParam);
    }
    return 0;
}

int WINAPI WinMain(HINSTANCE hInstance, HINSTANCE, LPSTR, int nCmdShow) {
    // Init common controls for status bar
    INITCOMMONCONTROLSEX icc = { sizeof(icc), ICC_BAR_CLASSES };
    InitCommonControlsEx(&icc);

    // Register window class
    WNDCLASSEXW wc = { sizeof(wc) };
    wc.lpfnWndProc = WndProc;
    wc.hInstance = hInstance;
    wc.hCursor = LoadCursor(nullptr, IDC_ARROW);
    wc.hbrBackground = (HBRUSH)(COLOR_WINDOW + 1);
    wc.lpszClassName = L"NotAlterra";

    if (!RegisterClassExW(&wc)) {
        MessageBoxW(nullptr, L"Failed to register window class.", L"NotAlterra", MB_ICONERROR);
        return 1;
    }

    // Create window
    HWND hwnd = CreateWindowExW(0, L"NotAlterra", L"NotAlterra - Dashboard",
                                 WS_OVERLAPPEDWINDOW, CW_USEDEFAULT, CW_USEDEFAULT,
                                 600, 400, nullptr, nullptr, hInstance, nullptr);
    if (!hwnd) {
        MessageBoxW(nullptr, L"Failed to create window.", L"NotAlterra", MB_ICONERROR);
        return 1;
    }

    ShowWindow(hwnd, nCmdShow);
    UpdateWindow(hwnd);

    // Message loop
    MSG msg;
    while (GetMessageW(&msg, nullptr, 0, 0)) {
        TranslateMessage(&msg);
        DispatchMessageW(&msg);
    }

    return (int)msg.wParam;
}
