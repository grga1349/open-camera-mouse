# ADR-0023: Website in `website/` — Astro + Tailwind v4, built with Bun, hosted on GitHub Pages

Status: Proposed (scaffold on `feature/website`, 2026-11-20)
Date: 2026-11-20

## Context

The project needs a public website at `https://grga1349.github.io/open-camera-mouse/` (a
GitHub Pages *project* site, so everything is served under `/open-camera-mouse/`, not `/`).
The user asked for it in the same repository as the app, with a fixed stack: Astro, Tailwind
CSS v4 through its Vite plugin, TypeScript where useful, vanilla JavaScript, Bun, GitHub
Actions, and no UI framework, component library, icon framework, CMS, backend, web fonts or
analytics. This is the first non-Rust code since the Go app was removed (ADR-0022), so it
needs a boundary that keeps it out of the Rust workspace and the Rust CI.

## Options considered

1. **Separate repository** (or the `gh-pages` branch of this one). Keeps the trees apart, but
   the site then can't share the icon and the design tokens without copying them across
   repos, and docs drift. The user asked for the monorepo.
2. **Plain hand-written HTML in `docs/`** (Pages "deploy from branch"). No build step, but no
   layouts, no shared header/footer, no base-path handling, and `docs/` already holds the
   project documentation.
3. **`website/` as a self-contained Astro project, deployed by a separate Actions workflow
   (Pages "GitHub Actions" source). Chosen.** Astro renders to static HTML with zero
   client JavaScript by default; Tailwind v4 is CSS-first (`@theme` in a stylesheet, no
   config file). Bun installs from a committed text lockfile (`bun.lock`).

## Decision

Option 3.

- `website/` is its own package (`package.json`, `bun.lock`) and is not a Cargo member; no
  Rust crate depends on it and it depends on nothing outside `website/` at build time. The
  icon is copied into `website/public/` (regenerate when `assets/icon/` changes).
- Astro `site` = `https://grga1349.github.io`, `base` = `/open-camera-mouse/`,
  `trailingSlash: "always"`, static output. Internal links and asset URLs go through one
  helper (`src/lib/url.ts`) built on `import.meta.env.BASE_URL`; a post-build script
  (`scripts/check-paths.mjs`) fails the build if any root-relative URL in `dist/` escapes the
  base.
- Dependencies: `astro`, `tailwindcss`, `@tailwindcss/vite`; dev-only `@astrojs/check` and
  `typescript` (6.x: `@astrojs/check` 0.9 does not accept 7.x yet). All MIT/Apache-2.0.
  Adding any other package needs an amendment to this ADR, the same rule as crates.
- `.github/workflows/website.yml` builds on pull requests and pushes that touch the website,
  including `feature/website`. It deploys only from `main` (push or manual run), and only when the
  repository variable `WEBSITE_DEPLOY` is `true`, so nothing is published before Pages is
  enabled. The build job has `contents: read` only; `pages: write` and `id-token: write`
  exist only on the deploy job, which never runs for `pull_request` events. No secrets.
- `.github/workflows/ci.yml` gets `paths-ignore` for website-only changes, so they don't
  compile Rust.

## Consequences

- Two toolchains in one repo: website work needs Bun (≥ 1.3) and Node is not required. The
  Rust gates don't cover `website/`; `AGENTS.md` lists the website gates separately.
- If branch protection on `main` ever *requires* the Rust CI checks, a website-only PR will
  wait on checks that never start (skipped by `paths-ignore`). Fix then by making those
  checks non-required or adding a no-op job, not now.
- The visual design follows `docs/PRODUCT.md` tokens (Carbon/Paper, two accents); the actual
  landing-page design is a separate task (`docs/WEBSITE_DESIGN.md`).
- Status becomes Accepted when the user approves the merge to `main`.
