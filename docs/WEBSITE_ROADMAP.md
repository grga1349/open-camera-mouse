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
- [x] Hero bottom edge: shallow symmetric V (layered diagonal/blade/wedge tried and rejected); crosshair removed
- [x] Logo as a one-colour SVG mark (`src/assets/logo.svg`), uppercase wordmark
- [x] Hero entrance and hover lift (CSS only; reduced motion collapses them, delays included)
- [x] Reusable `Hero`, `Section`, `ShowcasePlaceholder` components
- [~] Responsive layout: checked at 320 and 1280px for the V edge (headless screenshots); 200% zoom not checked
- [ ] Typography decision: a real typewriter or display face would be a self-hosted web font (currently excluded; needs the user's OK and an ADR-0023 amendment)
- [ ] Header on other pages: yellow with a flat bottom today (only 404 exists); decide whether it gets its own angular edge once the manual/download pages exist
- [ ] Footer design (still the plain scaffold footer)
- [x] Typewriter (system monospace) for all text; Carbon/Paper theme switch in the header (JS + localStorage, hidden without JS)
- [~] Navigation for more pages: "Manual" link in the header (wraps at 320px); revisit when there are more pages
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

## Phase 4 — Downloads  (mostly done)

- [x] Platform data (`src/lib/platforms.ts`: asset suffixes from `release.yml`, notes from README)
- [x] Pure TS modules: `release.ts` (asset matching by suffix, version from the tag) and `detect-platform.ts` (OS only, no CPU guessing), `bun test` in CI
- [x] Release data at **build time**, not in the visitor's browser (user's call, ADR-0023 amendment 2): CI fetches `releases/latest`, the HTML carries the version and `browser_download_url`s; publishing a release rebuilds the site; fallback to the Releases page if the API fails
- [x] Shared by hero and download section: direct file buttons per card (work without JS), JS points the hero button at the visitor's file and marks "Your system"; unsupported-device message (phones, tablets, ChromeOS). Intel Mac, Wayland and ARM can't be detected reliably: the cards' requirements and notes say it
- [x] Decide and document the release-data request (no third-party request from visitors, so no privacy note needed)
- [x] Install steps per OS: the manual's Installation page (Phase 5); the download cards link its sections. No separate `/download/` page
- [ ] Release-triggered rebuild verified on GitHub (needs Pages enabled and a published release on `main`)

## Phase 5 — User manual

The website manual becomes the canonical end-user documentation (README keeps a short
install/use summary and links to it).

Slices: (1) infrastructure + getting started, requirements, installation ✅; (2) using it:
permissions, pointer, tracking point, recentering, clicking/dwell, handover ✅; (3) settings,
accessibility options, troubleshooting, FAQ, README shortened to link here, defaults check;
(4) annotated screenshots (needs the user's screenshots).

- [x] Content collection (`src/content/manual/*.md`, Astro built-in, no doc framework), `/manual/` and `/manual/<slug>/` routes, sidebar (`<details>` on small screens), "On this page", prev/next, heading ids
- [x] Build check resolves relative links and `#anchors` (`check-paths.mjs`)
- [x] Pages: getting started, system requirements, installation per OS
- [x] Pages: first-time setup and permissions, selecting a tracking point, using the pointer, recentering, clicking and dwell clicking, mouse handover (from `status.rs`, `options.rs`, `controller.rs`, `pointer-core`)
- [ ] Pages: settings, accessibility options, troubleshooting, FAQ — each only from real app behaviour (`PRODUCT.md`, `BEHAVIOR.md`, the code)
- [ ] README: keep a short install/use summary and link to the manual (after the site is deployed)
- [ ] Annotated screenshots; a check that manual defaults match the app's

## Phase 6 — Showcase and launch

- [ ] Real 30–45 s demonstration recording (no simulated tracking), poster, captions (`.vtt`), `controls`, no autoplay — not a diagnostic recording; consent of anyone shown
- [ ] Application screenshots, regenerated per UI change
- [ ] Mobile pass; accessibility audit (keyboard, VoiceOver/NVDA, axe/Lighthouse, 200% zoom, reduced motion)
- [ ] Performance check (page weight, Lighthouse)
- [ ] Before merging: delete `docs/WEBSITE_HANDOFF.md` and the references to it (this file, `AGENTS.md`)
- [ ] Merge to `main`, enable Pages, first deployment; ADR-0023 → Accepted
