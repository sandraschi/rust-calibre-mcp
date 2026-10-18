; NSIS Installer Script for rust-calibre-mcp & Calibre ArtCraft Desktop App
; Builds a self-contained Windows installer packaging:
;   - calibre-app.exe (L3 native egui 0.37 desktop application)
;   - rust-calibre-mcp.exe (L2 CLI + MCP 2024-11-05 stdio server)
;   - icon.png

!include "MUI2.nsh"

Name "Calibre ArtCraft & rust-calibre-mcp"
OutFile "rust-calibre-mcp-setup.exe"
InstallDir "$LOCALAPPDATA\Programs\rust-calibre-mcp"
InstallDirRegKey HKCU "Software\rust-calibre-mcp" "Install_Dir"
RequestExecutionLevel user

!define MUI_ABORTWARNING

!insertmacro MUI_PAGE_WELCOME
!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_INSTFILES
!insertmacro MUI_PAGE_FINISH

!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES

!insertmacro MUI_LANGUAGE "English"

Section "Calibre ArtCraft & MCP Server (required)" SecMain
  SectionIn RO
  SetOutPath "$INSTDIR"

  File "..\target\release\calibre-app.exe"
  File "..\target\release\rust-calibre-mcp.exe"
  File "..\crates\calibre-app\icons\icon.png"

  WriteRegStr HKCU "Software\rust-calibre-mcp" "Install_Dir" "$INSTDIR"
  WriteUninstaller "$INSTDIR\uninstall.exe"

  CreateDirectory "$SMPROGRAMS\Calibre ArtCraft"
  CreateShortCut "$SMPROGRAMS\Calibre ArtCraft\Calibre ArtCraft.lnk" "$INSTDIR\calibre-app.exe"
  CreateShortCut "$SMPROGRAMS\Calibre ArtCraft\Uninstall.lnk" "$INSTDIR\uninstall.exe"
SectionEnd

Section "Uninstall"
  Delete "$INSTDIR\calibre-app.exe"
  Delete "$INSTDIR\rust-calibre-mcp.exe"
  Delete "$INSTDIR\icon.png"
  Delete "$INSTDIR\uninstall.exe"

  Delete "$SMPROGRAMS\Calibre ArtCraft\Calibre ArtCraft.lnk"
  Delete "$SMPROGRAMS\Calibre ArtCraft\Uninstall.lnk"
  RMDir "$SMPROGRAMS\Calibre ArtCraft"
  RMDir "$INSTDIR"

  DeleteRegKey HKCU "Software\rust-calibre-mcp"
SectionEnd
