---
name: release
description: Cut a release — pick the next SemVer number (patch / minor / major), bump the single version source, run the gates, commit, tag and let release.yml build a draft. Use when the user says "release", "bump the version", "release next patch/minor/major" or names a version like 0.4.1.
---

## Objective

Ship a numbered release where the app, the tag and the GitHub Release all carry the same
version, with every outward step (commit, push, tag, publish) confirmed by the user.

## The one version source

`Cargo.toml` (repo root), `[workspace.package] version = "X.Y.Z"`. Every crate inherits it
(`version.workspace = true`). Nothing else is edited by hand:

- The app shows it in Options → General (`env!("CARGO_PKG_VERSION")` in
  `apps/desktop/src/options.rs`), so the bump updates the UI on the next build.
- `Cargo.lock` follows it (refresh, don't hand-edit).
- `.github/workflows/release.yml` fails if the tag isn't `v` + that version.
- The website reads the latest *published* release at build time (ADR-0023 amendment 2);
  publishing re-runs `website.yml`. No website edit is needed for a version bump.

Version mentions elsewhere (`README.md`, `AGENTS.md`, `docs/PRODUCT.md`,
`website/src/lib/*.test.ts`) are history or test fixtures: leave them.

## Which number (SemVer `MAJOR.MINOR.PATCH`)

Map the user's words to the part that goes up; the parts to its right reset to 0.

| User says | SemVer part | 0.4.0 → | 1.2.3 → |
|---|---|---|---|
| "patch", "bugfix", "smallest", "hotfix" | PATCH | 0.4.1 | 1.2.4 |
| "minor", "feature release", "middle" | MINOR | 0.5.0 | 1.3.0 |
| "major", "biggest", "breaking" | MAJOR | 1.0.0 | 2.0.0 |

**If the user didn't name the part (just "release", "cut a release"), ask; don't pick.**
Read the current version and the changes since the last tag (`git log vX.Y.Z..HEAD`),
then ask one question with the three options, each showing the resulting number (e.g.
"Patch → 0.4.1", "Minor → 0.5.0", "Major → 1.0.0"; in Claude Code use `AskUserQuestion`).
Mark your recommendation using this guide (pre-1.0, as now, MINOR is the "big" step and
1.0.0 is a deliberate user decision):

- PATCH: fixes and small additions; no changed default, no settings-schema bump.
- MINOR: new options/features, a changed default (with its ADR amendment), a settings
  schema version bump, or a platform added/dropped.
- MAJOR: never recommended; only when the user picks it.

Even when the user named the part, state the current and the new version before editing.
If the user names an exact version, check it is greater than the current one.

## Steps

1. **Check the base.** `git status` clean; `git log -5`. Releases are tagged on `main`
   (README "Release"). On a release branch, the bump commit goes on the branch and the tag
   waits until it is merged into `main`.
2. **Bump.** Edit the root `Cargo.toml` version, then refresh the lock without upgrading
   dependencies: `cargo update --workspace` (only workspace crates' versions change; check
   `git diff Cargo.lock` shows nothing else).
3. **Gates** (AGENTS.md, one heavy command at a time): `cargo fmt --all --check`, clippy,
   `cargo test --workspace --locked`. The Options kittest asserts the crate version, so it
   keeps passing without edits.
4. **Docs.** Short entry in `docs/ROADMAP.md` for the version: what changed, known
   limitations, platforms actually tested (say "CI only" where no device was used). Remove
   any temporary per-release docs (e.g. `docs/V0.4.1_*.md`) and their `AGENTS.md` row if
   the release plan says so.
5. **Commit** `Release vX.Y.Z` — only after the user's yes. Push only after a separate yes.
6. **Tag** — only after the user's yes, on `main` once the commit is there:
   `git tag -a vX.Y.Z -m "vX.Y.Z"` then `git push origin vX.Y.Z`.
7. **Draft.** `release.yml` builds macOS, Windows and Linux and creates a *draft* GitHub
   Release with `SHA256SUMS`. Tell the user to check the assets and publish it themselves;
   publishing triggers the website rebuild. A dry run without a release is the workflow's
   manual `workflow_dispatch`.

## Never

- Commit, push, tag or publish without the user's explicit yes (each one separately).
- Move or delete a pushed tag, or reuse a version number; if a release is bad, ship the
  next PATCH.
- Hand-edit the version anywhere other than the root `Cargo.toml`.
- Run release builds or packaging locally without asking (slow).
