# Handoff

Keep this short and current. Overwrite (don't append history) each time you hand off —
git history is the log if you need the past. See `agent/WORKFLOW.md` for when to update this.

## As of 2026-11-12 (Phase 4 active; E0 done)

**Branch:** `rewrite/rust`. Phases 0–3 DONE (Phase 3 CI run 43119858155 green). Phase 4 =
task 0006. E0 (window-only gradients, pyr_down fast paths) is KEPT: `lk_step` at 640 went
4.13 → 0.30 ms with bit-identical outputs. Committed `f68f042` + `ad25a34`, pushed, CI run
43121280464 green on 3 OS + cargo-deny.

**Next session, in order:**
1. `git status` / `git log -1` (expect a clean tree at or after `ad25a34`). To check CI later
   (no `gh` here): `curl -s https://api.github.com/repos/grga1349/open-camera-mouse/actions/runs?branch=rewrite/rust\&per_page=1`.
2. E1 forward-backward validation, per the task file. First add an ablation mode to
   `tests/baseline.rs` (B-LK18 vs variant, same corpus, same metrics). Keep
   `LkParams::default()` bit-identical: use `lk_fingerprint` to prove it.
3. Record each experiment's keep/reject + numbers in `docs/research/TRACKING_IMPROVEMENTS.md`.

**Read first:** task 0006, `docs/research/TRACKING_IMPROVEMENTS.md`, `TRACKING_BASELINE.md`
§Findings, ADR-0009, ADR-0006, `crates/tracking-core/src/lk.rs`. Skills: `computer-vision`,
`tracking-validation`, `benchmarking` (Read their SKILL.md).

**Toolchain:** `export PATH="$HOME/.cargo/bin:$PATH"`. `cargo-deny` isn't installed locally;
CI runs it. The full baseline takes ~5 min (NCC is 33 ms/frame); `lk_fingerprint` takes ~1 min.

**Blockers:** none.
