cmake_minimum_required(VERSION 3.20)
project(NotAlterra VERSION 0.5.0 LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_EXTENSIONS OFF)

# ── Dependencies via FetchContent ──────────────────────────────────────────
include(FetchContent)

# tl-expected — header-only library
# Download the single header directly to avoid FetchContent variable issues
set(TL_EXPECTED_INCLUDE_DIR ${CMAKE_BINARY_DIR}/_deps/tl-expected/include)
set(TL_EXPECTED_HEADER ${TL_EXPECTED_INCLUDE_DIR}/tl/expected.hpp)
file(MAKE_DIRECTORY ${TL_EXPECTED_INCLUDE_DIR}/tl)

if(NOT EXISTS ${TL_EXPECTED_HEADER})
    message(STATUS "Downloading tl-expected.hpp...")
    file(DOWNLOAD
        https://raw.githubusercontent.com/TartanLlama/expected/v1.1.0/include/tl/expected.hpp
        ${TL_EXPECTED_HEADER}
        TLS_VERIFY ON
    )
    message(STATUS "Downloaded tl-expected.hpp")
endif()

# Google Test
FetchContent_Declare(
    googletest
    GIT_REPOSITORY https://github.com/google/googletest.git
    GIT_TAG v1.16.0
)
set(FETCHCONTENT_QUIET OFF)
FetchContent_MakeAvailable(googletest)

# ── Library target: notalterra_backend ─────────────────────────────────────
add_library(notalterra_backend STATIC
    src/gvas/parser.cpp
)

target_include_directories(notalterra_backend PUBLIC
    ${CMAKE_CURRENT_SOURCE_DIR}/src
    ${TL_EXPECTED_INCLUDE_DIR}
)

# ── Test target ───────────────────────────────────────────────────────────
enable_testing()

add_executable(notalterra_tests
    tests/gvas/roundtrip_test.cpp
)

target_link_libraries(notalterra_tests PRIVATE
    notalterra_backend
    GTest::gtest
    GTest::gtest_main
)

target_include_directories(notalterra_tests PRIVATE
    ${CMAKE_CURRENT_SOURCE_DIR}/src
)

include(GoogleTest)
gtest_discover_tests(notalterra_tests)

# ── Dump tool (manual testing) ──────────────────────────────────────────
add_executable(dump_metadata
    tests/gvas/dump_metadata.cpp
)
target_link_libraries(dump_metadata PRIVATE
    notalterra_backend
)
target_include_directories(dump_metadata PRIVATE
    ${CMAKE_CURRENT_SOURCE_DIR}/src
)

# ── GUI target (WinUI 3) ─────────────────────────────────────────────────
# C++/WinRT WinUI 3 desktop app. NuGet packages (CppWinRT, WindowsAppSDK)
# are resolved by MSBuild via Directory.Build.props in the project root.
add_executable(NotAlterra WIN32
    src/ui/App.cpp
)

target_include_directories(NotAlterra PRIVATE
    ${CMAKE_CURRENT_SOURCE_DIR}/src
    ${CMAKE_CURRENT_BINARY_DIR}/winrt  # generated C++/WinRT headers
    "C:/Program Files (x86)/Windows Kits/10/Include/10.0.26100.0/cppwinrt"  # SDK base
    "C:/Users/petemar.UNIVERSE/AppData/Local/Temp/WinAppSDK/include"  # MddBootstrap.h
)

target_link_libraries(NotAlterra PRIVATE
    notalterra_backend
    "C:/Users/petemar.UNIVERSE/AppData/Local/Temp/WinAppSDK/lib/win10-x64/Microsoft.WindowsAppRuntime.lib"
    "C:/Users/petemar.UNIVERSE/AppData/Local/Temp/WinAppSDK/lib/win10-x64/Microsoft.WindowsAppRuntime.Bootstrap.lib"
    "C:/Program Files (x86)/Windows Kits/10/Lib/10.0.26100.0/um/x64/WindowsApp.lib"
    user32
)

set(ALL_BUILD_TARGETS notalterra_backend notalterra_tests dump_metadata NotAlterra)

# Default target builds everything available
add_custom_target(notalterra_all ALL DEPENDS ${ALL_BUILD_TARGETS})


