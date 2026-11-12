# Handoff

Keep this short and current. Overwrite (don't append history) each time you hand off —
git history is the log if you need the past. See `agent/WORKFLOW.md` for when to update this.

## As of 2026-11-12 (Phase 4 active; E0–E3 done)

**Branch:** `rewrite/rust`. Phases 0–3 DONE. Phase 4 = task 0006. E0 KEPT. E1 KEPT, default off
(now redundant). E2 KEPT, default off (`min_ref_zncc`, candidate 0.7). **E3 KEPT, default off**
(`19a2f4a`, CI 43128269741 green): `LkParams::gain_bias`, a per-level mean/σ-matched photometric map that stays fixed while the level
iterates. Results: `docs/research/TRACKING_IMPROVEMENTS.md` §E3. E2+E3 at zncc ≥ 0.7 has 0 silent
errors, mean ok 75.4 % (= LK18), worst gate Δp95 +0.070 (`rotate_15` 1280). **Proposed default:
E2 + E3, E1 off.** That goes into the default-tracker ADR after E4.

**Next session, in order:**
1. `git status` / `git log -1`. CI check (no `gh` here):
   `curl -s https://api.github.com/repos/grga1349/open-camera-mouse/actions/runs?branch=rewrite/rust\&per_page=1`.
2. E4, loss recovery / re-detection. It needs an ADR first: it changes the 2018 invalidate → re-select
   policy (ADR-0006 reason (a) defect). Reuse E2's selection reference window. On `Held`/`Lost`,
   do a coarse-to-fine ZNCC search, then resume only when the match is confident and unambiguous
   (peak-to-second-peak). It must not inherit NCC's `repeated_texture` wrong lock (finding 7). Targets
   `fast_40px` and `edge_*` (findings 2, 4). Measure on top of E2+E3. Probably needs a distractor
   fixture.
3. Keep `LkParams::default()` bit-identical (`lk_fingerprint` diffed against a worktree at HEAD
   with its own `CARGO_TARGET_DIR`).

**Watch:** the `rotate_15` 320 ZNCC margin (0.74 vs 0.7) is unchanged by E3. Real head roll is a
Phase 6 risk. E3's `rotate_15` Δp95 +0.070 at 1280 is inside the gate but is the closest margin.

**Read first:** task 0006, `TRACKING_IMPROVEMENTS.md`, `TRACKING_BASELINE.md` §Findings,
ADR-0009, ADR-0006, `crates/tracking-core/src/lk.rs`. Skills: `computer-vision`,
`tracking-validation`, `benchmarking`, `rust-architecture` (ADR).

**Toolchain:** `export PATH="$HOME/.cargo/bin:$PATH"` (also needed for `scripts/check-core-deps.sh`).
`cargo-deny` isn't installed locally; CI runs it. `ablation_tables` ~2 min, `lk_fingerprint` ~1 min.

**Blockers:** none.
