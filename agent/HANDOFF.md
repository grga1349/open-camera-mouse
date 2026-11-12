# Handoff

Keep this short and current. Overwrite (don't append history) each time you hand off —
git history is the log if you need the past. See `agent/WORKFLOW.md` for when to update this.

## As of 2026-11-12 (Phase 5 DONE)

**Branch:** `rewrite/rust`. Phases 0–5 DONE. Task 0007 is in `tasks/done/`. The approved UI is
`docs/design/design-lab.html` v3 (open from disk; deep links like
`#theme=paper&state=searching&tab=general&diag=1`). The egui plan is `docs/design/EGUI_FEASIBILITY.md`.

**Next session, in order:**
1. `git status` / `git log -3` — Phase 5 is committed and pushed (`3578620` + recenter follow-up);
   the working tree should be clean.
2. Phase 6 (desktop app) per `docs/ROADMAP.md`: write its task file from the deliverables/gate.
   Inputs: EGUI_FEASIBILITY §"Phase 6 acceptance criteria" + §"Inputs for Phase 6" (new AppState
   variants for pointer permission and handover, global hotkey adapter ADR, font choice).

**User design decisions (don't drift):** monochrome UI; neon green `#00ff00` only for the thin ring marker,
yellow only for primary buttons; Carbon + Paper themes switchable in Options; two fixed-size windows; no
auto-recenter; hotkeys fixed F11/F12; recenter = white centre circle (pick-circle style) + 5 s countdown 5-4-3-2-1, then the point in the circle
becomes the target; resume from pause goes through recenter; Recenter also ends a pause (R-A3).

**Read first:** `docs/ROADMAP.md` Phase 6, `docs/design/EGUI_FEASIBILITY.md`, BEHAVIOR_SPEC §7,
ADR-0006, ADR-0007, ADR-0011, `docs/architecture/PLATFORM_RISKS.md`. Skills: `egui-development`,
`platform-integration`, `rust-architecture`.

**Toolchain:** `export PATH="$HOME/.cargo/bin:$PATH"`. `cargo-deny` runs in CI only.

**Blockers:** none.
