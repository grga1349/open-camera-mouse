#!/usr/bin/env bash
set -e
VERSION=$(cat VERSION)
TAG=${GITHUB_REF_NAME#v}
if [ "$VERSION" != "$TAG" ]; then
  echo "VERSION ($VERSION) does not match tag ($TAG)" >&2
  exit 1
fi
