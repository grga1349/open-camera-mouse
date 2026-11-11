# Handoff

Keep this short and current. Overwrite (don't append history) each time you hand off —
git history is the log if you need the past. See `agent/WORKFLOW.md` for when to update this.

## As of 2026-11-11 (task 0003 session)

**Branch/commit:** `rewrite/rust`, task 0003 + ADR-0006 committed and pushed (`9fcde6b`).

**Task:** 0003 (Phase 1 behavior spec) — REVIEW. Deliverable written; needs an independent
read-through before DONE (criteria at the bottom of `tasks/active/0003-...md`).

**Changed files (in `9fcde6b`):** new `docs/research/BEHAVIOR_SPEC.md`, new
`docs/research/tools/dump_il.py`, edited `docs/research/RE_EVIDENCE.md` (smoothing-polarity
correction + superseded note), `tasks/BACKLOG.md` (0003 → REVIEW, 0004 added as TRIAGE),
`tasks/active/0003-...md` (outcome section), `agent/*`.

**Key new facts (from decoding `rea-evidence.json` CIL, not the prose research):** 2018 loss
policy is a 4-way table on `Run_54321_OnLossOfTrackPoint` × an edge predicate whose x terms
look buggy; LK flags = `PYR_A_READY` constant; first post-pick frame unchecked; offset drifts
±1 px/tick while clamped at screen limits; full `DefaultConfig`; `m.a 0x060001fd` is
mouse-inactivity auto-start, not dwell. Legacy defects L-1…L-12 (spec §3.7), incl. smoothing
slider 0 freezes cursor, dwell repeat-clicks, silent camera-loss.

**Verified:** `dump_il.py` reproduces cited anchors; all 4 originally-cited evidence IDs exist.
`@AGENTS.md` import confirmed expanding in a fresh Claude Code session (closes ADR-0005 risk).

**Product direction (user, 2026-11-11):** favor 2018 behavior where practical, recorded as
ADR-0006 (accepted; per-item table). Absolute pointer + handover + 2018 dwell defaults are
adopted. Hotkeys (Q-R8) are still open: 2018 F9/F5 vs. the earlier fixed-F11/F12 decision.

**Blockers:** none for review.

**First next action:** independent review of `BEHAVIOR_SPEC.md` (sample `dump_il.py`
citations + legacy line refs), then mark 0003 DONE and expand task 0004 into a full task file.
