# Website handoff (temporary)

Session-to-session handoff for the website story on `feature/website`. Rewrite it at the end
of every website session. **Delete this file before merging to `main`** (roadmap Phase 6).
Plan and status: [`WEBSITE_ROADMAP.md`](WEBSITE_ROADMAP.md).

## Latest session (2026-11-21, seventh): Phase 5 slice 3, reference pages

- Four manual pages, `order` 10–13: `settings` (every Options tab as a Setting/Range/Default
  table, Capture format / Keyboard / Mouse takeover as read-only, Restore defaults keeps
  camera + appearance + larger controls, file location per OS, every settings notice with
  its exact wording), `accessibility` (Larger controls sizes from `theme.rs`, themes,
  status without colour, fixed keys, known gaps: reduce-motion animation, screen readers
  untested), `troubleshooting` (won't open, camera, pointer by "Pointer:" owner, shaky /
  slow, lost target, dwell, keys, settings notice), `faq`.
- Sources: `options.rs`, `settings.rs`, `settings_store.rs` (notices), `app.rs` (save /
  apply notices, hotkey lines), `status.rs`, `theme.rs`, README "Known limitations".
- Defaults check: `src/lib/manual-defaults.ts` + `.test.ts` parse `settings.rs` (`ranges`,
  `impl Default`) and compare every `| Setting | Range | Default |` row on every manual
  page; the Settings page must list every setting. Shown to fail on a changed default.
  `website.yml` now also triggers on `crates/app-core/src/settings.rs`.
- FAQ has one origin line ("independent … inspired by the classic Camera Mouse … not
  affiliated or endorsed"), no historical claims; approved by the user (2026-11-21).
- Verified: check 0/0/0, 22 tests, build + path check (587 URLs and anchors); headless
  Chrome: `settings` at 320px (Carbon); the notices became a list (a table was cramped).

## Earlier sessions

Sixth (2026-11-20): everyday-use manual pages (setup, tracking point, pointer, recentering,
clicking, handover); header back to one row, skip link removed. Fifth: manual
infrastructure, getting started, requirements, installation. Fourth: header GitHub mark +
night/day switch, hero copy. Third: OS-aware downloads with release data at build time.

## State

- Phase 1 done, 2 mostly, 3 partly, 4 mostly, 5 slices 1–3 of 4, README part of 3 waits for deployment (see roadmap). Pages not enabled.
- `bun run build` then `bun run preview` → `http://localhost:4321/open-camera-mouse/` (a
  preview from an older session may already run on `:4321`; `bunx astro preview stop`).
- Headless check: Chrome `--headless=new --force-prefers-reduced-motion
  --window-size=W,H --blink-settings=preferredColorScheme=0|1 --screenshot=…`; `#anchor`
  screenshots come out blank, take a tall window and crop; under ~500px use a 320px `<iframe>`.

## Next

Phase 5 slice 4 needs the user's screenshots; until then pick from the roadmap: Phase 2
footer design, or Phase 3 features / engineering sections (screenshots again) or the
origin story (research with sources first). README shortening waits for the deployment.
Release-triggered rebuild still only verifiable after merge.

## Waiting on the user

A real typewriter web font yes/no;
demo recording and screenshots.

## Not yet checked by hand

Manual (all thirteen pages): keyboard walk-through (sidebar, `<details>` at small widths, "On this page",
prev/next), VoiceOver read of the tables and `kbd`, 200% zoom.
200% zoom, keyboard walk-through (night/day switch click + Space/Enter, new file buttons, "Other platforms"),
VoiceOver (file buttons read "Download macOS Disk image (.dmg)"), real reduced-motion
setting, Windows/Linux browsers and a real phone (detection + unsupported message).
