# Website handoff (temporary)

Session-to-session handoff for the website story on `feature/website`. Rewrite it at the end
of every website session. **Delete this file before merging to `main`** (roadmap Phase 6).
Plan and status: [`WEBSITE_ROADMAP.md`](WEBSITE_ROADMAP.md).

## Latest session (2026-11-20, sixth): Phase 5 slice 2, using the app

- Six manual pages, `order` 4–9: `first-time-setup` (first launch, camera and pointer
  permission states, camera error cards, key check), `tracking-point` (picking, good spots,
  marker/status table, lost and "Find the target again"), `using-the-pointer` (mapping,
  pause, speed/smoothing/limits/main monitor with ranges and defaults), `recentering`,
  `clicking` (dwell, when it doesn't click, settings table), `mouse-handover`.
- Every statement checked against the code: wording from `apps/desktop/src/status.rs`,
  `options.rs`, `preview.rs`; behaviour from `crates/app-core/src/controller.rs` (pause →
  pointer home, resume → recenter, `take_over` moves the pointer to the main monitor's
  centre, re-acquire jump bound) and `pointer-core` (`dwell.rs`, `handover.rs`: 5 px, 4 s,
  timer restarts on movement). Ranges from `app-core` `settings::ranges`.
- Getting started links the new pages. Hero h1 lost its full stop (user's request).
- Header: a nav row + hamburger overlay was tried and dropped (user, 2026-11-21); header
  is back to the committed one (Manual link, GitHub, theme switch). The manual is linked
  from "How it works" ("Read the manual →") and "Open source". Below `sm` the header is one
  row: "OCM" wordmark and a book icon for Manual (full names kept for screen readers). Skip link removed (user's
  decision; docs and the `web-accessibility` skill updated).
- Verified: check 0/0/0, 17 tests, build + path check (325 URLs and anchors); headless
  Chrome: `tracking-point` at 320px (Carbon), home hero at 1280px (Paper).

## Earlier sessions (2026-11-20)

Fifth: manual infrastructure (collection, sidebar, "On this page", prev/next, anchor
checks), getting started, requirements, installation. Fourth: header GitHub mark +
night/day switch, user's hero copy. Third: OS-aware downloads with release data at build
time (ADR-0023 amendment 2).

## State

- Phase 1 done, 2 mostly, 3 partly, 4 mostly, 5 slices 1–2 of 4 (see roadmap). Pages not enabled.
- `bun run build` then `bun run preview` → `http://localhost:4321/open-camera-mouse/` (a
  preview from an older session may already run on `:4399`; `bunx astro preview stop`).
- Headless check: Chrome `--headless=new --force-prefers-reduced-motion
  --window-size=W,H --blink-settings=preferredColorScheme=0|1 --screenshot=…`; `#anchor`
  screenshots come out blank, take a tall window and crop; under ~500px use a 320px `<iframe>`.

## Next

**Phase 5 slice 3**: settings reference (all Options tabs incl. Camera "Capture format",
General "Show diagnostics", "Restore defaults" keeps camera and appearance, where settings
are saved and the bad-value/corrupt-file notices), accessibility options (Larger controls,
themes), troubleshooting, FAQ, a check that manual defaults match the app's
(`Settings::default()`); README shortened to link the manual only after deployment.
Release-triggered rebuild still only verifiable after merge.

## Waiting on the user

A real typewriter web font yes/no;
demo recording and screenshots.

## Not yet checked by hand

Manual (all nine pages): keyboard walk-through (sidebar, `<details>` at small widths, "On this page",
prev/next), VoiceOver read of the tables and `kbd`, 200% zoom.
200% zoom, keyboard walk-through (night/day switch click + Space/Enter, new file buttons, "Other platforms"),
VoiceOver (file buttons read "Download macOS Disk image (.dmg)"), real reduced-motion
setting, Windows/Linux browsers and a real phone (detection + unsupported message).
