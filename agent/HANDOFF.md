# Handoff

Keep this short and current. Overwrite (don't append history) each time you hand off —
git history is the log if you need the past. See `agent/WORKFLOW.md` for when to update this.

## As of 2026-11-11 (task 0004 in progress)

**Branch:** `rewrite/rust`. Phase 2 work is complete locally and **uncommitted**.

**Next:** with the user's OK, commit and push so `.github/workflows/ci.yml` runs. The gate
needs green macOS/Windows/Linux test jobs and the `cargo deny` job. Then close 0004 (move to
`tasks/done/`, update BACKLOG, ROADMAP Phase 2 → DONE, Phase 3 → NEXT).

**Read first:** `tasks/active/0004-phase-2-rust-architecture.md` (Progress), ADR-0007,
ADR-0008, `docs/architecture/DEPENDENCIES.md`, `PLATFORM_RISKS.md`.

**Toolchain:** `export PATH="$HOME/.cargo/bin:$PATH"` (rustup installed without PATH edits).

**Blockers:** none. `cargo deny` not runnable locally (install was denied), so CI is its
first run.
