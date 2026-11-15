# ADR-0012: Phase 6 dependencies and settings persistence

Status: Accepted (persistence D2, camera D3); pointer and hotkey rows are filled in by D4–D5
Date: 2026-11-12
Task: 0008 slice D2. Amends ADR-0007 (what `app-core` may depend on).

## Context

Phase 6 needs its first crates outside egui: settings persistence now (D2), then a camera
backend (D3), pointer injection (D4) and global hotkeys (D5). ADR-0007 keeps the cores free of
I/O, OS and GUI crates, but it doesn't say whether `app-core` may use `serde` to describe the
settings it already owns. `deny.toml` allows permissive licences only (no MPL, no GPL).

Versions and licences re-queried from the crates.io API on 2026-11-12. Licences of every
crate newly added to `Cargo.lock` were checked against `deny.toml`.

## Options considered

**Where the serde derives live**
1. `#[derive(Serialize, Deserialize)]` on `app_core::Settings`, with `serde` in `app-core`. One
   struct, so field names on disk can't drift from the code. `serde` is pure data mapping (no
   I/O, no OS, no GUI) and builds on every target.
2. A mirror DTO in `platform` with hand-written conversions. Keeps `app-core` to `thiserror`
   only, but every new setting has to be added in two places, and a missed field is silently
   dropped on save.

**Config directory**
1. `directories` 6.0.0. Pulls in `dirs-sys` → `option-ext` 0.2.0, which is **MPL-2.0**, so
   `cargo deny` fails. Allowing it would mean a licence exception for one path lookup.
2. A small hand-written function in `platform` (three OS rules, tested on every OS with
   injected environment values). No new dependency.

**Format**: TOML (`toml` 1.2.1) vs JSON. TOML is hand-editable with comments, and the file is
small and flat. `toml` was already in the tree through eframe's dependencies.

## Decision

- `app-core` may depend on `serde` (`derive` + `std`, no other features) for `Settings` and its
  nested types, besides `thiserror` and the two cores. No I/O crates, no `toml`, no OS or GUI
  crates. `scripts/check-core-deps.sh` now checks `app-core` against that allowlist too.
- Persistence lives in `platform::settings_store` (`toml`, `std::fs`). The config directory is
  computed by hand-written code, not `directories`:
  macOS `~/Library/Application Support/Open Camera Mouse/`, Windows
  `%APPDATA%\Open Camera Mouse\`, Linux/Unix `$XDG_CONFIG_HOME/open-camera-mouse/` or
  `~/.config/open-camera-mouse/`. Relative environment values are ignored.
- File: `settings.toml`, `schema_version` on the first line, then `Settings` fields. Missing
  fields take defaults, unknown fields are ignored (within one schema version). Writes are
  atomic (temporary file + rename).
- Loading never fails; each fallback comes with a user-facing notice (R-A5):

| File | Settings | Store |
|---|---|---|
| missing | defaults, no notice | writable |
| unparsable, wrong types, missing or older `schema_version` | defaults; file moved to `settings.toml.corrupt` | writable |
| newer `schema_version` | defaults | read-only (an older build never overwrites a newer file) |
| values out of range | only those fields reset (`Settings::repaired`) | writable |
| unreadable (permissions, I/O) | defaults | read-only |

### Crates

| Crate | Version | Licence | Where | Status |
|---|---|---|---|---|
| `serde` | 1.0.229 | MIT OR Apache-2.0 | app-core (derive) | **adopted** |
| `toml` | 1.2.1 | MIT OR Apache-2.0 | platform | **adopted** |
| `tempfile` | 3.27.0 | MIT OR Apache-2.0 | platform (dev) | **adopted** |
| `directories` | 6.0.0 | MIT OR Apache-2.0, but `option-ext` is MPL-2.0 | — | **rejected** |
| `nokhwa` (`input-native` only, no `decoding`) | 0.10.11 | Apache-2.0 (51 new transitive crates, all permissive, checked 2026-11-15) | platform (all OS) | **adopted** (D3, measured below) |
| `objc2-av-foundation` | 0.3.2 | Zlib OR Apache-2.0 OR MIT | — | **fallback**, not built (D3): `nokhwa` met every gate; would need `unsafe` + an ADR |
| `objc2-core-graphics` | 0.3.2 | Zlib OR Apache-2.0 OR MIT | platform (macOS) | candidate, D4 |
| `objc2-application-services` | 0.3.2 | Zlib OR Apache-2.0 OR MIT | platform (macOS) | candidate, D4 (Accessibility check) |
| `windows-sys` | 0.61.2 | MIT OR Apache-2.0 | platform (Windows) | candidate, D4 |
| `x11rb` | 0.14.0 | MIT OR Apache-2.0 | platform (Linux) | candidate, D4 |
| `global-hotkey` | 0.10.0 | Apache-2.0 OR MIT | platform | candidate, D5 |
| `enigo` | 0.6.1 | MIT | — | fallback only |

D3, D4 and D5 each update their row here (adopted or rejected, with the measurement) and add
the crate to `platform` per target. Check every new transitive licence before adopting:
`cargo deny` runs only in CI.

## Camera backend (task 0008 D3, 2026-11-15)

Measured on the dev Mac (macOS 14.7.3 arm64, built-in FaceTime HD Camera), first in a
throwaway spike, then through the adapter on a worker thread
(`crates/platform/tests/camera_device.rs`, `#[ignore]`d, run with `--ignored`):

| Check | Result |
|---|---|
| Opens | yes, permission already granted to the terminal |
| 640×480 negotiated | yes, YUYV (`yuvs`), reported 30 fps |
| Real FPS (timestamps, 120 frames after 30 warm-up) | 30.09 (adapter), 30.00–30.02 (spike) |
| Open → first frame | cold 1.2–1.4 s, warm 0.11–0.21 s |
| Close (stop stream + drop) | 76–97 ms. Camera LED going off (nokhwa #224) not checked by the agent: D7 manual QA |
| Re-open after close in one process | yes, same format and FPS |
| Panics | none on this camera; the P1 panics are reported on external webcams (C922), not reproducible here |

Findings that shaped the adapter:
- `nokhwa` 0.10.11 on macOS maps an **NV12** request to a 10-bit format the FaceTime camera
  refuses ("Cannot fulfill request"), and maps every 4:2:0 output to `YUYV` by mistake. The
  adapter asks for YUYV first and asks for NV12 only off macOS.
- `compatible_camera_formats()` returns an empty list on macOS, so the adapter can't pick from
  a list. It tries requests in order: YUYV 640×480@30, NV12 (not macOS), highest frame rate at
  640×480, driver default.
- `frame()` blocks with no timeout. Disconnect detection therefore lives on the reader side
  (no frame for 3 s → `Disconnected`), and `close` waits at most 2 s for the capture thread.
- `decoding` (JPEG via `mozjpeg`) stays off. A camera that only offers MJPEG gets a `Backend`
  error naming it. If Phase 7 hardware needs MJPEG, add a pure-Rust decoder through an ADR.
- `nokhwa`'s macOS bindings pull in `block` 0.1.6, which raises a future-incompatibility
  warning (not an error) on the pinned toolchain. Watch it on toolchain bumps. It's the
  strongest argument for the `objc2-av-foundation` fallback later.

Why not the thin `objc2-av-foundation` adapter now: `nokhwa` passed every gate above and covers
Windows and Linux with the same code, while the objc2 adapter would need `unsafe`
(the workspace denies it), a delegate class and its own Windows/Linux counterparts. It stays the
fallback if P1 panics or the `block` warning become real problems in Phase 7.

## Consequences

- One `Settings` type is the in-memory model, the validation and the on-disk schema. Renaming
  a field is a schema change: bump `Settings::SCHEMA_VERSION` and add a migration in
  `settings_store::decode`, which now rejects older versions as corrupt because none have
  shipped.
- A hand-edited file with one bad value keeps every other setting.
- Running an older build after a newer one shows defaults and saves nothing until the newer
  build is back. That's preferable to losing the newer settings.
- The hand-written Windows path reads `%APPDATA%` instead of calling
  `SHGetKnownFolderPath`. If Phase 7 finds a setup where the variable is missing or wrong, the
  fix is that API through `windows-sys` (already a candidate), not `directories`.
- Saving on change is wired with the Options window (D6). D2 loads at startup and shows the
  notice in the skeleton window.
