# ADR-0001: Rust rewrite on `rewrite/rust`, old app archived under `legacy/`

Status: Accepted
Date: 2026-11-11

## Context

The working Go/Wails/GoCV app (v0.3.0, shipping on macOS and Windows) is being rewritten from
scratch in Rust to drop the OpenCV/GoCV native dependency and build a maintainable,
AI-assisted, cross-platform engineering workflow. This needed a migration strategy that does
not lose the working app, its history, or its documentation.

## Options considered

- **Rewrite in place on `main`**, deleting old code as it's replaced. Rejected: destroys the
  working regression/reference baseline and risks losing comparison ground truth mid-rewrite.
- **New repository.** Rejected: loses git history, issue/PR continuity, and the point of
  having `legacy/` as an in-tree comparator for Phase 3 benchmarks.
- **Feature branch (`rewrite/rust`) + `legacy/` archive directory on that branch**, `main`
  untouched. Chosen.

## Decision

Created `rewrite/rust` from `main` (which was clean except for the untracked planning seed).
Moved every old-app file (`main.go`, `app.go`, `internal/`, `frontend/`, `build/`, `ci/`,
`docs/` → `legacy/docs/`, `Makefile`, `wails.json`, `go.mod`/`go.sum`, `VERSION`,
`.github/workflows/release.yml`, the old root `README.md`, and the old root `CLAUDE.md` → now
`legacy/ai-config/CLAUDE.md`) into `legacy/` using `git mv`, preserving rename history.
`main` is untouched and still builds the shipping Go app; `rewrite/rust` is where all new work
happens until it's ready to become the new `main`.

## Consequences

- `legacy/`'s GitHub Actions release workflow is no longer at `.github/workflows/` and will
  not trigger — intentional. Restoring it (to cut another Go release from `main`) just means
  working on `main`, not `rewrite/rust`.
- `legacy/ai-config/CLAUDE.md` is archived, not active; the new root `AGENTS.md`/`CLAUDE.md`
  are canonical on this branch (see ADR-0002).
- Phase 3 tracking work can diff against `legacy/internal/tracking/tracking.go` directly as a
  real, working comparator (see `docs/research/RE_EVIDENCE.md`).
- No tracking engine, Cargo workspace, or crate code was created by this ADR — that's Phase 2.
