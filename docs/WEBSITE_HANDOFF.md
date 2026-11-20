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
- Hero: crosshair removed; layered shape (diagonal edge, blade, gapped wedge from `lg`),
  rules in `WEBSITE_DESIGN.md` "Header and hero".
- Reduced motion now also zeroes animation delays (staggered items were briefly invisible).

## State

- Phase 1 done, 2 mostly done, 3 partly (see roadmap). Pushed, not merged; Pages not enabled.
- `bun run build` then `bun run preview` → `http://localhost:4321/open-camera-mouse/`
  (`bunx astro preview stop` ends a server left running).
- Headless check used: Chrome `--headless=new --force-prefers-reduced-motion
  --window-size=W,H --blink-settings=preferredColorScheme=0|1 --screenshot=…`; for widths
  under ~500px load the page in a 320px `<iframe>` (Chrome won't make the window that narrow).

## Next

1. **User review of the new header/hero** in the browser; adjust before moving on.
2. **Phase 4, OS-aware downloads:** pure module in `src/lib/` with `bun test` tests, reusing
   `src/lib/platforms.ts`; then the client script for the hero and download section.
   **Ask first:** the visitor's browser calling `api.github.com` (first third-party request):
   record it in ADR-0023 and the privacy wording before shipping.

## Waiting on the user

Hero copy ("Your head. Your mouse." is a draft); a real typewriter/display web font yes/no;
demo recording and screenshots.

## Not yet checked by hand

200% zoom, keyboard walk-through, VoiceOver, real reduced-motion setting, Windows/Linux
browsers (monospace stack falls back to Cascadia Mono/Consolas, Liberation Mono).
