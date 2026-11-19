# ADR-0022: Post-rewrite cleanup — remove `legacy/` and the rewrite tooling

Status: Accepted (user decisions 2026-11-19)
Date: 2026-11-19
Supersedes: the "keep `legacy/`" and "keep the seed archive" parts of ADR-0001 and the
`AGENTS.md` "Never" rule that protected them. Amends ADR-0005 (agent workflow).

## Context

The Rust rewrite (phases 0–7) is complete and is about to replace the Go app on `main` as
v0.4.0. The repository still carried everything used to *do* the rewrite: the archived Go app
under `legacy/` (135 MB), the original planning seed `docs/archive/`, the Camera Mouse 2018
reverse-engineering and comparison research `docs/research/` (incl. a tool and a research
prompt), the phase task files `tasks/`, session state `agent/`, an `agent-handoff` skill, a
recording helper script, per-phase QA logs, and design-process notes. Future work is ordinary
product development, with tasks coming in as prompts, not phases.

## Options considered

1. Keep everything (status quo): accurate history in the tree, but most files describe a
   finished process; new contributors and agents read stale instructions.
2. Archive into one folder: still in every checkout and every agent's search results.
3. **Remove, condense what stays true into a few project docs, rely on git history.**

## Decision

Option 3 (user: "all that will not be needed for further development").

- **Removed:** `legacy/`, `docs/archive/`, `docs/research/` (all), `docs/qa/`,
  `docs/design/DESIGN_PLAN.md` and `EGUI_FEASIBILITY.md`, `docs/product/`,
  `docs/architecture/DEPENDENCIES.md` and `PLATFORM_RISKS.md`, `tasks/`, `agent/`,
  `.agents/skills/agent-handoff/`, `scripts/record-scenario.sh`, the legacy `.gitignore`
  patterns, the debug-only `OCM_DEV_*` environment variables. `OCM_PERF` / `OCM_RECORD` stay.
- **Condensed into:** `README.md` (install, permissions, known limitations, build, package,
  release), `docs/PRODUCT.md` (product, defaults, UI design + tokens),
  `docs/BEHAVIOR.md` (the R-* requirements, fixture corpus, metrics),
  `docs/ARCHITECTURE.md` (crates, runtime, adapters, platform risks P1–P20, dependencies,
  packaging), `docs/TESTING.md` (gates, test layout, manual QA, diagnostics), `docs/ROADMAP.md`
  (open items). `docs/design-lab.html` (approved design) and all ADRs stay.
- **Workflow:** `AGENTS.md` describes a prompt-driven workflow; the project skills in
  `.agents/skills/` stay, updated to the new doc paths.

## Consequences

- Where to find removed material: the Go app is `main` before the merge and the `v0.3.0`
  tag; everything else is at commit `da37d0e` on `rewrite/rust`
  (`git show da37d0e:docs/research/BEHAVIOR_SPEC.md`, …). Older ADRs keep their links to
  removed files as historical references; read them at that commit.
- Code comments cite `BEHAVIOR` R-* IDs and platform risks P1–P20, which live on in
  `docs/BEHAVIOR.md` and `docs/ARCHITECTURE.md`.
- Deferred Phase 7 work and Phase 8 open items live in `docs/ROADMAP.md`.
