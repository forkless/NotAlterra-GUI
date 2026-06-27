// NotAlterra — MainWindow implementation

#include <windows.h>

#include <winrt/Windows.Foundation.h>
#include <winrt/Windows.Foundation.Collections.h>
#include <winrt/Microsoft.UI.Xaml.h>
#include <winrt/Microsoft.UI.Xaml.Controls.h>
#include <winrt/Microsoft.UI.Xaml.Media.h>

#include "MainWindow.xaml.h"
#include "MainWindow.xaml.g.hpp"   // generated InitializeComponent

using namespace winrt;
using namespace Microsoft::UI::Xaml;
using namespace Microsoft::UI::Xaml::Controls;

namespace winrt::NotAlterra::implementation
{
    MainWindow::MainWindow()
    {
        // Run the XAML compiler-generated InitializeComponent
        InitializeComponent();

        // ── Apply Mica backdrop ──
        using namespace Microsoft::UI::Xaml::Media;

        auto mica = MicaBackdrop();
        SystemBackdrop(mica);

        // ── Select the first nav item by default ──
        MainNav().SelectedItem(MainNav().MenuItems().GetAt(0));
    }

    void MainWindow::OnNavSelectionChanged(
        NavigationView const& /*sender*/,
        NavigationViewSelectionChangedEventArgs const& args)
    {
        auto selected = args.SelectedItemContainer();
        if (!selected) return;

        auto tag = unbox_value<hstring>(selected.Tag());

        // Hide all pages first
        DashboardPage().Visibility(Visibility::Collapsed);
        SavesPage().Visibility(Visibility::Collapsed);
        BackupsPage().Visibility(Visibility::Collapsed);
        ConfigPage().Visibility(Visibility::Collapsed);

        // Show the selected page
        if (tag == L"dashboard")
            DashboardPage().Visibility(Visibility::Visible);
        else if (tag == L"saves")
            SavesPage().Visibility(Visibility::Visible);
        else if (tag == L"backups")
            BackupsPage().Visibility(Visibility::Visible);
        else if (tag == L"config")
            ConfigPage().Visibility(Visibility::Visible);
    }
}
