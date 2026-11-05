; Kanso's Windows installer (Inno Setup 6). The release workflow builds it:
;
;   ISCC.exe /DAppVersion=0.2.0 /DSourceExe=...\kanso.exe /DOutputDir=...\dist
;            /DOutputBase=Kanso-0.2.0-windows-x64-setup packaging\windows\kanso.iss
;
; It installs for the current user only, into %LOCALAPPDATA%\Programs\Kanso,
; so it never asks for an administrator, and Kanso's own updater can swap
; kanso.exe later without one. Notes live apart, in %APPDATA%\Kanso, and
; stay when Kanso is uninstalled unless the person asks for them to go.

#ifndef AppVersion
  #define AppVersion "0.0.0"
#endif
#ifndef SourceExe
  #define SourceExe "..\..\target\release\kanso.exe"
#endif
#ifndef SourceHostExe
  #define SourceHostExe "..\..\target\release\kanso-plugin-host.exe"
#endif
#ifndef SourcePdfWorker
  #define SourcePdfWorker "..\..\target\release\kanso-pdf-worker.exe"
#endif
; The folder scripts/fetch-pdfium.sh made for win-x64.
#ifndef SourcePdfium
  #define SourcePdfium "..\..\vendor\pdfium\win-x64"
#endif
#ifndef OutputDir
  #define OutputDir "..\..\dist"
#endif
; Windows' file version takes only numbers, so a dev build's
; "0.2.0-dev.f66153a" becomes 0.2.0 there; AppVersion keeps the full text.
#ifndef NumericVersion
  #define DashPos Pos("-", AppVersion)
  #if DashPos > 0
    #define NumericVersion Copy(AppVersion, 1, DashPos - 1)
  #else
    #define NumericVersion AppVersion
  #endif
#endif
#ifndef OutputBase
  #define OutputBase "Kanso-" + AppVersion + "-windows-x64-setup"
#endif

[Setup]
; Never change AppId: Windows recognises an installed Kanso by it, so an
; installer for a newer version upgrades in place.
AppId={{6F0B8E52-3C1D-4D6A-9B7E-5A2C4E1F8D30}
AppName=Kanso
AppVersion={#AppVersion}
AppVerName=Kanso {#AppVersion}
AppPublisher=Ngonidzashe Mangudya
AppPublisherURL=https://github.com/iamngoni/kanso
AppSupportURL=https://github.com/iamngoni/kanso/issues
AppUpdatesURL=https://github.com/iamngoni/kanso/releases
VersionInfoVersion={#NumericVersion}
DefaultDirName={userpf}\Kanso
DisableDirPage=yes
DisableProgramGroupPage=yes
PrivilegesRequired=lowest
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
MinVersion=10.0
SetupIconFile={#SourcePath}\..\..\assets\icon\kanso.ico
UninstallDisplayIcon={app}\kanso.exe
UninstallDisplayName=Kanso
OutputDir={#OutputDir}
OutputBaseFilename={#OutputBase}
Compression=lzma2/max
SolidCompression=yes
WizardStyle=modern
; Close a running Kanso before replacing it, and open it again afterwards.
CloseApplications=yes
RestartApplications=yes

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked

[Files]
Source: "{#SourceExe}"; DestDir: "{app}"; DestName: "kanso.exe"; Flags: ignoreversion
; Plugins run in their own process, beside Kanso (docs/PLUGINS.md).
Source: "{#SourceHostExe}"; DestDir: "{app}"; DestName: "kanso-plugin-host.exe"; Flags: ignoreversion
; PDFs are drawn in a contained helper (an AppContainer) beside its PDFium
; library, with PDFium's licences (issue #43).
Source: "{#SourcePdfWorker}"; DestDir: "{app}"; DestName: "kanso-pdf-worker.exe"; Flags: ignoreversion
Source: "{#SourcePdfium}\bin\pdfium.dll"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#SourcePdfium}\LICENSE"; DestDir: "{app}\pdfium-licenses"; Flags: ignoreversion
Source: "{#SourcePdfium}\licenses\*"; DestDir: "{app}\pdfium-licenses"; Flags: ignoreversion recursesubdirs

[Icons]
Name: "{userprograms}\Kanso"; Filename: "{app}\kanso.exe"
Name: "{userdesktop}\Kanso"; Filename: "{app}\kanso.exe"; Tasks: desktopicon

[Registry]
; The kanso:// scheme, for this user: Sign in with Apple's page returns to
; Kanso through it, and a second kanso.exe hands the URL to the running one.
Root: HKCU; Subkey: "Software\Classes\kanso"; ValueType: string; ValueName: ""; ValueData: "URL:Kanso"; Flags: uninsdeletekey
Root: HKCU; Subkey: "Software\Classes\kanso"; ValueType: string; ValueName: "URL Protocol"; ValueData: ""
Root: HKCU; Subkey: "Software\Classes\kanso\DefaultIcon"; ValueType: string; ValueName: ""; ValueData: "{app}\kanso.exe,0"
Root: HKCU; Subkey: "Software\Classes\kanso\shell\open\command"; ValueType: string; ValueName: ""; ValueData: """{app}\kanso.exe"" ""%1"""
; Open with > Kanso for Markdown files, for this user. Kanso is listed there
; and does not become the default program. A second kanso.exe started with
; the file hands it to the running one.
Root: HKCU; Subkey: "Software\Classes\Applications\kanso.exe"; ValueType: string; ValueName: "FriendlyAppName"; ValueData: "Kanso"; Flags: uninsdeletekey
Root: HKCU; Subkey: "Software\Classes\Applications\kanso.exe\shell\open\command"; ValueType: string; ValueName: ""; ValueData: """{app}\kanso.exe"" ""%1"""
Root: HKCU; Subkey: "Software\Classes\Applications\kanso.exe\SupportedTypes"; ValueType: string; ValueName: ".md"; ValueData: ""
Root: HKCU; Subkey: "Software\Classes\Applications\kanso.exe\SupportedTypes"; ValueType: string; ValueName: ".markdown"; ValueData: ""
Root: HKCU; Subkey: "Software\Classes\.md\OpenWithProgids"; ValueType: string; ValueName: "Applications\kanso.exe"; ValueData: ""; Flags: uninsdeletevalue
Root: HKCU; Subkey: "Software\Classes\.markdown\OpenWithProgids"; ValueType: string; ValueName: "Applications\kanso.exe"; ValueData: ""; Flags: uninsdeletevalue

[Run]
Filename: "{app}\kanso.exe"; Description: "{cm:LaunchProgram,Kanso}"; Flags: nowait postinstall skipifsilent

[UninstallDelete]
; What Kanso's updater leaves beside itself: the previous exe, kept until
; the next start, and its staging folder.
Type: files; Name: "{app}\kanso.exe.old"
Type: filesandordirs; Name: "{app}\.kanso-update"

[Code]
procedure CurUninstallStepChanged(CurUninstallStep: TUninstallStep);
var
  Notes: String;
begin
  if CurUninstallStep <> usPostUninstall then
    Exit;
  Notes := ExpandConstant('{userappdata}\Kanso');
  if not DirExists(Notes) then
    Exit;
  // Kept unless the person says otherwise; a silent uninstall never asks.
  if UninstallSilent then
    Exit;
  if MsgBox('Also delete your Kanso notes and settings on this computer?' + #13#10 + #13#10 +
            Notes + #13#10 + #13#10 +
            'Notes synced to your account stay there. Choose No to keep them here too.',
            mbConfirmation, MB_YESNO or MB_DEFBUTTON2) = IDYES then
    DelTree(Notes, True, True, True);
end;
