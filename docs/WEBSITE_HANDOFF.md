# Website handoff (temporary)

Session-to-session handoff for the website story on `feature/website`. Rewrite it at the end
of every website session. **Delete this file before merging to `main`** (roadmap Phase 6).
Plan and status: [`WEBSITE_ROADMAP.md`](WEBSITE_ROADMAP.md).

## Latest session (2026-11-21, ninth): footer, squirrel redraw

- **Footer** (`SiteFooter.astro`, `.angular-top` in `global.css`): the yellow brand surface
  again with a shallow Λ top edge mirroring the hero's V, so every page starts and ends
  yellow. Logo + wordmark, the hero's one-liner, a "Footer" nav with Manual (getting started,
  installation, troubleshooting, questions) and Project (download, all releases, source
  code, report a problem) lists, a 2px black rule, licence line, "No cookies, no analytics."
  No script. The user committed it without comment on the direction or the "No cookies"
  line; treat both as accepted unless they say otherwise.
- **Squirrel redrawn "more cute and angular"** (user's request), then "ears all black and
  some black in the tail": faceted outlines (straight edges, obtuse angles, round joins), big
  octagonal head, chubby body, solid black triangle ears, eyes with a shine, blush slashes,
  "w" mouth, hexagonal acorn, trapezoid feet in front, bushy faceted tail with a black tip and
  a black band. Pose maths unchanged except `SMALL_REACH` 5 and `NECK` y 236. Open: it can
  read a little cat-like (octagon head + pointed ears); offered shorter tufted ears and
  puffier cheeks if the user agrees.
- **Gotcha:** in Astro, `{...line} fill="currentColor"` renders two `fill` attributes and the
  HTML parser keeps the first (`none`); merge into the spread instead
  (`{...{ ...line, fill: "currentColor" }}`). The nose had this bug unnoticed.
- Verified: check 0/0/0, 34 tests, build + path check (672 URLs); footer headless screenshots
  at 1280 (both themes, home and 404) and 320; squirrel as zoomed stills only.
- **Preview servers:** don't run `bunx astro preview stop` or kill by port: it stopped the
  user's preview this session. Start your own (`bunx astro preview --port 4321` in the
  background), note its pid, and stop only that pid.

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

Phase 5 slice 4 needs the user's screenshots; until then pick from the roadmap: Phase 3 features / engineering sections (screenshots again) or the
origin story (research with sources first). README shortening waits for the deployment.
Release-triggered rebuild still only verifiable after merge.

## Waiting on the user

A real typewriter web font yes/no;
demo recording and screenshots.

## Not yet checked by hand

Squirrel redraw with a real pointer (tilt/pupils/tail with the new shapes).
Footer: keyboard walk-through, VoiceOver (footer nav and list labels), 200% zoom, 768px.
Hero squirrel in a real browser with a real mouse and head pointer (feel of the motion, CPU
while moving); the reload fix in Safari/Firefox.
Manual (all thirteen pages): keyboard walk-through (sidebar, `<details>` at small widths, "On this page",
prev/next), VoiceOver read of the tables and `kbd`, 200% zoom.
200% zoom, keyboard walk-through (night/day switch click + Space/Enter, new file buttons, "Other platforms"),
VoiceOver (file buttons read "Download macOS Disk image (.dmg)"), real reduced-motion
setting, Windows/Linux browsers and a real phone (detection + unsupported message).
