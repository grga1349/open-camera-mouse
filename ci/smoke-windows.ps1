$ErrorActionPreference = "Stop"

$exe = Get-ChildItem "build/bin" -Filter *.exe | Select-Object -First 1
if (-not $exe) {
  throw "No .exe found in build/bin"
}

$deps = & bash -lc "objdump -p `"$($exe.FullName -replace '\\','/')`" | grep 'DLL Name:' | awk '{print `$3}' | sort -u"

if ($deps -match "opencv_highgui" -or $deps -match "Qt6") {
  throw "Qt/HighGUI dependency detected in exe dependencies."
}

$systemRegex = '^(KERNEL32|USER32|GDI32|ADVAPI32|SHELL32|OLE32|OLEAUT32|WS2_32|SHLWAPI|COMDLG32|IMM32|WINMM|UCRTBASE|VCRUNTIME140|VCRUNTIME140_1|MSVCP140|MSVCP140_1|MSVCP140_2|MSVCP140_ATOMIC_WAIT|MSVCP140_CODECVT_IDS|VERSION|COMCTL32|WINSPOOL|api-ms-win|bcrypt|CRYPT32|ntdll|RPCRT4|SETUPAPI|USERENV|UxTheme|dwmapi|iphlpapi|MPR|NETAPI32|Secur32|WINHTTP|urlmon)\.dll$'

foreach ($dll in $deps) {
  if ($dll -match $systemRegex) { continue }
  $path = Join-Path $exe.DirectoryName $dll
  if (-not (Test-Path $path)) {
    throw "Missing required DLL next to exe: $dll"
  }
}

$p = Start-Process -FilePath $exe.FullName -PassThru
Start-Sleep -Seconds 2
if ($p.HasExited -and $p.ExitCode -ne 0) {
  throw "Process exited early with code $($p.ExitCode)"
}
if (-not $p.HasExited) {
  Stop-Process -Id $p.Id -Force
}
