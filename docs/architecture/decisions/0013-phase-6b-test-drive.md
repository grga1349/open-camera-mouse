# ADR-0013: Phase 6b — test drive before multiplatform release

Status: Accepted (user decision)
Date: 2026-11-15
Amends: `docs/ROADMAP.md` (adds a phase between 6 and 7; no renumbering)

## Context

Phase 6 closed with a manual QA checklist (`docs/qa/MANUAL_QA.md`, rows 1–18 pass), which
checks each workflow once. It doesn't show what comes up when someone actually uses the app for
a while: comfort of the default speeds, dwell timing, tiring head motion, small UI
annoyances, rare failures. Phase 7 (packaging, signing, other OSes) multiplies the cost of
changing behaviour later, so the user wants to drive the app first and fix what comes up.

## Options considered

1. Go straight to Phase 7 and fold issues found later into Phase 8 hardening. Packaging and
   cross-OS validation would then run against behaviour that may still change.
2. Insert a short user-driven phase on the dev Mac before Phase 7. Behaviour settles first.
3. Renumber the phases (new Phase 7, old 7–8 become 8–9). Breaks every existing reference to
   phase numbers (tasks, ADRs, memory); AGENTS.md forbids silent renumbering.

## Decision

Option 2, named **Phase 6b — Test drive**, so Phases 7 and 8 keep their numbers. The user
uses the app on the dev Mac; each issue they report is logged, triaged and either fixed
(with a regression test where it can be tested headless) or explicitly deferred to a named
phase. The phase ends when the user says the app is ready for Phase 7.

## Consequences

- The roadmap has ten rows; "nine phases" in older docs still means 0–8, with 6b as an
  interlude.
- Phase 6b is not a place for new features: new behaviour beyond fixing what was found goes
  through the normal rules (option off by default, ADR for new crates).
- Deferred carry-overs (MANUAL_QA rows 19–21, `OCM_RECORD` threshold re-measurement) stay
  where Phase 6 put them unless an issue found in 6b needs them earlier.
