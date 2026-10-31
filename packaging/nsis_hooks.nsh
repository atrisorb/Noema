; NOEMA Tauri NSIS Hooks: Automatically manage PATH on install and uninstall
!macro NSIS_HOOK_POSTINSTALL
  ExecWait '"$INSTDIR\noema.exe" path setup'
!macroend

!macro NSIS_HOOK_PREUNINSTALL
  ExecWait '"$INSTDIR\noema.exe" path remove "$INSTDIR"'
!macroend
