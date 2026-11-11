# Seed consumption report

Produced 2026-11-11 at the end of the Phase 0 session that extracted this seed (formerly
`dump/`, now archived here unchanged) into real repository artifacts. Per the seed's own
`ENTRY.md` and `09-operator-playbook.md`, this directory stays archived as historical input —
it is not deleted and not live policy.

## Mapping: seed file → where it landed

| Seed file | Extracted into |
|---|---|
| `ENTRY.md` | This report; reading-map content folded into `AGENTS.md`'s repo map |
| `initial-promt.md` | Executed this session; see `agent/HANDOFF.md` for outcome |
| `01-product-and-context.md` | `docs/product/PRODUCT.md` |
| `02-roadmap-nine-phases.md` | Referenced directly (not duplicated) from `AGENTS.md`, `tasks/active/0001`, `tasks/active/0003`; still the source of truth for phase staging |
| `03-research-and-evidence.md` | `docs/research/RE_EVIDENCE.md` (synthesized with actual `legacy/` source reading, not just restated), `docs/architecture/decisions/0004-proposed-tracking-baseline-candidates.md` |
| `04-repository-migration.md` | Executed as `docs/architecture/decisions/0001-rust-rewrite-on-branch-legacy-archived.md`; the proposed eventual layout table was revised, not copied verbatim — see that ADR and `AGENTS.md`'s repo map for what was actually built in Phase 0 vs. deferred to later phases |
| `05-architecture-and-testing.md` | `docs/architecture/decisions/0003-proposed-crate-boundaries.md` (crate list superseded by `10-...md`'s revision, noted there); testing-layer content folded into `.agents/skills/tracking-validation`, `.agents/skills/rust-testing`, `.agents/skills/benchmarking` |
| `06-ai-workflow.md` | `AGENTS.md`, `agent/WORKFLOW.md`, `tasks/BACKLOG.md` + lifecycle, `.agents/skills/agent-handoff`; skill list superseded by `10-...md`'s table (used instead) |
| `07-design-direction.md` | `docs/design/DESIGN_PLAN.md` |
| `08-decisions-and-open-questions.md` | Decisions marked "agreed intent" became ADRs 0001/0002; "proposed, not yet ratified" items became explicitly-Proposed-status ADRs 0003/0004; open questions distributed into the relevant ADR/doc "consequences"/"unresolved" sections rather than copied as a flat list — see each ADR and `docs/research/RE_EVIDENCE.md`'s "unresolved questions" section |
| `09-operator-playbook.md` | Followed directly this session (branch-first, no blind `dump/` deletion, handoff discipline); "before major changes" guidance folded into `agent/WORKFLOW.md` |
| `10-rust-skills-architecture-testing-research.md` | Its "executive decisions" (no CLI, 4-crate+app layout, deterministic-test-first) are the versions actually recorded in ADR-0003 and `AGENTS.md`'s principles, explicitly overriding `05`'s earlier 3-crate suggestion; its skill table is the one implemented in `.agents/skills/` (10 skills, not `06`'s 8); its test-design sections (A-G) are folded into the relevant skill files' "deliverables and gates" |

## Explicitly deferred, not omitted

- Cargo workspace / crate skeleton, egui app, `design-lab.html`, tracking algorithm code —
  all Phase 2+ by the seed's own staging; Phase 0 deliberately did not create them.
- AI-tooling discovery verification (symlink/skill-loading behavior in Claude Code and Codex)
  could not be done by this same session against its own newly-created files — tracked as
  `tasks/active/0002-verify-ai-tooling-discovery.md`, not silently skipped.
- Codex's exact project-local skill-directory convention was not confirmed against current
  official docs this session — left as an open item in task 0002 rather than guessed.

## What changed from the seed's proposals (and why)

- `CLAUDE.md` is a plain pointer file, not a symlink to `AGENTS.md` — ADR-0002, driven by the
  Windows git-symlink failure mode the seed itself flagged as a risk to test for.
- Crate list is `tracking-core`/`pointer-core`/`app-core`/`platform` (seed `10`'s revision),
  not `05`'s original three — `10` explicitly supersedes `05` on this point.
- Tracking baseline is recorded as two candidates to measure (LK and NCC), not a predetermined
  PyrLK-only plan — because `legacy/`'s actual source turned out to already be a working NCC
  implementation, not a blank slate, which the seed's authors could not have confirmed without
  reading `legacy/internal/tracking/tracking.go` directly (done this session).

## Verification status of this report itself

This mapping was written by the same session that did the extraction, not by an independent
reviewer. Treat it as a first-pass index, not a guarantee every nuance survived — if a future
session finds a seed detail that didn't make it into a real doc/task/ADR, add it rather than
assuming this report is exhaustive.
