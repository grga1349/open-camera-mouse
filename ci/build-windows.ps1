$ErrorActionPreference = "Stop"

Write-Host "== Preflight =="
node --version
npm --version

$env:PATH = "C:\msys64\mingw64\bin;C:\msys64\usr\bin;" + $env:PATH

$gopath = (go env GOPATH)
$env:PATH = "$gopath\bin;" + $env:PATH

wails version

$env:CC = "gcc"
$env:CXX = "g++"
$env:CGO_ENABLED = "1"

$pcPath = (Resolve-Path ".\ci").Path
$env:PKG_CONFIG_PATH = $pcPath

& C:\msys64\usr\bin\bash -lc "true" | Out-Null
$cflagsRaw = & C:\msys64\usr\bin\bash -lc "PATH=/mingw64/bin:/usr/bin:\$PATH pkg-config --cflags opencv4-nogui"
$libsRaw   = & C:\msys64\usr\bin\bash -lc "PATH=/mingw64/bin:/usr/bin:\$PATH pkg-config --libs opencv4-nogui"

$cflags = ($cflagsRaw | Out-String).Trim()
$libs   = ($libsRaw   | Out-String).Trim()

if ([string]::IsNullOrWhiteSpace($cflags) -or [string]::IsNullOrWhiteSpace($libs)) {
  throw "pkg-config returned empty flags. cflags='$cflags' libs='$libs'"
}

Write-Host "== pkg-config flags =="
Write-Host "CFLAGS: $cflags"
Write-Host "LDFLAGS: $libs"

$env:CGO_CFLAGS   = $cflags
$env:CGO_CXXFLAGS = $cflags
$env:CGO_LDFLAGS  = $libs

Write-Host "== Build =="
wails build -clean -skipbindings -tags "customenv" --nsis -ldflags "-H=windowsgui"
