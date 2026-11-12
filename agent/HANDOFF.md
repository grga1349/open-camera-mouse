# Handoff

Keep this short and current. Overwrite (don't append history) each time you hand off —
git history is the log if you need the past. See `agent/WORKFLOW.md` for when to update this.

## As of 2026-11-12 (Phase 4 active; E0–E4 done, E5/E6 not attempted)

**Branch:** `rewrite/rust`. Phases 0–3 DONE. Phase 4 = task 0006. E0 KEPT. E1 KEPT, default off
(redundant). E2 KEPT, default off (`min_ref_zncc` 0.7). E3 KEPT, default off (`gain_bias`).
**E4 KEPT, default off**: `LkParams::redetect` + `crate::redetect`, ADR-0010 Accepted. Candidate
`support::e4_candidate()`: τ 0.9, δ 0.1, K 16, L2, r 0.25 w on E2+E3 → 89.4 % mean ok (70 runs), 0
wrong-lock / false-track. Results: `docs/research/TRACKING_IMPROVEMENTS.md` §E4–§E6. Check `git log`
to see whether E4 has been committed and pushed.

**Next session, in order:**
1. `git status` / `git log -1`. CI check (no `gh` here):
   `curl -s https://api.github.com/repos/grga1349/open-camera-mouse/actions/runs?branch=rewrite/rust\&per_page=1`.
2. Default-tracker ADR (ADR-0011): recommend E2 (0.7) + E3 + E4 candidate, E1 off. List every
   deviation from 2018 with its ADR-0006 reason (min_eigen ADR-0009, appearance gate, gain/bias,
   re-detection ADR-0010). Decide how it's exposed: change `LkParams::default()` (breaks the
   `lk_fingerprint`/LK18 baseline identity), or add e.g. `LkParams::recommended()` and keep
   Default = B-LK18. Recommendation: the latter, so the ablation baseline stays reproducible.
   Note for app-core (Phase 5): `Lost(Searching)` → `Tracking` re-acquisition vs 2018 handover.
3. Final latency for the recommended tracker (tracked step + searching step benches), then close
   task 0006 against its acceptance criteria. Update ROADMAP and move the task file to done.

**Watch:** `rotate_15` 320 ZNCC margin (0.74 vs 0.7). E3's `rotate_15` 1280 Δp95 +0.070 (closest gate
margin). E4 spatial prior r is only bracketed to [0.15, 0.32) by the corpus. 320 re-acquisition offset
1–1.5 px (follow-up: LK refine against the selection pyramid).

**Read first:** task 0006, `TRACKING_IMPROVEMENTS.md`, ADR-0010, ADR-0009, ADR-0006,
`crates/tracking-core/src/{lk,redetect}.rs`. Skills: `rust-architecture` (ADR), `benchmarking`.

**Toolchain:** `export PATH="$HOME/.cargo/bin:$PATH"` (also needed for `scripts/check-core-deps.sh`).
`cargo-deny` isn't installed locally; CI runs it. `ablation_tables` ~2.5 min, `lk_fingerprint` ~1 min.

**Blockers:** none.
