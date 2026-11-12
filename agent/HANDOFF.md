# Handoff

Keep this short and current. Overwrite (don't append history) each time you hand off —
git history is the log if you need the past. See `agent/WORKFLOW.md` for when to update this.

## As of 2026-11-12 (Phase 4 DONE)

**Branch:** `rewrite/rust`. Phases 0–4 DONE. Task 0006 is in `tasks/done/`. The recommended tracker is
`LkParams::recommended()` (ADR-0011): B-LK18 + E2 (ZNCC ≥ 0.7) + E3 + E4 re-detection (ADR-0010), E1 off.
`Default` stays B-LK18 for the ablation baseline. 89.4 % mean ok on 70 runs, 0 silent errors, tracked
step ~0.3 ms and searching step ~0.58 ms at 640.

**Next session, in order:**
1. `git status` / `git log -1`. CI check for the Phase 4 close-out commit (no `gh` here):
   `curl -s https://api.github.com/repos/grga1349/open-camera-mouse/actions/runs?branch=rewrite/rust\&per_page=1`.
   Record the green run id in `tasks/BACKLOG.md` (the 0006 row says "CI pending").
2. Phase 5 (design system) per `docs/ROADMAP.md`: write its task file from the deliverables/gate.
   UI states must include tracking / held / lost / **searching (re-acquiring)** / paused / no camera
   / permission, plus a "can't auto-recover this selection" hint (`redetect_armed() == false`).

**Watch (carry into Phase 6):** re-measure all tracker thresholds on recordings. Re-acquisition →
control handover is undecided. 320 re-acquisition offset follow-up (LK refine vs the selection pyramid).

**Read first:** `docs/ROADMAP.md` Phase 5, `docs/design/`, BEHAVIOR_SPEC §7, ADR-0006, ADR-0011.
Skills: `egui-development`.

**Toolchain:** `export PATH="$HOME/.cargo/bin:$PATH"`. `cargo-deny` runs in CI only.

**Blockers:** none.
