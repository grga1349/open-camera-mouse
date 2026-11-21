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
analytics or tracking, and no third-party requests from the visitor's browser: the latest
release is read at build time (ADR-0023 amendment 2, "Downloads" below).

Status: first homepage pass (yellow hero, demo video, why it exists, how it works, engineering,
OS-aware download cards, questions, open source) and the manual's thirteen pages (getting started, requirements,
installation, everyday use, settings, accessibility options, troubleshooting, FAQ); see the
roadmap.

## Homepage

Planned sections, in order. Built ones are marked; the rest wait for their roadmap phase.

| # | Section | Purpose | Status |
|---|---|---|---|
| 1 | Hero | What it is, that it uses an ordinary webcam, free and open source; primary download, link to the showcase | built (draft copy) |
| 2 | Showcase (`#showcase`) | A real recording of the app in use: `public/showcase/` (`demo.mp4` 23 s, H.264, no audio, ~1.6 MB; `poster.jpg`), no autoplay, no captions (no sound; the user dropped them), the figcaption (screen readers only, `sr-only`) is the text alternative | built |
| 3 | Why it exists (`#why-it-exists`) | Three short paragraphs: the classic Camera Mouse (Boston College, Gips and Betke, EagleEyes), its free release in 2007 and reach, then Open Camera Mouse as an independent successor with a non-affiliation line; a numbered "Sources" list (`originSources` in `index.astro`) | built |
| 4 | How it works (`#how-it-works`) | Three steps: pick a spot, move your head, click by holding still | built |
| 5 | Features | Real features, each with a real screenshot | Phase 3 |
| 6 | Engineering (`#engineering`) | Four notes: independent Rust implementation, tracking and stability, privacy (on-device), testing (CI on all three systems; the not-yet-tried-on-a-device caveat is left out of this section, user's call); link to `ARCHITECTURE.md`. Copy only from README, PRODUCT, ARCHITECTURE, TESTING | built (before Features, which waits for screenshots) |
| 7 | Download (`#download`) | Supported platforms, caveats, direct files, install guidance | built, OS-aware |
| 7b | Questions (`#faq`) | Six questions from the manual's FAQ (`HOME_FAQ` in `src/lib/home-faq.ts`), taken from the rendered `faq.md` so the answers have one source; links rebased to `/manual/…`; "All questions →" | built |
| 8 | Open source (`#open-source`) | Licence line, then two labelled link lists: "Get involved" (source, issues, build steps in the README, `AGENTS.md`) and "Documentation" (manual, PRODUCT, TESTING, ADRs) | built |

### Origin story

The classic Camera Mouse was influential accessibility software; Open Camera Mouse is an
independent project that continues the idea with a modern open-source implementation. When
writing this section: verify every historical claim against a source and cite it; separate
the discontinued classic distribution from any later research or implementations; say
plainly that the project is not affiliated with or endorsed by the original authors; be
respectful, no marketing superlatives. Never use decompiled or proprietary material
(`AGENTS.md`).

Built (2026-11-21) from three sources, listed under the section: Betke, Gips & Fleming,
IEEE TNSRE 10(1), 2002 (DOI 10.1109/TNSRE.2002.1021581); Boston College News, 25 August
2016 (free since June 2007, over three million downloads, users); Boston College News, In
Memoriam James Gips, 12 June 2018. The section claims nothing about the classic program's
current availability, licence or platforms: `cameramouse.org` refused connections on
2026-11-21 and only third-party download sites list Camera Mouse 2018, so neither is cited.
A new sentence needs a source in that list.

## Manual

`/manual/` lists the pages; `/manual/<slug>/` renders `src/content/manual/<slug>.md`
(content collection in `src/content.config.ts`, frontmatter `title`, `description`,
`order`). Each page has the sidebar (`ManualNav.astro`: a collapsed `<details>` under `md`,
a sidebar from `md` up), an "On this page" list of its `##` sections when there are two or
more, and previous/next links. Astro gives every heading an id, so sections are linkable;
`order` must be unique (`src/lib/manual-order.ts`, tested). Code blocks aren't highlighted
(`markdown.syntaxHighlight: false`, monochrome) and wrap instead of scrolling.

Writing a page: only real app behaviour (`README.md`, `PRODUCT.md`, `BEHAVIOR.md`, the
code); link other manual pages relatively (`../installation/#windows-x64`), the build
checks them. Version numbers stay out of the copy (`<version>` in file names).

Settings tables use the header `| Setting | Range | Default |` and the app's label in the
first cell. `src/lib/manual-defaults.test.ts` checks every such row on every page against
`crates/app-core/src/settings.rs` (ranges and `Default` impls) and that the Settings page
lists every setting; a new setting needs a row in `expectedRows` (`manual-defaults.ts`).
The website workflow also runs when `settings.rs` changes. Every table column needs a
header (no empty first cell; screen readers announce it): `manual-tables.test.ts` checks it.

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
  scripts/check-paths.mjs post-build check of internal URLs and #anchors in dist/
  src/
    layouts/BaseLayout.astro   <html>, <head>, header, <main id="main">, footer;
                               `fullBleed` lets a page lay out its own full-width sections
    assets/logo.svg            one-colour logo mark (currentColor), imported as a component
    components/SiteHeader.astro, SiteFooter.astro   header and footer on the yellow brand surface
    components/Hero.astro      yellow angular hero
    components/HeroSquirrel.astro   decorative squirrel that watches the pointer beside the hero text (lg+)
    components/Section.astro   homepage section: column, anchor id, h2
    components/Showcase.astro    the demo video (`<video controls preload="metadata">`)
    components/DownloadEnhancer.astro      client script: OS detection → hero button, "Your system"
    components/ManualNav.astro   manual contents: <details> on small screens, sidebar from md
    pages/index.astro, 404.astro   routes (file = URL)
    pages/manual/index.astro, [slug].astro   manual contents and one manual page
    content.config.ts          the `manual` content collection (glob over content/manual/*.md)
    content/manual/*.md        manual pages (frontmatter: title, description, order)
    styles/global.css          Tailwind import, tokens, base styles, brand surface, angular
                               edge, entrance animations, reduced motion
    lib/url.ts                 url() base-path helper, REPO_URL
    lib/platforms.ts           release platforms and asset suffixes (from release.yml)
    lib/release.ts             API response → version + per-platform files (pure, tested)
    lib/detect-platform.ts     user agent → macos / windows / linux / unsupported (pure, tested)
    lib/latest-release.ts      build-time loader: CI file, else anonymous fetch, else null
    lib/manual-order.ts        manual page order and previous/next (pure, tested)
    lib/faq-excerpt.ts, home-faq.ts   home page FAQ: questions picked from manual/faq.md (tested)
    lib/squirrel.ts            hero squirrel pose maths (pure, tested)
    lib/manual-defaults.ts     app settings source → expected manual table rows (pure, tested
                               against settings.rs and content/manual/*.md)
    lib/manual-tables.ts       manual table header rows (tested: no empty header cells)
    data/latest-release.json   written by CI only (gitignored)
```

## Local development

Needs Bun ≥ 1.3 (`curl -fsSL https://bun.sh/install | bash`); Node isn't needed.

```sh
cd website
bun install                  # first time, or after a dependency change
bun run dev                  # http://localhost:4321/open-camera-mouse/
bun run check                # astro check (types, .astro diagnostics)
bun run test                 # bun test: src/lib/*.test.ts
bun run build                # dist/ + path check
bun run preview              # serve dist/ at http://localhost:4321/open-camera-mouse/
```

The dev server and preview serve under the base too, so a link that only works at `/` shows
up locally. Astro's anonymous CLI telemetry is on by default; turn it off on your machine
with `bunx astro telemetry disable` (CI sets `ASTRO_TELEMETRY_DISABLED=1`). It is not part
of the built site.

Audits (not in CI, no dependency committed): install Lighthouse in a scratch folder outside
the repo (it bundles `axe-core` and `puppeteer-core`), then run Lighthouse against `bun run
preview` and the full axe rule set (`wcag2a`…`wcag22aa`, `best-practice`) on every page in
both themes at 320/768/1280px. Last run 2026-11-21: Lighthouse 100 in all four categories
(mobile and desktop, 11–17 KiB per page, CLS 0); axe 0 violations, the only "incomplete"
items are the `aria-hidden` `→` arrows; no horizontal scroll at any width.

## Conventions

- **Base path.** Every internal page or `public/` URL goes through `url()` from
  `src/lib/url.ts`; page paths end in `/`. `bun run build` runs `scripts/check-paths.mjs`,
  which resolves every internal `href`/`src`/`srcset`/CSS `url()` in `dist/` (root-relative,
  relative or `#fragment`) and fails if it leaves `/open-camera-mouse/`, points at a missing
  file, or names an `#id` that the target page doesn't have.
- **No JavaScript by default.** Static UI uses HTML and CSS; an interaction that needs JS
  is a component `<script>` written as progressive enhancement.
- **One layout.** Pages use `BaseLayout` with a `description` (and `title` except on the
  home page); header and footer are only in the layout. Full-width pages pass `fullBleed`
  and put content in `Section` (or their own `max-w-5xl` column).
- **Tokens, not colours.** Tailwind's default palette is removed; use the token utilities
  listed in `WEBSITE_DESIGN.md`.
- Agent skills: `astro-development`, `website-design`, `web-accessibility`
  (`.agents/skills/`).

## Downloads

The version and file links are baked in at build time (ADR-0023 amendment 2):
`latest-release.ts` reads `src/data/latest-release.json` (written by CI with `gh api`) or,
locally, fetches `releases/latest` anonymously; `release.ts` keeps only assets named
`open-camera-mouse-<version>-<suffix>` (suffixes from `platforms.ts`) whose URL is this
repository's release download. If nothing usable comes back, the build logs `[release]` and
the page falls back to "Go to the latest release". Without JavaScript the hero button goes
to `#download` and every card links its files; with it, `DownloadEnhancer.astro` points the
hero button at the visitor's file and shows the matching `data-detect` notes. Only the OS is
detected, never the CPU.

## CI and deployment

`.github/workflows/website.yml`:

| Event | What runs |
|---|---|
| Pull request touching `website/**`, the workflow or `crates/app-core/src/settings.rs` | `bun install --frozen-lockfile`, `bun run check`, `bun run test`, latest release via `gh api`, `bun run build` |
| Push to `feature/website` (same paths) | same |
| Push to `main` (same paths), or a manual run on `main` | same, then uploads `website/dist` and deploys, **only if** the repository variable `WEBSITE_DEPLOY` is `true` |
| A release is published | if `WEBSITE_DEPLOY` is `true`: dispatches a manual run on `main` (so the new version goes live) |

The build job only has `contents: read`; the `rebuild` job only `actions: write`; the deploy job alone has `pages: write` and
`id-token: write`, and never runs for pull requests. The Rust CI (`ci.yml`) ignores
`website/**`, `docs/WEBSITE*.md` and `website.yml`, so website-only changes don't compile
Rust.

### Enabling Pages (one-time, by the repository owner)

1. Settings → Pages → Build and deployment → Source: **GitHub Actions**.
2. Settings → Environments → `github-pages` (created by step 1): keep the deployment branch
   rule at `main`.
3. Settings → Secrets and variables → Actions → Variables: add `WEBSITE_DEPLOY` = `true`.
4. Run the workflow from `main` (Actions → Website → Run workflow) or push a website change.
