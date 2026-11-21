---
title: Installation
description: Download, install and first launch on macOS, Windows and Linux, with the permissions each one needs.
order: 3
---

Download the file for your system from the [download section](../../#download) or from
the [latest release](https://github.com/grga1349/open-camera-mouse/releases/latest) on
GitHub. Check the [system requirements](../system-requirements/) first.

Each release also has a `SHA256SUMS` file. To verify a download, compare it with that file:
`shasum -a 256 <file>` on macOS, `sha256sum <file>` on Linux, and
`Get-FileHash <file>` in PowerShell on Windows.

## macOS (Apple Silicon)

1. Open `open-camera-mouse-<version>-macos-arm64.dmg` and drag **Open Camera Mouse** to
   Applications.
2. The app isn't notarized yet, so macOS blocks the first launch. Open it once, then go to
   **System Settings → Privacy & Security** and click **Open Anyway**.
3. Allow **Camera** when macOS asks.
4. Grant **Accessibility**, so the app can move the pointer: **System Settings → Privacy &
   Security → Accessibility**, then turn **Open Camera Mouse** on. The app has a button
   that opens this page and notices the change within a few seconds, without a restart.

Updates keep both permissions. Intel Macs aren't supported.

## Windows (x64)

1. Run `open-camera-mouse-<version>-windows-x64-setup.exe`. It installs for your user only
   and doesn't need administrator rights.
2. The installer isn't signed yet, so SmartScreen may warn you. Click **More info**, then
   **Run anyway**.
3. If Windows asks, allow camera access: **Settings → Privacy & security → Camera**, then
   let desktop apps access your camera.

## Linux (x86_64, X11)

Use an X11 session. On Wayland the app can't move the pointer or catch the keys outside
its window.

Your user needs access to the camera, usually by being in the `video` group.

### AppImage

Make the file executable and run it:

```sh
chmod +x open-camera-mouse-<version>-linux-x86_64.AppImage
./open-camera-mouse-<version>-linux-x86_64.AppImage
```

The AppImage needs FUSE 2 (for example the `libfuse2` package).

### Archive

Unpack `open-camera-mouse-<version>-linux-x86_64.tar.gz` and run `open-camera-mouse`
from it. A `.desktop` file and the icon are included.

---

Next: [Getting started](../getting-started/#first-steps) walks through the first session.
