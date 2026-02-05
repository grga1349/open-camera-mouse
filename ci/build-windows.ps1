$ErrorActionPreference = "Stop"

$env:PATH = "C:\msys64\mingw64\bin;C:\msys64\usr\bin;$env:PATH"

if (-not (Get-Command node -ErrorAction SilentlyContinue)) {
  throw "node not found in PATH. Ensure actions/setup-node ran."
}
node --version
npm --version

if (-not (Get-Command go -ErrorAction SilentlyContinue)) {
  throw "go not found in PATH. Ensure actions/setup-go ran."
}

if (-not (Get-Command wails -ErrorAction SilentlyContinue)) {
  $goPath = (go env GOPATH)
  $gopathBin = (cygpath -u $goPath) + "/bin"
  $env:PATH = "$gopathBin;$env:PATH"
}

if (-not (Get-Command wails -ErrorAction SilentlyContinue)) {
  throw "wails not found in PATH. Ensure go install wails CLI succeeded."
}
wails version

$env:PKG_CONFIG_PATH = "$PWD\ci"
$env:CGO_ENABLED = "1"

$cflags = bash -lc "pkg-config --cflags opencv4-nogui"
$libs = bash -lc "pkg-config --libs opencv4-nogui"

$env:CGO_CFLAGS = $cflags
$env:CGO_CXXFLAGS = $cflags
$env:CGO_LDFLAGS = $libs
$env:CC = "gcc"
$env:CXX = "g++"

if (Test-Path "build/windows/icon.ico") {
  Remove-Item "build/windows/icon.ico" -Force
}

wails build -clean -skipbindings -tags "customenv" --nsis -ldflags "-H=windowsgui"
