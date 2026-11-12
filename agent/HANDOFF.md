# Handoff

Keep this short and current. Overwrite (don't append history) each time you hand off —
git history is the log if you need the past. See `agent/WORKFLOW.md` for when to update this.

## As of 2026-11-12 (Phase 6 ACTIVE, no code yet)

**Branch:** `rewrite/rust`. Phases 0–5 DONE. Phase 6 task file written:
`tasks/active/0008-phase-6-desktop-app.md`, slices D0–D7. No slice started. Code is unchanged
since Phase 4 (`66768c9`, CI green). This session was docs/design only.

**Done this session:**
- Task 0008 written from ROADMAP Phase 6 + EGUI_FEASIBILITY (slices, decisions table, gate, verification).
- User decision: re-detection (ADR-0010) is a user option **"Find the target again"**
  (Options → Camera, setting `loss_recovery`), **off by default**. ADR-0011 amended:
  `LkParams::recommended()` is unchanged; app-core sets `redetect = None` when the option is off.
  Off = 2018 (loss → mouse has control → re-select or recenter). On = `Lost(Searching)` →
  `Tracking` gives control back only within the R-P4 jump bound. Recorded in BEHAVIOR_SPEC R-P4,
  ADR-0006 loss row, ADR-0010, DESIGN_PLAN, EGUI_FEASIBILITY, task 0007 Decision 10.
- Design lab: the switch on the Camera tab (synced with the Searching/Lost states); "Keep both speeds the
  same" uses the label colour; switches align with the label line, not the help text.

**Next session, in order:**
1. `git status` / `git log -3`. The tree should be clean.
2. Start **D0**: the full `app-core` controller (pure, injected clock `now_us`, no I/O). Read the D0 bullet
   and the "Decisions owed" table in task 0008 first. D0 also decides how the pointer-permission and
   mouse-handover states fit R-A1 (variant vs sub-state) and writes that into BEHAVIOR_SPEC R-A1.

**User direction (don't drift):** start from 2018 (ADR-0006) but improve on it, taking the best of
2018 and legacy. Ship proven improvements as options, off by default. Design: monochrome; neon
green `#00ff00` only for the thin ring marker; yellow only for primary buttons; Carbon + Paper themes;
two fixed-size windows; no auto-recenter; hotkeys fixed F11/F12; recenter = white centre circle + 5 s
countdown badge; resume from pause goes through recenter; Recenter ends a pause.

**Still open (defaults in task 0008 apply if the user doesn't decide):** grayscale vs colour preview
(default grayscale), font (default egui fonts). Both are needed only by D6.

**Read first:** task 0008, BEHAVIOR_SPEC §7, ADR-0006, -0007, -0008, -0011 (incl. amendment),
`docs/design/EGUI_FEASIBILITY.md`, `docs/architecture/PLATFORM_RISKS.md`. Skills: `rust-architecture`,
`rust-testing` (D0/D1), then `platform-integration`, `egui-development`.

**Toolchain:** `export PATH="$HOME/.cargo/bin:$PATH"`. `cargo-deny` runs in CI only.

**Blockers:** none.
