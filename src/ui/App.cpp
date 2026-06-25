// NotAlterra — WinUI 3 Desktop application (minimal compile test)

#include <windows.h>
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

int WINAPI WinMain(HINSTANCE, HINSTANCE, LPSTR, int) {
    Application::Start([](auto const&) { make<App>(); });
    return 0;
}
