@echo off
cd /d "%~dp0"

call "C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Auxiliary\Build\vcvars64.bat"

echo === Configuring CMake ===
cmake -B build -G "NMake Makefiles" -DCMAKE_BUILD_TYPE=Release -Wno-dev

echo === Building NotAlterra ===
cmake --build build --config Release --target NotAlterra

echo === Copying bootstrap DLL ===
rem Copy bootstrap DLL alongside the exe
copy "%TEMP%\WinAppSDK\runtimes\win-x64\native\Microsoft.WindowsAppRuntime.Bootstrap.dll" build\ /Y

echo.
echo === Done ===
echo Run: build\NotAlterra.exe
pause
