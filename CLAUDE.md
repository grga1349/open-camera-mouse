# Read AGENTS.md first

This file is intentionally not the canonical instructions and is **not a symlink** — a real
symlink here breaks on a naive Windows `git clone` without Developer Mode / `core.symlinks`,
which this project's cross-platform goal rules out. See
`docs/architecture/decisions/0002-claude-md-wrapper-not-symlink.md` for the reasoning.

**Your first action in this repository must be to read `AGENTS.md`.** It is the real,
canonical, kept-up-to-date instruction file. Everything below is a redundant summary only in
case that read is somehow skipped — `AGENTS.md` wins on any conflict.

Quick orientation: this repo is mid-migration from a working Go app (now in `legacy/`) to a
Rust rewrite, currently in Phase 0 (foundation, no tracking-engine implementation yet). Current
state lives in `agent/CURRENT_STATE.md`; active work lives in `tasks/active/`.
