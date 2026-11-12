# Handoff

Keep this short and current. Overwrite (don't append history) each time you hand off —
git history is the log if you need the past. See `agent/WORKFLOW.md` for when to update this.

## As of 2026-11-12 (Phase 4 active; E0, E1, E2 done)

**Branch:** `rewrite/rust`. Phases 0–3 DONE. Phase 4 = task 0006. E0 KEPT (`f68f042`). E1 KEPT,
default off (`cac424f`, CI green). **E2 KEPT, default off**: `LkParams::min_ref_zncc` (ZNCC vs
the selection-frame window), `LossReason::AppearanceChanged`, `Diagnostics::score`. Candidate
threshold 0.7. Results: `docs/research/TRACKING_IMPROVEMENTS.md` §E2. E2 may still be
uncommitted; check `git status`.

**Next session, in order:**
1. `git status` / `git log -1`. CI check (no `gh` here):
   `curl -s https://api.github.com/repos/grga1349/open-camera-mouse/actions/runs?branch=rewrite/rust\&per_page=1`.
2. E3 gain/bias-compensated LK. Targets finding 5: at the `brightness_step` gain step, plain LK's
   raw estimate exceeds the 35 px guard, and it then tracks 6.4 px off (E2 now reports that as lost). Add
   `variants()` entries alone and with E2 zncc ≥ 0.7 (± E1 fb ≤ 0.5). E2 dominates E1. Check
   whether E3 rescues E1's `contrast_drop` loss, or whether E1 can be dropped.
3. Keep `LkParams::default()` bit-identical. Prove it with `lk_fingerprint` diffed against a
   pre-change run (a worktree at HEAD with its own `CARGO_TARGET_DIR` works).

**Watch:** E2's `rotate_15` margin at 320 is thin (min ZNCC 0.74 vs 0.7). Re-check it under any E3 change.
E4 should reuse E2's reference window.

**Read first:** task 0006, `TRACKING_IMPROVEMENTS.md`, `TRACKING_BASELINE.md` §Findings,
ADR-0009, ADR-0006, `crates/tracking-core/src/lk.rs`. Skills: `computer-vision`,
`tracking-validation`, `benchmarking`.

**Toolchain:** `export PATH="$HOME/.cargo/bin:$PATH"` (also needed for `scripts/check-core-deps.sh`).
`cargo-deny` isn't installed locally; CI runs it. `ablation_tables` now takes ~3 min, `lk_fingerprint` ~1 min.

**Blockers:** none.
