# Handoff

Keep this short and current. Overwrite (don't append history) each time you hand off —
git history is the log if you need the past. See `agent/WORKFLOW.md` for when to update this.

## As of 2026-11-16 (Phase 6b: slice 2d done and device-checked; next = user test drive)

**Branch:** `rewrite/rust`, task `tasks/active/0009-phase-6b-test-drive.md`. CI green for
`d0dc472`. Slice 2d committed and pushed as `df4c670`.

**Done in 2d:** the head-turn losses were the E2 appearance gate, not LK (with the gate off,
LK follows the synthetic extreme turn within 4.2 px). New `LkParams::anchored`
(`AnchoredUpdate { lag_frames: 10, min_selection_zncc: 0.3 }`), **on in `recommended()`**
(user decision 2026-11-16: default on). Gate passes when ZNCC vs the window accepted 10 steps
earlier ≥ 0.7 and vs the selection ≥ 0.3. Ring reset on `select` and re-detection resume.
CI: `turn_extreme` 90/90 (was 31/90); new adversary scenes `occluder_slide`, `occluder_creep`
at 0 wrong lock / 0 false tracking; fixtures adversarial list runs recommended + plain gate.
Local replay (not in repo): S2 turns losses 5 → 2, S1 unchanged, drift 0 %. The feature
cloud (2d.2) was not built: the cheap candidate met the stop condition.

**Next:** 2d committed (`df4c670`) and device-checked ("turns hold much better now"); verify
CI. Then optional 2e (research report) or the user's test drive → Phase 7 when they say
ready. If turns still lose the point later, take a fresh ≈ 1–2 min S2 recording and replay it
locally (`replay_tune` lists anchored variants and prints each loss).

**Local diagnosis (not in repo):** recordings `~/ocm-recordings/S1-still-2026-11-15`,
`S2-turns-2026-11-15` (may be deleted). Replay tool: `cd ~/ocm-recordings/_tools/replay-tool &&
OCM_REPLAY=<dir> cargo test -q --test replay replay_tune -- --ignored --nocapture`.

**Open from 2a (not blocking):** `ui` ≈ 15/17 ms while Options is open (likely vsync wait);
minimised rate and dwell not re-checked on device; pointer runs at full rate while minimised.

**Machine load:** one heavy command at a time, scoped (`-p`, test filter); release builds:
ask first.

**How the user runs the app:** `export PATH="$HOME/.cargo/bin:$PATH"` then
`cargo run -p open-camera-mouse` (never `--release` for the test drive). Settings:
`~/Library/Application Support/Open Camera Mouse/settings.toml`.

**Known, not bugs (don't "fix" without the user):** Searching animation ignores Reduce Motion
(Phase 8); font is egui default; F10 not F11 on macOS; re-detection off by default; no
auto-recenter.

**User direction (don't drift):** start from 2018 (ADR-0006) but improve on it; new behaviour
ships as options, off by default unless the user approves a default (ADR-0017 and ADR-0018
defaults were approved); monochrome UI, neon green ring marker, yellow primary buttons,
Carbon/Paper, two fixed windows; hotkeys fixed (F10/F12 on macOS); automated checks for every
fix. Recordings: diagnosis only, keep asks ≈ 2 min.

**Toolchain:** `export PATH="$HOME/.cargo/bin:$PATH"`. No `gh`; CI via
`curl -s "https://api.github.com/repos/grga1349/open-camera-mouse/actions/runs?branch=rewrite/rust&per_page=1"`.

**Blockers:** none.
