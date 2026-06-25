// NotAlterra — WinUI 3 Desktop application
// Dashboard: stat cards + activity feed.
// UI built in C++/WinRT code — no XAML compiler needed.
//
// Windows headers — C++/WinRT needs the full Windows API surface
#include <windows.h>

// C++/WinRT projections — order matters
// Windows.Foundation is pulled in via the generated WinUI headers
#include <winrt/Microsoft.UI.Xaml.h>
#include <winrt/Microsoft.UI.Xaml.Controls.h>
#include <winrt/Microsoft.UI.Xaml.Controls.Primitives.h>
#include <winrt/Microsoft.UI.Xaml.Media.h>
#include <winrt/Microsoft.UI.Xaml.Navigation.h>
#include <winrt/Microsoft.UI.Dispatching.h>

#include <string>

// Explicit using declarations — no blanket `using namespace winrt;`
// which would conflict with Windows SDK header types like Version/Build.
using winrt::box_value;
using winrt::hstring;
using winrt::Microsoft::UI::Xaml::Thickness;
using winrt::Windows::UI::ColorHelper;

using namespace Microsoft::UI::Xaml;
using namespace Microsoft::UI::Xaml::Controls;
using namespace Microsoft::UI::Xaml::Media;

// ── Helpers ─────────────────────────────────────────────────────────────────

static Thickness margin(double all) {
    return { all, all, all, all };
}
static Thickness margin(double l, double t, double r, double b) {
    return { l, t, r, b };
}

// accent_brush() removed — unused, requires extra SDK includes

/// Build a stat card (Border + StackPanel with icon, number, label).
static Border make_stat_card(hstring const& icon, hstring const& value, hstring const& label) {
    auto border = Border();
    border.Background(SolidColorBrush(Windows::UI::ColorHelper::FromArgb(20, 128, 128, 128)));
    border.CornerRadius({ 8, 8, 8, 8 });
    border.Margin({ 6, 6, 6, 6 });
    border.MinWidth(140);
    border.MinHeight(90);

    auto stack = StackPanel();
    stack.VerticalAlignment(VerticalAlignment::Center);
    stack.HorizontalAlignment(HorizontalAlignment::Center);

    auto iconText = TextBlock();
    iconText.Text(icon);
    iconText.FontSize(24);
    iconText.HorizontalAlignment(HorizontalAlignment::Center);

    auto valueText = TextBlock();
    valueText.Text(value);
    valueText.FontSize(28);
    valueText.FontWeight(Windows::UI::Text::FontWeight{ 600 });
    valueText.HorizontalAlignment(HorizontalAlignment::Center);
    valueText.Margin({ 0, 4, 0, 0 });

    auto labelText = TextBlock();
    labelText.Text(label);
    labelText.FontSize(13);
    labelText.Foreground(SolidColorBrush(Windows::UI::Colors::Gray()));
    labelText.HorizontalAlignment(HorizontalAlignment::Center);

    stack.Children().Append(iconText);
    stack.Children().Append(valueText);
    stack.Children().Append(labelText);
    border.Child(stack);
    return border;
}

static TextBlock make_timeline_entry(hstring const& icon, hstring const& text) {
    auto line = TextBlock();
    line.Text(icon + L"  " + text);
    line.Margin({ 12, 4, 12, 4 });
    line.TextWrapping(TextWrapping::NoWrap);
    line.TextTrimming(TextTrimming::CharacterEllipsis);
    return line;
}

// ── Application ─────────────────────────────────────────────────────────────

struct App : ApplicationT<App> {
    void OnLaunched([[maybe_unused]] LaunchActivatedEventArgs const&) {
        auto window = Window();
        window.Title(L"NotAlterra");

        // ── Navigation View ──────────────────────────────────────────────
        auto nav = NavigationView();
        nav.PaneDisplayMode(NavigationViewPaneDisplayMode::LeftCompact);
        nav.IsSettingsVisible(false);

        // Pane items
        auto dashItem = NavigationViewItem();
        dashItem.Content(box_value(L"Dashboard"));
        dashItem.Icon(SymbolIcon(Symbol::Home));
        nav.MenuItems().Append(dashItem);

        auto savesItem = NavigationViewItem();
        savesItem.Content(box_value(L"Saves"));
        savesItem.Icon(SymbolIcon(Symbol::Library));
        nav.MenuItems().Append(savesItem);

        auto backupItem = NavigationViewItem();
        backupItem.Content(box_value(L"Backups"));
        backupItem.Icon(SymbolIcon(Symbol::Backup));
        nav.MenuItems().Append(backupItem);

        auto configItem = NavigationViewItem();
        configItem.Content(box_value(L"Config"));
        configItem.Icon(SymbolIcon(Symbol::Setting));
        nav.MenuItems().Append(configItem);

        // ── Dashboard content ────────────────────────────────────────────
        auto root = StackPanel();
        root.Margin({ 20, 20, 20, 20 });

        // InfoBar
        auto infoBar = InfoBar();
        infoBar.Title(box_value(L"Save folder not set"));
        infoBar.Message(L"Go to Settings to set your Subnautica 2 save folder path.");
        infoBar.Severity(InfoBarSeverity::Warning);
        infoBar.IsOpen(true);
        root.Children().Append(infoBar);

        // Stat cards grid
        auto cardGrid = Grid();
        cardGrid.ColumnDefinitions().Append(ColumnDefinition());
        cardGrid.ColumnDefinitions().Append(ColumnDefinition());
        cardGrid.ColumnDefinitions().Append(ColumnDefinition());
        cardGrid.ColumnDefinitions().Append(ColumnDefinition());
        cardGrid.Margin({ 0, 12, 0, 0 });

        auto savesCard = make_stat_card(L"💾", L"5", L"Saves");
        savesCard.SetValue(Grid::ColumnProperty(), 0);
        cardGrid.Children().Append(savesCard);

        auto backupCard = make_stat_card(L"📦", L"12", L"Backups");
        backupCard.SetValue(Grid::ColumnProperty(), 1);
        cardGrid.Children().Append(backupCard);

        auto timeCard = make_stat_card(L"⏱", L"2h ago", L"Last Backup");
        timeCard.SetValue(Grid::ColumnProperty(), 2);
        cardGrid.Children().Append(timeCard);

        auto playCard = make_stat_card(L"🎮", L"32.3h", L"Playtime");
        playCard.SetValue(Grid::ColumnProperty(), 3);
        cardGrid.Children().Append(playCard);

        root.Children().Append(cardGrid);

        // Recent Activity section
        auto activityHeader = TextBlock();
        activityHeader.Text(L"Recent Activity");
        activityHeader.FontSize(18);
        activityHeader.FontWeight(Windows::UI::Text::FontWeight{ 500 });
        activityHeader.Margin({ 0, 20, 0, 8 });
        root.Children().Append(activityHeader);

        auto activityBorder = Border();
        activityBorder.Background(SolidColorBrush(Windows::UI::ColorHelper::FromArgb(12, 128, 128, 128)));
        activityBorder.CornerRadius({ 8, 8, 8, 8 });
        activityBorder.MinHeight(120);

        auto activityStack = StackPanel();
        activityStack.VerticalAlignment(VerticalAlignment::Top);
        activityStack.Margin({ 0, 8, 0, 8 });

        activityStack.Children().Append(make_timeline_entry(L"✅", L"Full backup created — Today 14:32"));
        activityStack.Children().Append(make_timeline_entry(L"✅", L"Config backup created — Today 14:30"));
        activityStack.Children().Append(make_timeline_entry(L"ℹ️", L"Save folder set: C:\\Users\\...\\SaveGames — Yesterday 17:45"));

        activityBorder.Child(activityStack);
        root.Children().Append(activityBorder);

        // Set dashboard as content, wrap in ScrollViewer
        auto scroll = ScrollViewer();
        scroll.Content(root);

        nav.Content(scroll);
        window.Content(nav);
        window.Activate();
    }
};

// ── Entry point ─────────────────────────────────────────────────────────────

int WINAPI WinMain(HINSTANCE, HINSTANCE, LPSTR, int) {
    // WinUI 3 requires MddBootstrapInitialize() at runtime for the
    // Windows App SDK. For compilation testing, the stub is sufficient.
    // Full initialization will be added when the WinUI shell ships.

    Application::Start([](auto const&) {
        make<App>();
    });

    return 0;
}
