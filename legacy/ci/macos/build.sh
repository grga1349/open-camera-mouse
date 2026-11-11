#!/usr/bin/env bash
set -euo pipefail
VERSION=$(cat VERSION)
export CGO_ENABLED=1
export PKG_CONFIG_PATH="$(brew --prefix opencv)/lib/pkgconfig:$(brew --prefix)/lib/pkgconfig${PKG_CONFIG_PATH:+:$PKG_CONFIG_PATH}"
export CGO_CFLAGS="$(pkg-config --cflags opencv4)"
export CGO_CXXFLAGS="$CGO_CFLAGS -std=c++17"
export CGO_LDFLAGS="$(pkg-config --libs opencv4) -Wl,-headerpad_max_install_names"
export PATH="$(go env GOPATH)/bin:$PATH"
export MACOSX_DEPLOYMENT_TARGET=14.0
wails build \
  -clean \
  -platform darwin/arm64 \
  -tags customenv \
  -skipbindings \
  -ldflags "-X main.version=${VERSION}"
