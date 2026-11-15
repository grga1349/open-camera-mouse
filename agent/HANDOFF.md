# Handoff

Keep this short and current. Overwrite (don't append history) each time you hand off —
git history is the log if you need the past. See `agent/WORKFLOW.md` for when to update this.

## As of 2026-11-15 (Phase 6b test drive: part 2 PLANNED, next session = slice 2a)

**Branch:** `rewrite/rust`, task `tasks/active/0009-phase-6b-test-drive.md` — **read its
"Part 2 plan" section first**; it is the agreed plan (slices 2a → 2b → 2c → 2d, one per
session, plus a "Deferred" list), built from `docs/research/HEAD_POINTER_STABILITY_INPUTS.md`
(the user's online research + 2018 RE brief; plan items cite it as A §n / B §n). Issue log
`docs/qa/TEST_DRIVE.md` (#1–#6). Part 1 and this planning session are committed and pushed;
CI for the push not checked yet (check it first). The `OCM_PERF=1` timing report
(`apps/desktop/src/perf.rs`) is in; desktop crate fmt/clippy/71 tests passed locally, the
full workspace gates were not re-run (one crate at a time, see "Machine load").

**Next action: slice 2a (issue #6 lag, ADR-0016).** Measured: the main thread spends ≈ 31 of
33 ms per frame (`track` 19 ms + preview `upload` 11.6 ms, dev build), frames wait 42 ms before
the controller, cursor ≈ 60 ms behind the camera. Steps: benches to find the 19 ms → dev
profile opt-levels → ADR-0016 → engine thread owning controller + camera slot + pointer, UI
reads snapshots → controller runs on capture timestamps. Targets and tests are in the plan.
Then ask the user to re-run `OCM_PERF=1 cargo run -p open-camera-mouse 2>&1 | grep perf`.
Expectation set with the user: 2a fixes lag; #5 only slightly (timestamp dt) until 2c; #2 not
until 2d. Offered, not agreed: pulling the gain-per-display option into 2a — ask at the start.

**Then:** 2b scripted recordings (`~/ocm-recordings`, NOT /tmp) + `replay_bench`; 2c cursor
stability (#5: gain per display, 1€ filter); 2d tracking continuity (#2/#3: feature-cloud
virtual anchor, with stop condition). #2 and #5 remain the user's top priority; 2a and 2b are
prerequisites, not detours.

**Replay tools** (`crates/app-core/tests/replay.rs`, all `#[ignore]`):
`OCM_REPLAY=<dir> cargo test --release -p app-core --test replay <name> -- --ignored --nocapture`.
`replay_recording`, `replay_tune`, `replay_redetect_scores`, `replay_redetect_best_positions`,
`replay_context_zncc`. Old recording `rec-1` is in an earlier session's scratchpad under
`/private/tmp/claude-501/…/1d908c94-…/scratchpad/` and will vanish; don't rely on it.

**Machine load (user complaint 2026-11-15):** one heavy command at a time, scoped (`-p`,
test filter), say so first; cap replays with `OCM_REPLAY_FRAMES`. A release build is a "heavy"
command — ask first.

**How the user runs the app:** dev build, `export PATH="$HOME/.cargo/bin:$PATH"` then
`cargo run -p open-camera-mouse` (never `--release` for the test drive; `--release` only for a
measurement, with the user's OK). Their settings:
`~/Library/Application Support/Open Camera Mouse/settings.toml`.

**Known, not bugs (don't "fix" without the user):** Searching animation ignores Reduce Motion
(Phase 8); font is egui default; F10 not F11 on macOS; re-detection off by default; no
auto-recenter. Pointer ~10 Hz while minimised — will change as a side effect of 2a; tell the user.

**User direction (don't drift):** start from 2018 (ADR-0006) but improve on it; new behaviour
ships as options, off by default (ask before changing a default); monochrome UI, neon green ring
marker, yellow primary buttons, Carbon/Paper, two fixed windows; hotkeys fixed (F10/F12 on
macOS); wants as much as possible checkable by automated tests.

**Toolchain:** `export PATH="$HOME/.cargo/bin:$PATH"`. No `gh`; check CI with
`curl -s "https://api.github.com/repos/grga1349/open-camera-mouse/actions/runs?branch=rewrite/rust&per_page=1"`.

**Blockers:** none. 2b needs ~10 min of recording from the user after 2a.
