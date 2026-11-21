# Website

The project website, `website/`, live at `https://grga1349.github.io/open-camera-mouse/`
(GitHub Pages project site). Why this stack: ADR-0023. Open items: [`ROADMAP.md`](ROADMAP.md)
"Website". Agent skills: `astro-development`, `website-design`, `web-accessibility`
(`.agents/skills/`).

## Scope

A small static site that introduces the app, offers the right download and hosts the **user
manual**, the canonical end-user documentation (installation, permissions, use, settings,
troubleshooting); the README keeps a short summary and links to it. Developer documentation
(`docs/`, ADRs) stays in the repository. Behaviour and defaults are defined by
[`PRODUCT.md`](PRODUCT.md) and the code; the site describes them and must not contradict
them. No accounts, forms, backend, analytics, cookies or tracking, and no third-party request
from the visitor's browser (release data is read at build time, see "Downloads").

### Content rules

- State only what `README.md`, `PRODUCT.md`, `BEHAVIOR.md` and the code support: no invented
  features, compatibility, release dates, download numbers, testimonials, or medical claims
  (a PRODUCT non-goal). Versions and file links come from the latest release at build time,
  never typed by hand; version numbers stay out of the copy (`<version>` in file names).
- Don't say on the site that the Windows or Linux builds haven't been tried on a real device
  (user's call, 2026-11-21); that caveat stays in `README.md` and `TESTING.md`. Other limits,
  like untested USB webcams, stay on the site.
- The showcase is a real recording of the app: never a mock-up, simulated tracking or a
  browser-based webcam tracker, never anything from a diagnostic recording, and nobody's
  camera image without their consent.
- Name: "Open Camera Mouse" in full; the only abbreviation is the header wordmark "OCM" below
  `sm` (its link name keeps the full name).
- Plain, respectful words, no marketing superlatives. The user reviews all copy in a final
  pass; wording isn't approved per change.

## Home page

`src/pages/index.astro`, sections in order:

| Section | Content |
|---|---|
| Hero | "Computing without barriers", one-line description, primary download, "See it working" |
| Showcase (`#showcase`) | `public/showcase/demo.mp4` (23 s screen recording on macOS, H.264, no audio, ~1.6 MB) + `poster.jpg`; no autoplay, no captions (no sound; user's call); an `sr-only` figcaption is the text alternative. With JS, a dark scrim and a white play ring cover it until the first play and after the end (native controls without JS) |
| Why it exists (`#why-it-exists`) | Three paragraphs on the classic Camera Mouse and this project, with a numbered "Sources" list (see "Origin story") |
| How it works (`#how-it-works`) | Three steps: pick a point, move your head, click by holding still; link to the manual |
| Engineering (`#engineering`) | Four notes: built from scratch, precise control, tested on every change, private; link to `ARCHITECTURE.md`. Copy only from README, PRODUCT, ARCHITECTURE, TESTING |
| Download (`#download`) | One card per platform (see "Downloads") |
| Questions (`#faq`) | Six questions taken from the rendered manual `faq.md` (`src/lib/home-faq.ts`), so answers have one source |
| Open source (`#open-source`) | MIT line, "Get involved" and "Documentation" link lists |

Screenshots and a Features section were dropped (user's call, 2026-11-21).

### Origin story

Every historical sentence needs a source in the section's list (`originSources` in
`index.astro`): Betke, Gips & Fleming, IEEE TNSRE 10(1), 2002 (DOI
10.1109/TNSRE.2002.1021581), and Boston College News, 25 August 2016. Say plainly that the
project isn't affiliated with or endorsed by Boston College or the original authors. The
site claims nothing about the classic program's current availability, licence, platforms
or whether its development stopped: no source was found. Never use decompiled or
proprietary material (`AGENTS.md`).

## Manual

`/manual/` lists the pages; `/manual/<slug>/` renders `src/content/manual/<slug>.md` (content
collection in `src/content.config.ts`, frontmatter `title`, `description`, `order`; `order`
unique, tested in `manual-order.ts`). Each page has the sidebar (`ManualNav.astro`), an "On
this page" list when it has two or more `##` sections, and previous/next links. Headings get
ids; link other manual pages relatively (`../installation/#windows-x64`), the build checks
them. Code blocks aren't highlighted (`markdown.syntaxHighlight: false`) and wrap.

Settings tables use the header `| Setting | Range | Default |` with the app's label in the
first cell. `manual-defaults.test.ts` checks every such row against
`crates/app-core/src/settings.rs` and that the Settings page lists every setting (a new
setting needs a row in `expectedRows`); the website workflow also runs when `settings.rs`
changes. Every table column needs a header (`manual-tables.test.ts`).

## Design

The site reads as the same product as the app ([`PRODUCT.md`](PRODUCT.md) "UI design",
[`design-lab.html`](design-lab.html)): monochrome, quiet, practical. Changing the visual
direction needs the user's OK.

- **Yellow bookends.** Every page starts and ends on the brand yellow (`.on-brand`, black
  content and focus outline, `theme-color` `#ffd60a`). The header has no borders; on the
  home page the hero continues it and ends in a shallow asymmetric **V** (`.angular-bottom`,
  depth `--cut`, point at `--apex` 30% from the left). The footer starts with a centred **Λ**
  (`.angular-top`). Geometry is decorative: straight edges, no gradients or shadows, no other
  shapes (a layered diagonal/blade/wedge hero was rejected).
- **Accents.** Neon green `#00ff00` only for the target-ring motif. Yellow: the primary
  call-to-action fill, the brand surface, and the `accent` (user's call, 2026-11-21): on
  Carbon it colours page and section headings, the rules above the home page's notes, list
  markers, small uppercase labels and the manual's current-page bar. Paper can't carry
  yellow text (1.4:1), so there `accent` is the text colour and headings get a yellow
  highlighter stroke (`mark`; `.heading-accent`, `.manual-prose h2`). Links are never
  yellow. Everything else is the grey scale.
- **Type.** The system monospace stack for all text (`font-mono` on `html`; no web font, user's
  call). Body `text-base`, help text `text-sm text-muted`, headings `font-semibold` by size,
  not colour; small labels and navigation uppercase `tracking-wide`; no all-caps body text.
  Mono runs about 0.6em per character: check long headings at 320px.
- **Layout.** Mobile first, one column at 320px, columns from `sm`/`md` only for parallel
  content; reading width `max-w-prose`, column `max-w-5xl` with `px-4`/`sm:px-6`; sections
  separated by space and 1px `line` rules, no shadows. No horizontal scroll at 320px, usable
  at 200% zoom. Targets at least 44px (`min-h-11`), since visitors point with their head.
- **Themes.** Carbon (dark) and Paper (light) follow `prefers-color-scheme` unless the visitor
  uses the header's theme switch (see below).
- **Motion.** CSS only, plus the hero squirrel: a one-time entrance on the first page of a
  visit (`data-entered` from an inline head script), hover lifts ≤ 2px, short transitions.
  No loops, parallax or anything that moves a control under the pointer. The global
  `prefers-reduced-motion` rule collapses all of it.
- **Accessibility.** WCAG 2.2 AA, rules in the `web-accessibility` skill: landmarks (no skip
  link, user's call), visible 2px focus outline, underlined links, status never by colour
  alone, reduced motion respected.
- **No JavaScript** for anything HTML and CSS can do; scripts are progressive enhancement
  (theme switch, OS-aware download, video play button, squirrel). No icon fonts or
  libraries (inline SVG), no third-party embeds.

### Tokens

Defined once in `website/src/styles/global.css` (`@theme inline`; Tailwind's default palette
is removed, so only these colour utilities exist). Values follow the app's
(`apps/desktop/src/theme.rs`), except the dimmer Carbon text.

| Utility | Carbon | Paper | Use |
|---|---|---|---|
| `bg-bg` | `#000000` | `#ececec` | page background |
| `bg-surface` / `bg-surface-2` | `#0a0a0a` / `#151515` | `#f8f8f8` / `#ffffff` | panels, cards, code |
| `border-line` / `border-line-strong` | `#262626` / `#3f3f3f` | `#d6d6d6` / `#a8a8a8` | separators / decorative outlines |
| `text-text` / `text-muted` | `#e6e6e6` / `#8f8f8f` | `#111111` / `#5a5a5a` | body / help text (app Carbon text is `#fafafa`; dimmed for less glare) |
| `text-accent` / `border-accent` | `#ffd60a` | `#111111` | headings, rules, list markers, labels, current manual page |
| `mark` | `transparent` | `#f5c518` | highlighter under headings |
| `*-marker` | `#00ff00` | `#00ff00` | target-ring motif only |
| `bg-yellow` + `text-on-yellow` | `#ffd60a` / `#111111` | `#f5c518` / `#111111` | primary call to action |
| `brand` / `on-brand` (`.on-brand`) | `#ffd60a` / `#000000` | same | header, hero, footer; inverted for the hero's primary button |
| `bg-scrim/50` + `text-on-scrim` | `#000000` / `#ffffff` | same | overlay and play ring on the demo video |
| `rounded-frame` / `rounded-widget` | 8px / 6px | same | panels, images / buttons |

Measured contrast (WCAG 2.x): text on bg 16.8:1 / 16.0:1; accent on bg 14.9:1 / 16.0:1;
muted on bg 6.5:1 / 5.8:1, on surface-2 5.6:1 / 6.9:1; on-yellow on yellow 13.4:1 / 11.6:1;
on-brand on brand 14.9:1; white on scrim/50 over the video's brightest pixel 3.9:1.
Below 3:1, so decorative only: line-strong on bg (2.0:1), yellow on Paper bg (1.4:1),
marker on Paper bg (1.2:1). A control boundary that matters uses `text`-coloured borders.
A new token goes in `global.css` and this table together, with its contrast.

### Header and hero

Header: logo (`src/assets/logo.svg`, one-colour, `currentColor`, `aria-hidden`) and the
uppercase wordmark as the home link; "Manual" (below `sm` an outline book icon with
screen-reader text; `aria-current="page"` on `/manual/`, no underline on manual pages since
it's the only link); the GitHub mark; the theme switch. One row at every width. The app icon
PNG is only the favicon.

**Theme switch:** a `<button role="switch">` "Night theme" (`aria-checked` true = Carbon), a
pill with sun and moon and a sliding knob. It sets `data-theme` on `<html>` and stores
`ocm-theme` in `localStorage`; an inline script in `BaseLayout` applies it before first
paint. It ships `hidden` and its script reveals it, so without JS the OS setting applies.

Hero: the h1 "Computing without barriers" in heavy uppercase (CSS, sentence case in the
source), two lines from `lg`; the description; the primary button (black, yellow text) and an
outlined "See it working", stacked below `sm` so the longer "Download for …" label can't
wrap and shift the page; then the download line, whose variants share one grid cell
(`.detect-stack`) so the block keeps the longest one's height. From `lg` a decorative
squirrel (`HeroSquirrel.astro`, pose maths in `src/lib/squirrel.ts`) watches the pointer:
faceted, not spiky, no crossing lines; it moves only while the pointer moves and stays still
with reduced motion, on touch-only screens and without JS. `aria-hidden`, not a control.

Footer: logo and wordmark (not a link), the description, a "Footer" `nav` with "Manual" and
"Project" link lists, a 2px black rule, the licence line and "No cookies, no analytics."

### Manual pages

Under the flat yellow header: from `md` a 13rem sidebar (current page bold with a 4px
`accent` bar and `aria-current`), below `md` a collapsed `<details>` above the page. Article:
h1, the description as a muted lead, "On this page" (4px `line` bar), the Markdown in
`.manual-prose` (`code`/`kbd` boxed on `surface-2`, code blocks on `surface` that wrap,
tables with `line` rows that fit 320px), then previous/next.

## Downloads

The version and file links are baked in at build time (ADR-0023 amendment 2):
`latest-release.ts` reads `src/data/latest-release.json` (written by CI with `gh api`) or,
locally, fetches `releases/latest` anonymously; `release.ts` keeps only assets named
`open-camera-mouse-<version>-<suffix>` (suffixes in `platforms.ts`) hosted by this repository.
Without usable data the build logs `[release]` and the page links "Go to the latest release".

Each card shows its requirement, notes, one outlined button per file and a link to its
install steps; below the cards a link to the release. Without JS the hero button goes to
`#download`. With JS, `DownloadEnhancer.astro` detects the OS (never the CPU, so the
requirement such as Apple Silicon is always shown): the hero button reads "Download for
<OS>" and links the file, the card gets a "Your system" label, and phones, tablets and
ChromeOS get "runs on macOS, Windows and Linux computers".

## Technology

| Piece | Choice |
|---|---|
| Site generator | Astro 7, static output; no client JS unless a component adds a script |
| Styling | Tailwind CSS v4 via `@tailwindcss/vite`, CSS-first tokens, no config file |
| Language | TypeScript in frontmatter and `src/lib/`, plain `<script>` for interactions |
| Packages | Bun (`bun.lock`, version in `packageManager`); Node isn't needed |
| Hosting / CI | GitHub Pages, deployed by `.github/workflows/website.yml` (OIDC, no secrets) |

Dependencies (exact versions in `website/package.json`): `astro`, `tailwindcss`,
`@tailwindcss/vite`; dev: `@astrojs/check`, `typescript`. A new package needs an ADR-0023
amendment.

## Structure

```
website/
  astro.config.mjs        site, base "/open-camera-mouse/", trailingSlash "always", Tailwind
  public/                 copied verbatim: favicon.png (copy of assets/icon/png/), showcase/
  scripts/check-paths.mjs post-build check of internal URLs and #anchors in dist/
  src/
    layouts/BaseLayout.astro   <head>, header, <main id="main">, footer; `fullBleed` pages
    components/                SiteHeader, SiteFooter, Hero, HeroSquirrel, Section, Showcase,
                               DownloadEnhancer (client script), ManualNav
    pages/                     index, 404, manual/index, manual/[slug]
    content.config.ts          the `manual` collection; content/manual/*.md
    styles/global.css          Tailwind import, tokens, base styles, brand surface, angular
                               edges, entrance animations, reduced motion
    lib/                       url() helper, platforms, release, detect-platform,
                               latest-release, manual-order, faq-excerpt/home-faq, squirrel,
                               manual-defaults, manual-tables (pure modules, *.test.ts)
    data/latest-release.json   written by CI only (gitignored)
```

Conventions: every internal URL goes through `url()` from `src/lib/url.ts` and page paths
end in `/`; `check-paths.mjs` fails the build on a URL outside `/open-camera-mouse/`, a
missing file or an unknown `#id`. Pages use `BaseLayout` with a `description`; header and
footer live only there.

## Local development

```sh
cd website
bun install                  # first time, or after a dependency change
bun run dev                  # http://localhost:4321/open-camera-mouse/
bun run check                # astro check: 0 errors/warnings
bun run test                 # bun test: src/lib/*.test.ts
bun run build                # dist/ + path check
bun run preview              # serve dist/ under the base path
```

Turn off Astro's CLI telemetry on your machine with `bunx astro telemetry disable` (CI sets
`ASTRO_TELEMETRY_DISABLED=1`).

**Audits** (not in CI, no dependency committed): install Lighthouse in a scratch folder
outside the repo (it bundles `axe-core` and `puppeteer-core`), then run Lighthouse against
`bun run preview` and axe (`wcag2a`…`wcag22aa`, `best-practice`) on every page in both themes
at 320/768/1280px. Last run 2026-11-21: Lighthouse 100 in all four categories on every page,
mobile and desktop (manual pages 12–14 KiB, home 119 KiB with the poster); axe 0
violations; no horizontal scroll; CLS 0 on mobile (desktop ≈ 0.0004 from the theme switch
appearing after its script).

## CI and deployment

`.github/workflows/website.yml`:

| Event | What runs |
|---|---|
| Pull request touching `website/**`, the workflow or `crates/app-core/src/settings.rs` | install (`--frozen-lockfile`), check, test, latest release via `gh api`, build |
| Push to `main` (same paths), or a manual run on `main` | same, then deploys to Pages if the repository variable `WEBSITE_DEPLOY` is `true` |
| A release is published | if `WEBSITE_DEPLOY` is `true`: dispatches a run on `main`, so the new version goes live |

The build job has only `contents: read`, the `rebuild` job only `actions: write`; the deploy
job alone has `pages: write` and `id-token: write` and never runs for pull requests. The
Rust CI (`ci.yml`) ignores `website/**`, `docs/WEBSITE.md` and `website.yml`.

Pages is enabled (source: GitHub Actions, `github-pages` environment limited to `main`) and
`WEBSITE_DEPLOY` is `true`; the first deployment ran from the merge of PR #2
(2026-11-21). To deploy without a website change: Actions → Website → Run workflow on `main`.
