# Website handoff (temporary)

Session-to-session handoff for the website story on `feature/website`. Rewrite it at the end
of every website session. **Delete this file before merging to `main`** (roadmap Phase 6).
Plan and status: [`WEBSITE_ROADMAP.md`](WEBSITE_ROADMAP.md).

## Last session (2026-11-20)

Visual pass on the header and hero, from the user's feedback:

- Logo traced from the user's transparent PNG into `website/src/assets/logo.svg` (one
  even-odd path, `currentColor`, checked by overlay against the PNG). `icon-128.png` removed;
  `favicon.png` is still the app icon.
- Page starts yellow: header on `.on-brand`, no borders; uppercase wordmark; mono uppercase
  nav and hero label.
- Hero: crosshair removed; bottom edge is now a shallow symmetric V. A layered
  diagonal + blade + wedge version was built first and rejected by the user as ugly.
  Rules in `WEBSITE_DESIGN.md` "Header and hero".
- Reduced motion now also zeroes animation delays (staggered items were briefly invisible).

## State

- Phase 1 done, 2 mostly done, 3 partly (see roadmap). Pushed, not merged; Pages not enabled.
- `bun run build` then `bun run preview` → `http://localhost:4321/open-camera-mouse/`
  (`bunx astro preview stop` ends a server left running).
- Headless check used: Chrome `--headless=new --force-prefers-reduced-motion
  --window-size=W,H --blink-settings=preferredColorScheme=0|1 --screenshot=…`; for widths
  under ~500px load the page in a 320px `<iframe>` (Chrome won't make the window that narrow).

## Next

**Phase 4, OS-aware downloads** (roadmap). The header/hero pass is done and reviewed by the
user (yellow header, SVG logo, shallow V edge); only small polish if asked.

1. **Ask first:** the visitor's browser calling `api.github.com` (first third-party
   request). If yes, record it in ADR-0023 (amendment) and add privacy wording on the page.
2. Pure TypeScript module in `src/lib/` (OS detection without guessing CPU architecture,
   asset match by suffix from `src/lib/platforms.ts`, version from the tag) with `bun test`
   tests; add a `test` script and run it in `website.yml`.
3. Small client `<script>` wiring it to the hero "Download" button and the download
   section: latest-release lookup, `browser_download_url`, version shown, manual platform
   choice, unsupported-platform message, fallback to the Releases page on network error or
   rate limit. Everything must still work with JavaScript off (current static links).

## Waiting on the user

Hero copy ("Your head. Your mouse." is a draft); a real typewriter/display web font yes/no;
demo recording and screenshots.

## Not yet checked by hand

200% zoom, keyboard walk-through, VoiceOver, real reduced-motion setting, Windows/Linux
browsers (monospace stack falls back to Cascadia Mono/Consolas, Liberation Mono).
