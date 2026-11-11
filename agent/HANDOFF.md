# Handoff

Keep this short and current. Overwrite (don't append history) each time you hand off —
git history is the log if you need the past. See `agent/WORKFLOW.md` for when to update this.

## As of 2026-11-12 (Phase 3 committed and pushed, CI pending at handoff)

**Branch:** `rewrite/rust`. Phases 0–2 DONE. Phase 3 (task 0005) is complete and pushed;
its only open gate item is a green CI run on that push.

**Next session, in order:**
1. `git status` / `git log -1`. Check CI for the latest push (repo is public, no `gh` here):
   `curl -s https://api.github.com/repos/grga1349/open-camera-mouse/actions/runs?branch=rewrite/rust\&per_page=1`
   then `.../actions/runs/<id>/jobs` for per-job results.
2. CI red → fix (likely platform-specific: float/f64 formatting, test timing on Windows), push,
   re-check. CI green → close 0005: move to `tasks/done/`, add an Outcome with the run URL,
   BACKLOG row → DONE, `docs/ROADMAP.md` Phase 3 → DONE (evidence: task 0005,
   ADR-0009, `docs/research/TRACKING_BASELINE.md`), Phase 4 → NEXT. Ask the user before
   committing that.
3. Start Phase 4 (Tracking improvements) per `docs/ROADMAP.md`: write its task file
   `tasks/active/0006-...` from the roadmap + the findings below. Don't pre-write it before
   step 2.

**Phase 4 inputs** (`docs/research/TRACKING_BASELINE.md` §Findings / §Performance):
- LK fails silently under full occlusion (follows occluder) → residual/confidence gating.
- LK can't recover after the 35/640 guard holds (`fast_40px`) or after edge exits
  (`edge_*`) → re-detection / template-based recovery (NCC recovers in 2–8 frames).
- LK drifts ~6 px on a 0.7 gain step → photometric (gain/bias) compensation.
- NCC silently locks wrong on repeated texture and after occlusion → don't adopt NCC as-is.
- Perf: LK step 4.1 ms @640 on M1, ~98 % pyramid build (full-image Scharr on every level,
  per-pixel reflect-101) vs PROPOSED < 2 ms. NCC 33 ms (naive search).
- Each experiment must be an ablation against the same corpus: `cargo test -p tracking-core
  --release --test baseline -- --ignored --nocapture` (≈5 min; `BASELINE_SCENE=<name>` for one).

**Read first:** `docs/research/TRACKING_BASELINE.md`, ADR-0009, ADR-0006, ADR-0004,
`crates/tracking-core/src/lk.rs`, `tests/support/mod.rs`. Skills: `computer-vision`,
`tracking-validation`, `benchmarking` (Read their SKILL.md; Claude Code doesn't auto-load).

**Toolchain:** `export PATH="$HOME/.cargo/bin:$PATH"` (rustup installed without PATH edits).
`cargo-deny` isn't installed locally (install was blocked); CI runs it.

**Blockers:** none.
