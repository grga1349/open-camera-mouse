# Handoff

Keep this short and current. Overwrite (don't append history) each time you hand off —
git history is the log if you need the past. See `agent/WORKFLOW.md` for when to update this.

## As of 2026-11-15 (Phase 6b: slice 2a DONE and pushed; next session = verify CI, then 2b)

**Branch:** `rewrite/rust`, task `tasks/active/0009-phase-6b-test-drive.md` (Progress entry
"slice 2a" + "Part 2 plan"). Slice 2a (issue #6 lag, ADR-0016 engine thread) is committed and
pushed; its device runs are recorded in `docs/qa/TEST_DRIVE.md` #6 (capture → move p95 ≈ 10 ms,
was ≈ 60; engine unaffected by Options; handover works).

**First action next session:** check CI for the 2a commit (command under Toolchain). CI was red
since `eb667e3` on all 3 OS because `tracking-core/tests/fixtures.rs`
`recommended_params_are_the_measured_candidate` didn't include the ADR-0011 2026-11-15
amendment; the 2a commit fixes it (renamed `..._as_amended`). All five crates passed locally.
If CI is still red, failing step names come from `.../actions/runs/<id>/jobs`; job logs need
auth (no `gh`), so reproduce locally per crate (`cargo test -p <crate>`, one at a time).

**Then: slice 2b** (task file "2b"): `docs/qa/BENCHMARK_PROTOCOL.md`, scripted recordings in
`~/ocm-recordings/<scenario>-<date>/` (NOT /tmp; recorder writes on its own thread),
`replay_bench` in `crates/app-core/tests/replay.rs`, synthetic appearance-change corpus in
`tracking-core` tests. Needs ~10 min of recording from the user.

**Open from 2a (not blocking):** `ui` pass ≈ 15/17 ms while Options is open (likely vsync wait
of the immediate viewport; cursor unaffected) — revisit only if the UI feels slow. Minimised
rate and dwell not re-checked on device. Tell the user if relevant: pointer now runs at full
rate while minimised (was ~10 Hz).

**Replay tools** (`crates/app-core/tests/replay.rs`, all `#[ignore]`):
`OCM_REPLAY=<dir> cargo test --release -p app-core --test replay <name> -- --ignored --nocapture`.

**Machine load (user complaint 2026-11-15):** one heavy command at a time, scoped (`-p`,
test filter), say so first; cap replays with `OCM_REPLAY_FRAMES`. A release build is a "heavy"
command — ask first.

**How the user runs the app:** dev build, `export PATH="$HOME/.cargo/bin:$PATH"` then
`cargo run -p open-camera-mouse` (never `--release` for the test drive). Their settings:
`~/Library/Application Support/Open Camera Mouse/settings.toml`.

**Known, not bugs (don't "fix" without the user):** Searching animation ignores Reduce Motion
(Phase 8); font is egui default; F10 not F11 on macOS; re-detection off by default; no
auto-recenter.

**User direction (don't drift):** start from 2018 (ADR-0006) but improve on it; new behaviour
ships as options, off by default (ask before changing a default); monochrome UI, neon green ring
marker, yellow primary buttons, Carbon/Paper, two fixed windows; hotkeys fixed (F10/F12 on
macOS); wants as much as possible checkable by automated tests. Gain per display: decided in
2c with replay numbers (user, 2026-11-15).

**Toolchain:** `export PATH="$HOME/.cargo/bin:$PATH"`. No `gh`; CI status via
`curl -s "https://api.github.com/repos/grga1349/open-camera-mouse/actions/runs?branch=rewrite/rust&per_page=1"`
(job logs need auth; failing step names via `.../actions/runs/<id>/jobs`).

**Blockers:** none. 2b needs ~10 min of recording from the user.
