[Code]
const
  DotNet9_URL = 'https://builds.dotnet.microsoft.com/dotnet/WindowsDesktop/9.0.17/windowsdesktop-runtime-9.0.17-win-x64.exe';
  DotNet9_Help = 'https://dotnet.microsoft.com/en-us/download/dotnet/9.0';
  WinAppSDK_URL = 'https://aka.ms/windowsappsdk/1.8/1.8.260317003/windowsappruntimeinstall-x64.exe';
  WinAppSDK_Help = 'https://learn.microsoft.com/en-us/windows/apps/windows-app-sdk/downloads';

function URLDownloadToFile(pCaller: Integer; szURL: string; szFileName: string; dwReserved: Integer; lpfnCB: Integer): Integer;
  external 'URLDownloadToFileW@urlmon.dll stdcall';

function TryInstall(Url: string; FileName: string; DisplayName: string; HelpUrl: string): Boolean;
var
  Path: string;
  ResultCode: Integer;
begin
  Result := False;
  Path := ExpandConstant('{tmp}\' + FileName);
  if URLDownloadToFile(0, Url, Path, 0, 0) = 0 then
  begin
    if Exec(Path, '', '', SW_SHOW, ewWaitUntilTerminated, ResultCode) and (ResultCode = 0) then
      Result := True
    else
    begin
      MsgBox(DisplayName + ' installer failed.', mbError, MB_OK);
      if MsgBox('Open download page?', mbConfirmation, MB_YESNO) = IDYES then
        ShellExec('open', HelpUrl, '', '', SW_SHOW, ewNoWait, ResultCode);
    end;
  end
  else
  begin
    MsgBox('Could not download ' + DisplayName + '.', mbError, MB_OK);
    if MsgBox('Open download page?', mbConfirmation, MB_YESNO) = IDYES then
      ShellExec('open', HelpUrl, '', '', SW_SHOW, ewNoWait, ResultCode);
  end;
end;

function NextButtonClick(CurPageID: Integer): Boolean;
begin
  Result := True;
  if CurPageID = wpWelcome then
  begin
    if MsgBox('Download and install .NET 9 Desktop Runtime?', mbConfirmation, MB_YESNO) = IDYES then
      TryInstall(DotNet9_URL, 'dotnet9-win-x64.exe', '.NET 9', DotNet9_Help);
    if MsgBox('Download and install Windows App SDK 1.8?', mbConfirmation, MB_YESNO) = IDYES then
      TryInstall(WinAppSDK_URL, 'WinAppSDK-x64.exe', 'WinAppSDK 1.8', WinAppSDK_Help);
  end;
end;
