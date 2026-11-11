# Handoff

Keep this short and current. Overwrite (don't append history) each time you hand off —
git history is the log if you need the past. See `agent/WORKFLOW.md` for when to update this.

## As of 2026-11-11 (task 0003 wrap-up)

**Branch:** `rewrite/rust`. Phases 0 and 1 are DONE (`docs/ROADMAP.md`).

**Next: Phase 2 — Rust architecture.** Read `docs/ROADMAP.md` §Phase 2 (deliverables + gate),
then write its task file in `tasks/active/` (next ID 0004) and add it to `tasks/BACKLOG.md`.
Inputs: `docs/research/BEHAVIOR_SPEC.md` §7/§9, ADR-0003 (crates, ratify or revise),
ADR-0004 (tracking candidates), ADR-0006 (favor 2018 where practical; absolute pointer;
hotkeys fixed F11/F12). Phase 2 must decide Q-R5 (mirroring) and Q-R6 (camera
resolution/FPS). Research current crate versions from live sources, not memory.

**Last session did:** Phase 1 spec (`BEHAVIOR_SPEC.md`, re-verified 2018 claims against
CIL in `../camera-mouse-reverse/research/rea-evidence.json`; reproduce with
`docs/research/tools/dump_il.py`), ADR-0006, new live `docs/ROADMAP.md`, `AGENTS.md`
pointed at it.

**Blockers:** none.
