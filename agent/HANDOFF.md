# Handoff

Keep this short and current. Overwrite (don't append history) each time you hand off —
git history is the log if you need the past. See `agent/WORKFLOW.md` for when to update this.

## As of 2026-11-12 (Phase 4 active; E0, E1 done)

**Branch:** `rewrite/rust`. Phases 0–3 DONE. Phase 4 = task 0006. E0 KEPT (committed
`f68f042`, CI green). E1 (forward-backward validation) KEPT as a component, **default off**.
Its enable/threshold decision waits for E3. Results: `docs/research/TRACKING_IMPROVEMENTS.md` §E1.
E1 committed `cac424f`, pushed, CI run 43124247853 green.

**Next session, in order:**
1. `git status` / `git log -1` (expect a clean tree at or after `cac424f`). To check CI (no `gh` here):
   `curl -s https://api.github.com/repos/grga1349/open-camera-mouse/actions/runs?branch=rewrite/rust\&per_page=1`.
2. E2 photometric residual / confidence gating (targets occlusion false-track, which E1 can't
   catch: the occluder moves rigidly). Add `variants()` entries in `tests/baseline.rs`, alone and
   with E1 fb ≤ 0.5. Run `ablation_tables` (~1.5 min).
3. Keep `LkParams::default()` bit-identical. Prove it with `lk_fingerprint` (diff against a
   pre-change run on the same toolchain).

**Read first:** task 0006, `TRACKING_IMPROVEMENTS.md`, `TRACKING_BASELINE.md` §Findings,
ADR-0009, ADR-0006, `crates/tracking-core/src/lk.rs`. Skills: `computer-vision`,
`tracking-validation`, `benchmarking`.

**Toolchain:** `export PATH="$HOME/.cargo/bin:$PATH"`. `cargo-deny` isn't installed locally;
CI runs it. `baseline_tables` takes ~5 min, `ablation_tables` ~1.5 min, `lk_fingerprint` ~1 min.

**Blockers:** none.
