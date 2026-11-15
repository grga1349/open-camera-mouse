# Handoff

Keep this short and current. Overwrite (don't append history) each time you hand off —
git history is the log if you need the past. See `agent/WORKFLOW.md` for when to update this.

## As of 2026-11-15 (Phase 6b: slice 2b done in code, uncommitted; device check pending)

**Branch:** `rewrite/rust`, task `tasks/active/0009-phase-6b-test-drive.md` (Progress entry
"slice 2b DONE in code"). CI for 2a (`f458e92`) is green. Gates green locally. **Not committed
yet** (ask).

**What 2b became:** recordings are **diagnosis-only** (user rule, see memory). The repo has
no recordings, no benchmark numbers and no code that reads recordings. Findings became synthetic
tests, and three fixes became defaults (ADR-0017): LK texture floor 0.25 + eigen ratio; the 1€
filter; scale by the Main monitor; steady zone 5 px (edges still reachable). Device check: user "resolved jittering good part" at 5.

**Next:** the user checks on device (dev build). **Their saved settings have `steady_px = 0`
and `recenter_display = 0` (the 1440-px laptop)**. They set Options → Pointer → Steady
zone to 5 on device. The scale is now 13.5 screen px per camera px, so they may want a
higher gain. Then 2d: turn losses (`turn_extreme` gap, `AppearanceChanged`).

**Local replay tool (not in repo):**
`/private/tmp/claude-501/-Users-ivangrga-Projects-open-camera-mouse/4d1c3c79-748a-4d65-ba42-fdfae3b09e3d/scratchpad/replay-tool/`
(may be gone; it's a scratch crate with path deps). Recordings: `~/ocm-recordings/S1-*`, `S2-*`
(the user will delete them).

**Open from 2a (not blocking):** `ui` pass ≈ 15/17 ms while Options is open (likely vsync wait
of the immediate viewport; cursor unaffected) — revisit only if the UI feels slow. Minimised
rate and dwell not re-checked on device. Tell the user if relevant: pointer now runs at full
rate while minimised (was ~10 Hz).

**Machine load (user complaint 2026-11-15):** one heavy command at a time, scoped (`-p`,
test filter), say so first; cap local replays with `OCM_REPLAY_FRAMES`. A release build is a "heavy"
command — ask first.

**How the user runs the app:** dev build, `export PATH="$HOME/.cargo/bin:$PATH"` then
`cargo run -p open-camera-mouse` (never `--release` for the test drive). Their settings:
`~/Library/Application Support/Open Camera Mouse/settings.toml`.

**Known, not bugs (don't "fix" without the user):** Searching animation ignores Reduce Motion
(Phase 8); font is egui default; F10 not F11 on macOS; re-detection off by default; no
auto-recenter.

**User direction (don't drift):** start from 2018 (ADR-0006) but improve on it; new behaviour
ships as options, off by default (ask before changing a default; ADR-0017's defaults were
user-approved); monochrome UI, neon green ring
marker, yellow primary buttons, Carbon/Paper, two fixed windows; hotkeys fixed (F10/F12 on
macOS); wants as much as possible checkable by automated tests. Recordings: diagnosis only, keep asks
≈ 2 min.

**Toolchain:** `export PATH="$HOME/.cargo/bin:$PATH"`. No `gh`; CI status via
`curl -s "https://api.github.com/repos/grga1349/open-camera-mouse/actions/runs?branch=rewrite/rust&per_page=1"`
(job logs need auth; failing step names via `.../actions/runs/<id>/jobs`).

**Blockers:** none.
