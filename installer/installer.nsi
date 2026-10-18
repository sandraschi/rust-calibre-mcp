; Calibre (Rust) v0.4.0 — current-user installer (no admin, no UAC).
; Built with NSIS 3 (makensis installer/installer.nsi, CWD = repo root).
; Installs the egui 0.37 desktop app + the CLI/MCP server side by side.

!include "MUI2.nsh"

Name "Calibre (Rust) 0.4.0"
OutFile "..\target\release\bundle\Calibre-Rust-0.4.0-setup.exe"
InstallDir "$LOCALAPPDATA\Programs\rust-calibre-mcp"
RequestExecutionLevel user

!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_INSTFILES
!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES
!insertmacro MUI_LANGUAGE "English"

Section "Binaries" SecBinaries
  SetOutPath "$INSTDIR"
  File "..\target\release\calibre-app.exe"
  File "..\target\release\rust-calibre-mcp.exe"
  File "..\crates\calibre-app\icons\icon.png"
  WriteUninstaller "$INSTDIR\uninstall.exe"
  WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\rust-calibre-mcp" \
    "DisplayName" "Calibre (Rust) 0.4.0"
  WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\rust-calibre-mcp" \
    "UninstallString" "$INSTDIR\uninstall.exe"
  WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\rust-calibre-mcp" \
    "DisplayIcon" "$INSTDIR\calibre-app.exe"
  WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\rust-calibre-mcp" \
    "DisplayVersion" "0.4.0"
  WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\rust-calibre-mcp" \
    "Publisher" "sandraschi"
SectionEnd

Section "Start Menu shortcuts" SecShortcuts
  CreateDirectory "$SMPROGRAMS\Calibre (Rust)"
  CreateShortcut "$SMPROGRAMS\Calibre (Rust)\Calibre (Rust).lnk" "$INSTDIR\calibre-app.exe"
  CreateShortcut "$SMPROGRAMS\Calibre (Rust)\Uninstall.lnk" "$INSTDIR\uninstall.exe"
SectionEnd

Section "uninstall"
  Delete "$INSTDIR\calibre-app.exe"
  Delete "$INSTDIR\rust-calibre-mcp.exe"
  Delete "$INSTDIR\icon.png"
  Delete "$INSTDIR\uninstall.exe"
  RMDir "$INSTDIR"
  Delete "$SMPROGRAMS\Calibre (Rust)\Calibre (Rust).lnk"
  Delete "$SMPROGRAMS\Calibre (Rust)\Uninstall.lnk"
  RMDir "$SMPROGRAMS\Calibre (Rust)"
  DeleteRegKey HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\rust-calibre-mcp"
SectionEnd
