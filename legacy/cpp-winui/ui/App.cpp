// NotAlterra — WinUI 3 Desktop application
// Custom compact nav (no NavigationView — requires XAML resources)

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
#include <winrt/Microsoft.UI.Xaml.Media.h>
#include <winrt/Windows.UI.h>

using namespace winrt;
using namespace Microsoft::UI::Xaml;
using namespace Microsoft::UI::Xaml::Controls;
using namespace Microsoft::UI::Xaml::Media;

static Windows::UI::Color C(uint8_t r, uint8_t g, uint8_t b) { return Windows::UI::Color{ 255, r, g, b }; }
static const auto C_BG       = C(0x0D, 0x11, 0x17);
static const auto C_SURFACE  = C(0x16, 0x1B, 0x22);
static const auto C_TEAL     = C(0x00, 0xBF, 0xA5);
static const auto C_DANGER   = C(0xFF, 0x6B, 0x6B);
static const auto C_TEXT_PRI = C(0xE6, 0xED, 0xF3);
static const auto C_TEXT_SUB = C(0x8B, 0x94, 0x9E);

static SolidColorBrush SB(Windows::UI::Color c) { return SolidColorBrush(c); }

// Stat card helper
static Border MakeCard(std::wstring const& val, std::wstring const& label) {
    auto card = Border();
    card.Background(SB(C_SURFACE));
    card.CornerRadius(CornerRadius{ 8 });
    card.Margin(Thickness{ 4 });
    card.MinWidth(120);
    auto stack = StackPanel();
    stack.Margin(Thickness{ 12, 16 });
    auto v = TextBlock(); v.Text(val); v.FontSize(36); v.FontWeight(Windows::UI::Text::FontWeight{ 700 }); v.Foreground(SB(C_TEAL)); v.Margin(Thickness{ 0, 0, 0, 4 });
    auto l = TextBlock(); l.Text(label); l.FontSize(13); l.Foreground(SB(C_TEXT_SUB));
    stack.Children().Append(v); stack.Children().Append(l);
    card.Child(stack);
    return card;
}

struct App : ApplicationT<App> {
    void OnLaunched(LaunchActivatedEventArgs const&) try {
        auto window = Window();
        window.Title(L"NotAlterra");

        // Root: nav rail (48px) + content area
        auto root = Grid();
        root.Background(SB(C_BG));
        // Column: nav rail | content
        {
            auto cd = ColumnDefinition(); cd.Width(GridLength{ 48 });
            root.ColumnDefinitions().Append(cd);
        }
        {
            auto cd = ColumnDefinition(); cd.Width(GridLength{ 1, GridUnitType::Star });
            root.ColumnDefinitions().Append(cd);
        }

        // Nav rail
        auto rail = Border();
        rail.Background(SB(C_SURFACE));
        rail.Width(48);
        auto railStack = StackPanel();
        railStack.Margin(Thickness{ 0, 8, 0, 0 });

        // Dashboard icon
        auto icon = TextBlock();
        icon.Text(L"\uE80F"); // Home glyph
        icon.FontSize(20);
        icon.HorizontalAlignment(HorizontalAlignment::Center);
        icon.Foreground(SB(C_TEAL));
        icon.Margin(Thickness{ 0, 8 });
        railStack.Children().Append(icon);

        auto label = TextBlock();
        label.Text(L"NotAlterra");
        label.FontSize(10);
        label.HorizontalAlignment(HorizontalAlignment::Center);
        label.Foreground(SB(C_TEXT_SUB));
        label.Margin(Thickness{ 0, 0, 0, 16 });
        railStack.Children().Append(label);

        rail.Child(railStack);
        root.Children().Append(rail);
        Grid::SetColumn(rail, 0);

        // Content area
        auto content = Grid();
        content.Background(SB(C_BG));
        content.Margin(Thickness{ 24, 24, 24, 0 });

        auto inner = StackPanel();
        auto header = TextBlock();
        header.Text(L"Status Report");
        header.FontSize(28);
        header.FontWeight(Windows::UI::Text::FontWeight{ 600 });
        header.Foreground(SB(C_TEXT_PRI));
        header.Margin(Thickness{ 0, 0, 0, 24 });
        inner.Children().Append(header);

        // Stat cards
        auto cards = StackPanel();
        cards.Orientation(Orientation::Horizontal);
        cards.Children().Append(MakeCard(L"3", L"Saves"));
        cards.Children().Append(MakeCard(L"12", L"Backups"));
        cards.Children().Append(MakeCard(L"84h", L"Playtime"));
        cards.Children().Append(MakeCard(L"12m", L"Last Run"));
        inner.Children().Append(cards);

        // Saves section
        auto sec = TextBlock();
        sec.Text(L"Recent Saves");
        sec.FontSize(18);
        sec.FontWeight(Windows::UI::Text::FontWeight{ 500 });
        sec.Foreground(SB(C_TEXT_PRI));
        sec.Margin(Thickness{ 0, 32, 0, 12 });
        inner.Children().Append(sec);

        // Table header
        auto tblHdr = Grid();
        auto addCol = [&](double w) { auto c = ColumnDefinition(); c.Width(GridLength{ w }); tblHdr.ColumnDefinitions().Append(c); };
        addCol(80); addCol(120); addCol(100); addCol(80);
        auto hc = [&](std::wstring const& t, int col) { auto c = TextBlock(); c.Text(t); c.FontSize(12); c.FontWeight(Windows::UI::Text::FontWeight{ 600 }); c.Foreground(SB(C_TEXT_SUB)); c.Margin(Thickness{ 8, 4 }); tblHdr.Children().Append(c); Grid::SetColumn(c, col); };
        hc(L"Slot", 0); hc(L"Playtime", 1); hc(L"Mode", 2); hc(L"Status", 3);
        inner.Children().Append(tblHdr);

        // Table rows
        struct R { std::wstring s, p, m, st; Windows::UI::Color sc; };
        R rows[] = { {L"Slot 0", L"34h 12m", L"Single Player", L"\u2713", C_TEAL}, {L"Slot 1", L"12h 08m", L"Multiplayer", L"\u2713", C_TEAL}, {L"Slot 2", L"--", L"--", L"empty", C_TEXT_SUB} };
        for (auto& rd : rows) {
            auto row = Grid();
            for (int i = 0; i < 4; i++) { auto c = ColumnDefinition(); c.Width(GridLength{ i==0?80.0 : i==1?120.0 : i==2?100.0 : 80.0 }); row.ColumnDefinitions().Append(c); }
            for (int c = 0; c < 4; c++) {
                auto cell = TextBlock(); cell.FontSize(14); cell.Foreground(SB(C_TEXT_PRI)); cell.Margin(Thickness{ 8, 6 });
                switch(c) { case 0: cell.Text(rd.s); break; case 1: cell.Text(rd.p); break; case 2: cell.Text(rd.m); break; case 3: cell.Text(rd.st); cell.Foreground(SB(rd.sc)); break; }
                row.Children().Append(cell); Grid::SetColumn(cell, c);
            }
            inner.Children().Append(row);
        }

        // Footer note
        auto note = TextBlock();
        note.Text(L"Nothing but void down here. Make a save.");
        note.FontSize(12); note.Foreground(SB(C_TEXT_SUB));
        note.Margin(Thickness{ 0, 24, 0, 0 });
        note.FontStyle(Windows::UI::Text::FontStyle::Italic);
        inner.Children().Append(note);

        auto scroll = ScrollViewer();
        scroll.Content(inner);
        content.Children().Append(scroll);
        root.Children().Append(content);
        Grid::SetColumn(content, 1);

        window.Content(root);
        window.Activate();
    } catch (winrt::hresult_error const& e) {
        std::string m = "Error: " + winrt::to_string(e.message());
        MessageBoxA(nullptr, m.c_str(), "NotAlterra", MB_OK);
    } catch (...) {
        MessageBoxA(nullptr, "Unknown error", "NotAlterra", MB_OK);
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
    if (!h) { MessageBoxA(nullptr, "Bootstrap DLL not found", "NotAlterra", MB_OK); return 1; }
    FN init = (FN)GetProcAddress(h, "MddBootstrapInitialize2");
    if (!init) { MessageBoxA(nullptr, "MddBootstrapInitialize2 not found", "NotAlterra", MB_OK); return 1; }

    PACKAGE_VERSION zero{};
    HRESULT hr = init(0x00010008, L"", zero, 0);
    if (FAILED(hr)) hr = init(0x00010008, L"stable", zero, 0);
    if (FAILED(hr)) hr = init(0x00010006, L"", zero, 0);
    if (FAILED(hr)) hr = init(0x00010006, L"stable", zero, 0);
    if (FAILED(hr)) { char buf[64]; sprintf_s(buf, "Bootstrap failed: 0x%08X", (unsigned)hr); MessageBoxA(nullptr, buf, "NotAlterra", MB_OK); return 1; }

    try { Application::Start([](auto const&) { make<App>(); }); }
    catch (winrt::hresult_error const& e) { std::string m = "Start: " + winrt::to_string(e.message()); MessageBoxA(nullptr, m.c_str(), "NotAlterra", MB_OK); }
    return 0;
}
