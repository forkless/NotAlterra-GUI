; Inno Setup script for NotAlterra - MINIMAL TEST
#define MyAppName "NotAlterra"
#define MyAppExeName "NotAlterra.UI.exe"
#ifndef SourceDir
  #define SourceDir "tmp_min"
#endif

[Setup]
AppId={{8FBD6084-3211-4AE3-8E4C-DDE929266317}
AppName={#MyAppName}
AppVerName={#MyAppName}
AppVersion=1.0
DefaultDirName={autopf}\{#MyAppName}
PrivilegesRequired=admin
OutputDir=publish
OutputBaseFilename=na_test_minimal

[Files]
Source: "{#SourceDir}\*"; DestDir: "{app}"; Flags: ignoreversion recursesubdirs createallsubdirs

[Run]
Filename: "{app}\{#MyAppExeName}"; Flags: nowait postinstall skipifsilent
