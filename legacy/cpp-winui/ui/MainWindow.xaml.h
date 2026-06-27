#pragma once

#include <winrt/Windows.Foundation.Collections.h>
#include "MainWindow.xaml.g.h"

namespace winrt::NotAlterra::implementation
{
    struct MainWindow : MainWindowT<MainWindow>
    {
        MainWindow();
        void OnNavSelectionChanged(
            Microsoft::UI::Xaml::Controls::NavigationView const& sender,
            Microsoft::UI::Xaml::Controls::NavigationViewSelectionChangedEventArgs const& args);
    };
}
