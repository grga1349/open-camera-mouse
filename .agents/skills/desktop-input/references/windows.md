# Windows: SetCursorPos, SendInput and what winit sees

Sources: Microsoft Learn pages for `SendInput`, `MOUSEINPUT`, `SetCursorPos`,
`GetAsyncKeyState`, `WM_MOUSEMOVE`, `MSLLHOOKSTRUCT` (fetched 2026-11-25, quoted as "MS
docs"); windows-sys 0.61.2; winit 0.30.13 `platform_impl/windows`; OCM
`crates/platform/src/pointer/windows.rs`. OCM on Windows is compiled in CI only; nothing
below was run on a Windows device.

## Moving

- OCM moves with `SetCursorPos(x, y)` in screen coordinates (MS docs), clicks with
  `SendInput` LEFTDOWN+LEFTUP without position (the cursor is already there).
- `SetCursorPos` (MS docs): clamped to a `ClipCursor` rectangle; needs
  `WINSTA_WRITEATTRIBUTES`; "The input desktop must be the current desktop". Docs also say
  "A window should move the cursor only when the cursor is in the window's client area" —
  advice OCM by nature ignores. Not documented: whether it produces `WM_MOUSEMOVE` (commonly
  observed yes) or raw input (commonly observed no) — unverified.
- `SendInput` alternative: `MOUSEEVENTF_MOVE | MOUSEEVENTF_ABSOLUTE` takes normalized
  0–65535; without `MOUSEEVENTF_VIRTUALDESK` it maps to the **primary monitor**, with it to
  the whole virtual desktop. Relative moves are subject to pointer speed and acceleration
  thresholds (up to ×4). `MOUSEEVENTF_MOVE_NOCOALESCE` stops `WM_MOUSEMOVE` coalescing
  (MS docs). Switching OCM to `SendInput` moves is an architectural change: measure and ADR.
- winit emits `DeviceEvent::MouseMotion` only from relative raw-input packets
  (`handle_raw_input`, `MOUSE_MOVE_RELATIVE`), so neither `SetCursorPos` nor absolute
  `SendInput` moves reach egui as `MouseMoved` deltas; apps that steer by raw deltas (games,
  some CAD) may ignore camera motion.

## Clicking and button state

- `SendInput` inserts events "serially", "not interspersed" with other input; it is
  "subject to UIPI": only into equal-or-lower integrity; on failure "neither GetLastError
  nor the return value will indicate the failure was caused by UIPI" (MS docs). OCM checks
  `sent == 2`; a UIPI block may still look like a plain backend error or not at all on the
  target. Secure desktop (UAC, lock screen) is out of reach (P8).
- Button flags "indicate changes in status, not ongoing conditions" (MS docs): a held
  physical button stays down while the camera calls `SetCursorPos`; whether the target
  window then sees `WM_MOUSEMOVE` with `MK_LBUTTON` (drag continues) is likely but
  unverified — the macOS `…Dragged` problem may not exist here; check on device.
- Held buttons: `GetAsyncKeyState(VK_LBUTTON)` reports the **physical** left button
  regardless of swap (`SM_SWAPBUTTON`); returns 0 when UIPI or a non-active desktop blocks
  it (MS docs). Whether it includes `SendInput`-injected buttons is not documented
  (unverified). Needed to avoid releasing a physical drag on Windows.
- Telling synthetic from physical: low-level mouse hook `MSLLHOOKSTRUCT.flags`
  `LLMHF_INJECTED` / `LLMHF_LOWER_IL_INJECTED`; `MOUSEINPUT.dwExtraInfo` readable via
  `GetMessageExtraInfo` in the receiving thread (MS docs). Not exposed by winit; a hook is a
  new platform mechanism (ADR first).

## What OCM's own windows get (winit)

- Any button down → `SetCapture` (refcounted), up → `ReleaseCapture`. `WM_MOUSEMOVE` goes
  to the capturing window ("Otherwise, the message is posted to the window that has
  captured the mouse", MS docs) with client-relative, signed coordinates (negative on
  multi-monitor; use `GET_X_LPARAM`, not `LOWORD`).
- winit turns leaving the client rect into `CursorLeft`, then keeps sending `CursorMoved`
  outside it; egui gets `PointerGone` + `PointerMoved` and the drag continues.
- Focus: winit windows are normal activating windows; a click on the inactive OCM window
  activates it and is delivered (no `WM_MOUSEACTIVATE` override in winit 0.30.13 — grep
  found none; behaviour unverified on device).

## Coordinates and DPI

- winit makes the process per-monitor DPI aware (V2, with fallbacks) when the event loop is
  created. Then `GetCursorPos`, `SetCursorPos`, `GetMonitorInfoW` all use physical pixels
  of the virtual screen, primary monitor at (0,0), others may be negative — the
  controller's desktop. Create `SystemPointer` only after the event loop exists (OCM does,
  in the engine factory). Mixed-DPI layouts are untested (P9).
- egui points = physical px / `pixels_per_point` of that window's monitor.
- `MSLLHOOKSTRUCT.pt` is in per-monitor-aware screen coordinates (MS docs).

## Hotkeys

global-hotkey 0.10: `RegisterHotKey` on a hidden message-only window; the manager must be
created on the thread running the Win32 message loop (crate docs). F11/F12 may already be
registered by another app → `Registration` failure shown in the UI (P13). Whether
`RegisterHotKey` also lets the focused window see `WM_KEYDOWN` is unverified; `KeyRouter`
de-duplicates.
