# Initial prompt — Open Camera Mouse Rust rewrite, phase 0 kickoff

You are the lead software architect and AI workflow engineer helping migrate an existing Open Camera Mouse application into a sustainable, AI-first Rust desktop project. Your first mission is **safe investigation and foundation planning**, not implementation of the new Rust application.

## First: load context

Read `dump/ENTRY.md`, the linked seed documents, and especially `dump/10-rust-skills-architecture-testing-research.md` for the updated no-CLI architecture and test decisions; this `dump/` directory is a **staging archive**. It is NOT the final repository docs, rules, skills or task system. Extract/reconcile its contents into genuine repo artifacts only as justified and validated. Inspect `../camera-mouse-reverse/` for the actual reverse engineering evidence (particularly `research/TRACKING_SPEC.md`), and the existing code, tests, Git state and existing `AGENTS.md`/`CLAUDE.md`/agent configs. The seed is background; real files and evidence take precedence.

## Phase 0 — safe foundation setup

1. Inventory repository, branch, uncommitted changes, tracked/untracked files, licenses and existing AI setup. Record exact findings.
2. Create the rewrite branch (`rewrite/rust`) **only if safe**. Avoid switching in a way that loses work. Stop destructive operations until a safe preservation path is known.
3. Propose a migration of old source into `legacy/` while retaining Git history and preserving relevant old instructions as *archived references*, not active root instructions. Do not blindly `git mv . legacy/`; carefully classify paths, ignore generated content and maintain relevant root metadata. Execute only a validated, reversible migration.
4. Research current official documentation for Cargo workspaces, egui/eframe, native camera and cursor APIs, Codex instructions/skills/subagents and Claude Code instructions/skills/subagents/hooks. Cite URLs, date/version, constraints. Do not assume symlinks work with every client; test.
5. Draft a lean self-contained repository layout, instruction policy, project-local skill catalog, subagent roles, task system, handoff strategy, ADR policy and basic quality gates. Prefer source-of-truth files over duplicated instructions.
6. Compare source implementation with actual Camera Mouse 2018 reverse engineering docs, distinguishing confirmed binary observations from proposed hypotheses and missing measurements.
7. Write an initial project architecture proposal and a plan for an HTML design lab, without implementing tracking or the application.
8. If implementing foundation artifacts in this stage, validate links, instructions discovery, skill discovery and Git state in BOTH installed clients where available. Do not silently create global machine-level settings.
9. Persist concise handoff and task state in the **actual chosen repo locations**; never equate chat compaction with durable storage. Preserve `dump/` unchanged as staging context until verified extraction.

## Execution discipline

Use agents for parallel, independent research/review only if actually supported. The lead agent owns design choices, task status, integration, and avoiding simultaneous edits to the same files. Prefer reversible changes and explicit tests. Do not copy an invented template in place of research. If a dependency or capability is uncertain, leave it an explicit open question. Do not install unnecessary community skills or add OpenCV to the new engine.

## Deliverable from this first session

Produce: current-state inventory; preservation/migration outcome; source-backed tool research; proposed final layout; phase 0 task list and acceptance checks; known RE findings and uncertainties; preliminary architecture/design outline; verification results; explicit next task and handoff. **Do not start the Rust tracking rewrite yet.** Later phases and gates are in `dump/02-roadmap-nine-phases.md`.
