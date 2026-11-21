# Website handoff (temporary)

Session-to-session handoff for the website story on `feature/website`. Rewrite it at the end
of every website session. **Delete this file before merging to `main`** (roadmap Phase 6).
Plan and status: [`WEBSITE_ROADMAP.md`](WEBSITE_ROADMAP.md).

## Latest session (2026-11-21, eleventh): origin story

- **Why it exists** (`#why-it-exists`, between the showcase and How it works): three
  paragraphs (classic Camera Mouse at Boston College by Gips and Betke from EagleEyes; free
  download since June 2007, 3M+ downloads by 2016, who used it, Gips died 2018; Open Camera
  Mouse as an independent from-scratch successor, not affiliated with or endorsed by BC or
  the authors), then a numbered "Sources" list (`originSources` in `index.astro`): the 2002
  IEEE TNSRE paper (DOI), BC News 2016, BC News In Memoriam 2018. Details and what was
  deliberately left out (current availability, licence, platforms of the classic program:
  `cameramouse.org` refused connections, only third-party mirrors list Camera Mouse 2018) are
  in `WEBSITE.md` "Origin story". Wording goes into the user's final copy pass.
- Verified: check 0/0/0, 34 tests, build + path check (672 URLs); headless screenshots at
  1280 Carbon and 320 Paper.
- Preview: Astro 7's `astro preview` reuses an already-running preview server (one was on
  `:4330`) and exits instead of starting a second; use that one (`astro preview status`).

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

Phase 5 slice 4 and the Features section need the user's screenshots; without them: the
squirrel ear/cheek tweak if the user wants it, or hero geometry/section accents (Phase 2). README shortening waits for the deployment.
Release-triggered rebuild still only verifiable after merge.

## Waiting on the user

A real typewriter web font yes/no;
demo recording and screenshots.

## Not yet checked by hand

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
