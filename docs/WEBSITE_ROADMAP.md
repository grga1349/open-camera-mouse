# Website roadmap

Phased plan for the website (`website/`, ADR-0023). How it's built: [`WEBSITE.md`](WEBSITE.md);
how it looks and what it may say: [`WEBSITE_DESIGN.md`](WEBSITE_DESIGN.md). One phase (or
slice of one) per session; tick items when they land and note what was deferred. Status as of
2026-11-20, all on `feature/website` (nothing merged or deployed yet).

Legend: `[x]` done · `[~]` started · `[ ]` open

Current session state and next steps: `docs/WEBSITE_HANDOFF.md` (temporary, deleted
before the merge to `main`).

## Phase 1 — Foundation  ✅

- [x] Astro 7 + Tailwind v4 (`@tailwindcss/vite`) + Bun, static output, `/open-camera-mouse/` base
- [x] Structure: `BaseLayout`, header, footer, 404, `url()` helper, post-build path check
- [x] GitHub Pages workflow (`website.yml`): build on PRs and `feature/website`, deploy from `main` behind `WEBSITE_DEPLOY`
- [x] Rust CI ignores website-only changes
- [x] Docs (`WEBSITE.md`, `WEBSITE_DESIGN.md`, this file), ADR-0023
- [x] Skills `astro-development`, `website-design`, `web-accessibility`; `AGENTS.md` website section
- [x] Minimal homepage (now superseded by the Phase 2/3 first pass)
- [ ] Enable Pages in repo settings + set `WEBSITE_DEPLOY=true` (owner, after merge to `main`)

## Phase 2 — Visual identity  (first pass)

- [x] Brand surface token (`brand` `#ffd60a` / `on-brand` `#000`), yellow angular hero with diagonal edge
- [x] Pages start yellow: header on the brand surface, no header borders or rules
- [x] Layered hero shape: diagonal edge, blade under it, gapped black wedge (`lg`+); crosshair removed
- [x] Logo as a one-colour SVG mark (`src/assets/logo.svg`), uppercase wordmark
- [x] Typewriter-style mono accents (nav, labels) on the system monospace stack
- [x] Hero entrance and hover lift (CSS only; reduced motion collapses them, delays included)
- [x] Reusable `Hero`, `Section`, `ShowcasePlaceholder` components
- [~] Responsive layout: checked at 320, 820, 1024 and 1280px (headless screenshots); 200% zoom not checked
- [ ] Typography decision: a real typewriter or display face would be a self-hosted web font (currently excluded; needs the user's OK and an ADR-0023 amendment)
- [ ] Header on other pages: yellow with a flat bottom today (only 404 exists); decide whether it gets its own angular edge once the manual/download pages exist
- [ ] Footer design (still the plain scaffold footer)
- [ ] Navigation for more pages (manual, download) incl. mobile; theme switch (Carbon/Paper, needs JS + storage)
- [ ] Hero geometry refinement, section transitions, tracking-inspired accents beyond the hero
- [ ] Hero copy approved by the user ("Your head. Your mouse." is a draft)

## Phase 3 — Homepage content

Built: hero, showcase placeholder, how it works (3 steps, from README "Use"), download cards,
open source. Open:

- [ ] Why it exists: the classic Camera Mouse and its legacy. **Research first**: every
      historical claim needs a citable source; distinguish the discontinued classic
      distribution from later research or implementations; no implied affiliation or
      endorsement (rules in `WEBSITE.md` "Origin story")
- [ ] Features with real screenshots (tracking window, Options)
- [ ] Engineering: independent Rust implementation, tracking and stability, privacy, testing
- [ ] Open source: contributing, documentation links

## Phase 4 — Downloads  (next)

- [~] Platform data (`src/lib/platforms.ts`: asset suffixes from `release.yml`, notes from README)
- [ ] Pure TS module: OS detection (no CPU-architecture guessing), asset matching by suffix, unit tests with `bun test` (built in, no new package)
- [ ] Client script: fetch `releases/latest` from the GitHub API, show version, use `browser_download_url`; fallback to the Releases page on network error or rate limit (403/429); works without JS
- [ ] Shared by hero and download section; manual platform choice; unsupported-platform message (Intel Mac, Wayland, ARM Windows/Linux)
- [ ] Decide and document the request to `api.github.com` (first third-party request; ADR-0023 amendment, privacy note)
- [ ] Download page (`/download/`) with install steps per OS

## Phase 5 — User manual

The website manual becomes the canonical end-user documentation (README keeps a short
install/use summary and links to it).

- [ ] Content collection (`src/content/manual/*.md`, Astro built-in, no doc framework), `/manual/<slug>/` routes, sidebar/prev-next, linkable headings
- [ ] Pages: getting started, system requirements, installation per OS, first-time setup and permissions, using the pointer, selecting a tracking point, recentering, clicking and dwell clicking, mouse handover, settings, accessibility options, troubleshooting, FAQ — each only from real app behaviour (`PRODUCT.md`, `BEHAVIOR.md`, the code)
- [ ] Annotated screenshots; a check that manual defaults match the app's

## Phase 6 — Showcase and launch

- [ ] Real 30–45 s demonstration recording (no simulated tracking), poster, captions (`.vtt`), `controls`, no autoplay — not a diagnostic recording; consent of anyone shown
- [ ] Application screenshots, regenerated per UI change
- [ ] Mobile pass; accessibility audit (keyboard, VoiceOver/NVDA, axe/Lighthouse, 200% zoom, reduced motion)
- [ ] Performance check (page weight, Lighthouse)
- [ ] Before merging: delete `docs/WEBSITE_HANDOFF.md` and the references to it (this file, `AGENTS.md`)
- [ ] Merge to `main`, enable Pages, first deployment; ADR-0023 → Accepted
