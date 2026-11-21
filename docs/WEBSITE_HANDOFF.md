# Website handoff (temporary)

Session-to-session handoff for the website story on `feature/website`. Rewrite it at the end
of every website session. **Delete this file before merging to `main`** (roadmap Phase 6).
Plan and status: [`WEBSITE_ROADMAP.md`](WEBSITE_ROADMAP.md).

## Latest session (2026-11-21, tenth): engineering and open source sections

- **Engineering** (`#engineering`, between How it works and Download): four notes in a
  2-column grid (`border-t-4 border-text`, like the steps): independent Rust implementation
  (no OpenCV, starts from Camera Mouse 2018, new behaviour opt-in), tracking and stability
  (pyramidal LK, appearance check so a hand in front ends tracking, smoothing + steady zone),
  private by design (nothing leaves the computer, no accounts/cloud/telemetry, settings a
  text file), tested without a camera (synthetic seeded video with ground truth, CI on three
  OSes, Windows/Linux not yet on a device). Link "Architecture on GitHub →". Copy only from
  README/PRODUCT/ARCHITECTURE/TESTING/ADR-0018; data array at the top of `index.astro`.
  Wording and content get the user's review in one final pass, not per session.
- **Open source** expanded: licence line, then "Get involved" (source, issues, README
  `#develop`, `AGENTS.md`) and "Documentation" (manual, PRODUCT, TESTING, ADR folder) lists.
  Docs links point at `main` (`blob/main/docs/…`, `tree/main/…` for the ADR folder). There
  is no CONTRIBUTING file; "pull requests welcome" was not written because no doc says it.
- Verified: check 0/0/0, 34 tests, build + path check (672 URLs); headless screenshots of
  the home page at 1280 (Carbon and Paper) and the engineering section at 320.
- Preview servers: start your own on another port, and stop it by the `node … astro preview`
  pid from `lsof -iTCP:<port>` (the `bunx` wrapper pid exits early). Never touch `:4321`.

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
origin story (research with sources first), or the squirrel ear/cheek tweak if the user
wants it. README shortening waits for the deployment.
Release-triggered rebuild still only verifiable after merge.

## Waiting on the user

A real typewriter web font yes/no;
demo recording and screenshots.

## Not yet checked by hand

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
