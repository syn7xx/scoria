' MSI immediate custom action: create Scoria.lnk on the user desktop when CREATE_DESKTOP_SHORTCUT=1.
Function CreateDesktopShortcut()
  On Error Resume Next
  If Session.Property("CREATE_DESKTOP_SHORTCUT") <> "1" Then
    CreateDesktopShortcut = 1
    Exit Function
  End If

  Dim folder, shell, sc
  folder = Session.Property("INSTALLFOLDER")
  If Len(folder) = 0 Then
    CreateDesktopShortcut = 1
    Exit Function
  End If
  If Right(folder, 1) = "\" Then
    folder = Left(folder, Len(folder) - 1)
  End If

  Set shell = CreateObject("WScript.Shell")
  Set sc = shell.CreateShortcut(shell.SpecialFolders("Desktop") & "\Scoria.lnk")
  sc.TargetPath = folder & "\scoria.exe"
  sc.WorkingDirectory = folder
  sc.IconLocation = folder & "\scoria.exe,0"
  sc.Save
  CreateDesktopShortcut = 1
End Function
