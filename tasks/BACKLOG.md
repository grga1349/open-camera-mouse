# Task backlog

Single source of truth for task status — don't duplicate status elsewhere. Lifecycle:
`TRIAGE → READY → ACTIVE → REVIEW → DONE` (or `BLOCKED`). One coordinator updates this file
and moves task files between `active/`, `done/`, `blocked/` as status changes; a checklist is
not completion evidence, passing verification commands are.

## Active

| ID | Title | Status |
|---|---|---|
| — | Phase 7 — multiplatform / release | NEXT — task file not written yet; the coordinator writes it from `docs/ROADMAP.md` |

## Done

| ID | Title | Status |
|---|---|---|
| 0011 | Phase 6d — calibrated pointer behaviour (ADR-0020) | DONE 2026-11-16 — steady zone 0.2 cam px + drift cap (#13), default gain 20, #12 closed (no jumps), #14 Main monitor = OS primary; user: ready for Phase 7; steps 3–4 dropped as obsolete; #7 → Phase 7 |
| 0010 | Phase 6c — tuning round (ADR-0019) | DONE — #8 real gain 1–60, #9 Screen tab, #10 Text size removed + defaults gain 22 / smoothing 0.20, #11 automatic steady zone (0.5 cam px); closed into 6d (ADR-0020) |
| 0009 | Phase 6b — test drive (ADR-0013) | DONE — issues #1–#6 (`docs/qa/TEST_DRIVE.md`); slices 2a–2d, ADR-0014…0018, last code `df4c670`; closed into 6c (ADR-0019) |
| 0008 | Phase 6 — desktop app | DONE — D0–D7 + D6b, final code `7e9a2bb` CI 43746097717 green on 3 OS + cargo-deny; manual QA rows 1–18 pass 2026-11-15 (`docs/qa/MANUAL_QA.md`); ADR-0012 |
| 0007 | Phase 5 — design system | DONE — lab v3 approved 2026-11-12 (Carbon/Paper, neon-green thin-ring marker, yellow primary buttons); `docs/design/EGUI_FEASIBILITY.md` |
| 0006 | Phase 4 — tracking improvements | DONE — E0–E4 (`f68f042`…`0074611`), E4 CI 43131916361 green, close-out `66768c9` CI 43132339284 green on 3 OS + cargo-deny; ADR-0010, -0011, `docs/research/TRACKING_IMPROVEMENTS.md` |
| 0005 | Phase 3 — tracking baseline | DONE — `4038fce`, CI run 43119858155 green on 3 OS + cargo-deny; ADR-0009, `docs/research/TRACKING_BASELINE.md` |
| 0004 | Phase 2 — Rust architecture | DONE — `ff0d1ab`, CI run 43116199528 green on macOS/Windows/Linux + cargo-deny; ADR-0007, -0008 |
| 0001 | Phase 0 — repository foundation | DONE — commit `a1e4f4e`, pushed to `origin/rewrite/rust` |
| 0003 | Phase 1 — research and behavior specification | DONE — `docs/research/BEHAVIOR_SPEC.md`, ADR-0006 |
| 0002 | Verify AI-tooling instruction/skill discovery in Claude Code and Codex | DONE — ADR-0005; `@AGENTS.md` import confirmed expanding in a fresh session (2026-11-11) |

## Blocked

(none yet)
