# Website handoff (temporary)

Session-to-session handoff for the website story on `feature/website`. Rewrite it at the end
of every website session. **Delete this file before merging to `main`** (roadmap Phase 6).
Plan and status: [`WEBSITE_ROADMAP.md`](WEBSITE_ROADMAP.md).

## Latest session (2026-11-21, twelfth): automated accessibility and performance audit

- Lighthouse 12.6.1 (installed in the session scratchpad, not the repo) against the
  preview: home, manual index, settings, troubleshooting, 404, mobile and desktop presets,
  all 100 in accessibility, performance, best practices and SEO; 11–17 KiB per page, CLS 0.
- Full axe-core 4.14 (WCAG 2.0–2.2 A/AA + best practice) on all 16 pages × Carbon/Paper ×
  320/768/1280px, `<details>` opened, reduced motion: one violation, an empty first header
  cell in the "Larger controls" table (`manual/accessibility.md`), now "Element"; new
  `manual-tables.test.ts` fails on any empty manual table header (seen failing before the
  fix). Remaining "incomplete": the `aria-hidden` `→` arrows (false positives). No
  horizontal overflow at any width. How to re-run: `WEBSITE.md` "Local development".
- Verified: check 0/0/0, 50 tests, build + path check (672 URLs).

### Eleventh session (2026-11-21): origin story

- Why it exists (`#why-it-exists`) with three cited sources (`originSources` in
  `index.astro`); what was left out is in `WEBSITE.md` "Origin story". Wording goes into
  the user's final copy pass.
- Astro 7's `astro preview` reuses an already-running preview server (one was on `:4330`).

### Tenth session (2026-11-21): engineering and open source sections

- Engineering (`#engineering`, four notes) and the expanded Open source section (licence,
  "Get involved" and "Documentation" link lists; docs links point at `main`). No
  CONTRIBUTING file, so no "pull requests welcome".

### Ninth session (2026-11-21): footer, squirrel redraw

- Yellow footer with a Λ top edge (`SiteFooter.astro`, `.angular-top`), committed by the user.
- Squirrel redrawn cuter and faceted, black ears and tail facets. Open: may read a little
  cat-like; offered shorter tufted ears and puffier cheeks.
- Gotcha: in Astro, `{...line} fill="currentColor"` renders two `fill` attributes and the
  parser keeps the first; merge into the spread (`{...{ ...line, fill: "currentColor" }}`).

## Earlier sessions

Eighth (2026-11-21): hero reload flicker fix, home page FAQ (`#faq`), hero squirrel.
Seventh (2026-11-21): manual reference pages (settings, accessibility, troubleshooting, FAQ)
and the defaults check (`manual-defaults.test.ts`).
Sixth (2026-11-20): everyday-use manual pages (setup, tracking point, pointer, recentering,
clicking, handover); header back to one row, skip link removed. Fifth: manual
infrastructure, getting started, requirements, installation. Fourth: header GitHub mark +
night/day switch, hero copy. Third: OS-aware downloads with release data at build time.

## State

- Phase 1 done, 2 mostly, 3 partly, 4 mostly, 5 slices 1–3 of 4, README part of 3 waits for deployment (see roadmap). Pages not enabled.
- `bun run build` then `bun run preview` → `http://localhost:4321/open-camera-mouse/` (a
  preview may already run; leave it alone and use another port if `:4321` is taken).
- Headless check: Chrome `--headless=new --force-prefers-reduced-motion
  --window-size=W,H --blink-settings=preferredColorScheme=0|1 --screenshot=…`; `#anchor`
  screenshots come out blank, take a tall window and crop; under ~500px use a 320px `<iframe>`.
- Load flicker / pointer checks: a throwaway Bun script driving Chrome over CDP
  (`--remote-debugging-port`, Bun's built-in WebSocket; `Page.startScreencast` for frames,
  `Input.dispatchMouseEvent` for the pointer). Not in the repo; rewrite when needed.

## Next

Phase 5 slice 4 and the Features section need the user's screenshots. Without them only
visual items remain, and they need the user's OK first: the squirrel ear/cheek tweak, hero
geometry/section accents (Phase 2), the header edge on other pages. README shortening waits for the deployment.
Release-triggered rebuild still only verifiable after merge.

## Waiting on the user

A real typewriter web font yes/no;
demo recording and screenshots.

## Not yet checked by hand

Automated audit is done (above); what follows needs a person.

Why it exists: keyboard walk-through of the source links, VoiceOver, 200% zoom, 768px;
the three source URLs from a normal browser (checked by fetch only).
Engineering and Open source sections: keyboard walk-through, VoiceOver (the two `h3` link
lists), 200% zoom, 768px.
Squirrel redraw with a real pointer (tilt/pupils/tail with the new shapes).
Footer: keyboard walk-through, VoiceOver (footer nav and list labels), 200% zoom, 768px.
Hero squirrel in a real browser with a real mouse and head pointer (feel of the motion, CPU
while moving); the reload fix in Safari/Firefox.
Manual (all thirteen pages): keyboard walk-through (sidebar, `<details>` at small widths, "On this page",
prev/next), VoiceOver read of the tables and `kbd`, 200% zoom.
200% zoom, keyboard walk-through (night/day switch click + Space/Enter, new file buttons, "Other platforms"),
VoiceOver (file buttons read "Download macOS Disk image (.dmg)"), real reduced-motion
setting, Windows/Linux browsers and a real phone (detection + unsupported message).
