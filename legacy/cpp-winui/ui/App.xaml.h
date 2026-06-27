#pragma once

#include <winrt/Windows.Foundation.Collections.h>
#include "App.xaml.g.h"

namespace winrt::NotAlterra::implementation
{
    struct App : AppT<App>
    {
        App() = default;
        void OnLaunched(Microsoft::UI::Xaml::LaunchActivatedEventArgs const& e);
    };
}
