#!/usr/bin/env bash
set -euo pipefail
GO_BIN=$(ls -d /c/hostedtoolcache/windows/go/*/x64/bin | sort -V | tail -1)
NODE_BIN=$(ls -d /c/hostedtoolcache/windows/node/*/x64 | sort -V | tail -1)
export PATH="$GO_BIN:$NODE_BIN:/mingw64/bin:/usr/bin:$PATH"
export PATH="$(cygpath -u "$(go env GOPATH)")/bin:$PATH"
export CC=gcc
export CXX=g++
export CGO_ENABLED=1
export CGO_CPPFLAGS="$(pkg-config --cflags opencv4)"
export CGO_CXXFLAGS="$CGO_CPPFLAGS"
export CGO_LDFLAGS="$(pkg-config --libs opencv4)"
VERSION=$(cat VERSION)
wails build \
  -clean \
  -skipbindings \
  -tags customenv \
  -ldflags "-H=windowsgui -X main.version=${VERSION}"
