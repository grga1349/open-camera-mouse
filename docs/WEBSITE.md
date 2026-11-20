# Website

The project website, `website/`, published at
`https://grga1349.github.io/open-camera-mouse/` (GitHub Pages project site). Decision and
trade-offs: ADR-0023. Look and content rules: [`WEBSITE_DESIGN.md`](WEBSITE_DESIGN.md).
Phases and status: [`WEBSITE_ROADMAP.md`](WEBSITE_ROADMAP.md).

## Purpose and scope

A small static site that introduces the app, offers the right download and, from Phase 5,
hosts the **user manual**, which becomes the canonical end-user documentation (installation,
permissions, use, settings, troubleshooting). Developer documentation (`docs/`, ADRs) stays
in the repository. Product behaviour and defaults remain defined by `docs/PRODUCT.md` and the
code; the manual describes them and must not contradict them. No accounts, forms, backend,
analytics or tracking; the only planned third-party request is the GitHub releases API for
the download button (Phase 4, decision pending).

Status: first homepage pass (yellow hero, showcase placeholder, how it works, download
cards, open source); see the roadmap.

## Homepage

Planned sections, in order. Built ones are marked; the rest wait for their roadmap phase.

| # | Section | Purpose | Status |
|---|---|---|---|
| 1 | Hero | What it is, that it uses an ordinary webcam, free and open source; primary download, link to the showcase | built (draft copy) |
| 2 | Showcase (`#showcase`) | A real recording of the app in use | placeholder |
| 3 | Why it exists | The classic Camera Mouse's legacy and why a maintained open-source successor is useful | Phase 3 |
| 4 | How it works (`#how-it-works`) | Three steps: pick a spot, move your head, click by holding still | built |
| 5 | Features | Real features, each with a real screenshot | Phase 3 |
| 6 | Engineering | Independent Rust implementation, tracking and stability, privacy (on-device), testing | Phase 3 |
| 7 | Download (`#download`) | Supported platforms, caveats, install guidance | built (static); OS-aware in Phase 4 |
| 8 | Open source (`#open-source`) | Repository, issues, contributing, documentation | built (minimal) |

### Origin story

The classic Camera Mouse was influential accessibility software; Open Camera Mouse is an
independent project that continues the idea with a modern open-source implementation. When
writing this section: verify every historical claim against a source and cite it; separate
the discontinued classic distribution from any later research or implementations; say
plainly that the project is not affiliated with or endorsed by the original authors; be
respectful, no marketing superlatives. Never use decompiled or proprietary material
(`AGENTS.md`).

## Technology

| Piece | Choice | Why |
|---|---|---|
| Site generator | Astro 7, static output | HTML at build time, no client JS unless a component adds a script |
| Styling | Tailwind CSS v4 via `@tailwindcss/vite` | CSS-first `@theme` tokens, no config file |
| Language | TypeScript in frontmatter and `src/lib/`, vanilla JS for any interaction | `astro check` catches type errors; no framework runtime |
| Packages | Bun (`bun.lock`, version in `packageManager`) | one tool to install, run scripts and run the path check |
| Hosting / CI | GitHub Pages, deployed by Actions (`.github/workflows/website.yml`) | built-in OIDC deployment, no secrets |

Dependencies (exact versions in `website/package.json`): `astro`, `tailwindcss`,
`@tailwindcss/vite`; dev: `@astrojs/check`, `typescript` (6.x, the newest
`@astrojs/check` accepts). Adding a package needs an amendment to ADR-0023.

## Structure

```
website/
  astro.config.mjs        site, base "/open-camera-mouse/", trailingSlash "always", Tailwind plugin
  package.json, bun.lock  scripts and pinned dependencies
  tsconfig.json           extends astro/tsconfigs/strict
  public/                 copied verbatim: favicon.png (copy of assets/icon/png/)
  scripts/check-paths.mjs post-build check of internal URLs in dist/
  src/
    layouts/BaseLayout.astro   <html>, <head>, skip link, header, <main id="main">, footer;
                               `fullBleed` lets a page lay out its own full-width sections
    assets/logo.svg            one-colour logo mark (currentColor), imported as a component
    components/SiteHeader.astro, SiteFooter.astro   header on the yellow brand surface
    components/Hero.astro      yellow angular hero
    components/Section.astro   homepage section: column, anchor id, h2
    components/ShowcasePlaceholder.astro   stand-in for the demo video (swap-in markup inside)
    pages/index.astro, 404.astro   routes (file = URL)
    styles/global.css          Tailwind import, tokens, base styles, brand surface, angular
                               edge, entrance animations, reduced motion
    lib/url.ts                 url() base-path helper, REPO_URL
    lib/platforms.ts           release platforms and asset suffixes (from release.yml)
```

## Local development

Needs Bun ≥ 1.3 (`curl -fsSL https://bun.sh/install | bash`); Node isn't needed.

```sh
cd website
bun install                  # first time, or after a dependency change
bun run dev                  # http://localhost:4321/open-camera-mouse/
bun run check                # astro check (types, .astro diagnostics)
bun run build                # dist/ + path check
bun run preview              # serve dist/ at http://localhost:4321/open-camera-mouse/
```

The dev server and preview serve under the base too, so a link that only works at `/` shows
up locally. Astro's anonymous CLI telemetry is on by default; turn it off on your machine
with `bunx astro telemetry disable` (CI sets `ASTRO_TELEMETRY_DISABLED=1`). It is not part
of the built site.

## Conventions

- **Base path.** Every internal page or `public/` URL goes through `url()` from
  `src/lib/url.ts`; page paths end in `/`. `bun run build` runs `scripts/check-paths.mjs`,
  which fails on any root-relative `href`/`src`/`srcset`/CSS `url()` in `dist/` that is
  outside `/open-camera-mouse/` or points at a missing file.
- **No JavaScript by default.** Static UI uses HTML and CSS; an interaction that needs JS
  is a component `<script>` written as progressive enhancement.
- **One layout.** Pages use `BaseLayout` with a `description` (and `title` except on the
  home page); header and footer are only in the layout. Full-width pages pass `fullBleed`
  and put content in `Section` (or their own `max-w-5xl` column).
- **Tokens, not colours.** Tailwind's default palette is removed; use the token utilities
  listed in `WEBSITE_DESIGN.md`.
- Agent skills: `astro-development`, `website-design`, `web-accessibility`
  (`.agents/skills/`).

## CI and deployment

`.github/workflows/website.yml`:

| Event | What runs |
|---|---|
| Pull request touching `website/**` or the workflow | `bun install --frozen-lockfile`, `bun run check`, `bun run build` |
| Push to `feature/website` (same paths) | same |
| Push to `main` (same paths), or a manual run on `main` | same, then uploads `website/dist` and deploys, **only if** the repository variable `WEBSITE_DEPLOY` is `true` |

The build job only has `contents: read`; the deploy job alone has `pages: write` and
`id-token: write`, and never runs for pull requests. The Rust CI (`ci.yml`) ignores
`website/**`, `docs/WEBSITE*.md` and `website.yml`, so website-only changes don't compile
Rust.

### Enabling Pages (one-time, by the repository owner)

1. Settings → Pages → Build and deployment → Source: **GitHub Actions**.
2. Settings → Environments → `github-pages` (created by step 1): keep the deployment branch
   rule at `main`.
3. Settings → Secrets and variables → Actions → Variables: add `WEBSITE_DEPLOY` = `true`.
4. Run the workflow from `main` (Actions → Website → Run workflow) or push a website change.
