# Website handoff (temporary)

Session-to-session handoff for the website story on `feature/website`. Rewrite it at the end
of every website session. **Delete this file before merging to `main`** (roadmap Phase 6).
Plan and status: [`WEBSITE_ROADMAP.md`](WEBSITE_ROADMAP.md).

## Latest session (2026-11-21, fourteenth): video play button

- `Showcase.astro`: a dark scrim over the video with a white play ring (circle outline + triangle) in the centre (the first yellow version was rejected by the user; tokens `scrim`/`on-scrim` added). The `<button>` covers
  the whole video (large dwell target) and ships `hidden`; its script hides the native
  controls and shows it. Click: controls back, focus to the video, `play()`. On `ended` the
  button returns. Without JS: native controls as before.
- Verified: check 0/0/0, 50 tests, build + path check; headless screenshots in both themes.
  Not checked by hand: clicking it in a real browser, keyboard, VoiceOver, 320px.

## Previous session (2026-11-21, thirteenth): decisions, hero, demo video, copy pass

All committed and pushed (`ec2dcb0`, `1345554`, `c129875`, `6818d7e`).

- **User decisions:**
  - Manual and other pages keep the simple flat yellow header.
  - The squirrel stays as drawn.
  - Footer Λ stays centred.
  - No screenshots, so the Features section and the annotated manual are dropped.
  - Keep the system monospace typewriter, no web font.
  - Checks by hand happen during use.
  - Never say on the site that the Windows or Linux builds haven't been tried on a real
    device (rule in `WEBSITE_DESIGN.md` "Constraints").
- **Hero:**
  - The V's point is now at `--apex` (30% from the left, in `:root` next to `--cut`).
  - From `lg` up the headline is two lines ("Computing without" / "barriers",
    `lg:text-6xl lg:max-w-none`), so the hero is about 90px shorter. Other operating
    systems' monospace fonts may still wrap it to three lines.
- **Demo video** (`Showcase.astro`, `public/showcase/demo.mp4` + `poster.jpg`):
  - Source: the user's own screen recording, cut at 23.0 s where camera tracking ends,
    1920×1200, 30 fps, x264 CRF 24, no audio, faststart, 1.6 MB. The original stays on
    their Desktop, not in the repo.
  - Timeline: paused 0–2 s, recentering countdown 2–7 s, tracking 7–23 s.
  - `preload="metadata"`, no autoplay.
  - No captions and no visible description (user's call). An `sr-only` figcaption is the
    text alternative.
- **Copy pass** (user's brief) over the home page, footer, download notes, manual index and
  all 13 manual pages. The hero copy is unchanged.
  - Origin story is down to 3 paragraphs and 2 sources. Nothing on the site says the
    classic app's development stopped: no source was found for it.
  - Engineering is 4 short notes: built from scratch, precise control, tested on every
    change, private.
  - The FAQ privacy answer mentions the opt-in local diagnostic recording (`OCM_RECORD`).
  - Manual terminology: "point" (not "spot"), and the "Searching" animation.
  - The macOS `xattr` command moved from Installation to Troubleshooting.
- **Verified:** check 0/0/0, 50 tests, build + path check (676 URLs); headless screenshots
  of the hero and the video.

## Earlier sessions

Twelfth (2026-11-21): automated audit:
- Lighthouse 100 in all four categories, mobile and desktop.
- axe on all pages × both themes × 320/768/1280px: one empty table header fixed, and
  `manual-tables.test.ts` now guards it.
- How to re-run both: `WEBSITE.md` "Local development".

Eleventh: origin story with sources. Tenth: engineering and open source sections. Ninth:
footer with Λ edge, squirrel redraw.
- Gotcha from the ninth session: `{...line} fill=…` renders two `fill` attributes; merge
  into the spread instead.

Eighth: hero reload flicker fix, home FAQ, squirrel. Seventh: manual reference pages and
`manual-defaults.test.ts`. Sixth: everyday-use manual pages. Fifth: manual infrastructure.
Fourth: header GitHub mark and theme switch. Third: OS-aware downloads with release data at
build time.

## State

- Content is done, as far as the user's decisions go. Pages is not enabled and nothing is
  merged.
- **Preview:** a preview server already runs on `http://localhost:4330/open-camera-mouse/`
  (Astro 7's `astro preview` reuses it and won't start another). It serves `dist/`, so run
  `bun run build` and reload. Stop it with `bunx astro preview stop` from `website/`.
- **Headless check:** Chrome `--headless=new --force-prefers-reduced-motion
  --window-size=W,H --blink-settings=preferredColorScheme=0|1 --screenshot=…`.
  - `#anchor` screenshots come out blank: take a tall window and crop.
  - Under ~500px, use a 320px `<iframe>`.
- **Video tools:** `ffmpeg` (Homebrew) has no `drawtext` filter. Make contact sheets with
  `fps=…,scale=…` then `tile=`.

## Next

1. Re-run Lighthouse and axe now that the video is in. The tooling isn't in the repo:
   reinstall it in the scratchpad.
2. Merge prep (Phase 6):
   - delete this file and the references to it (`WEBSITE_ROADMAP.md`, `AGENTS.md`),
   - mark ADR-0023 Accepted,
   - with the user's OK, merge to `main`.
3. The user turns on Pages (source: GitHub Actions) and sets `WEBSITE_DEPLOY=true`.
4. After the first deployment:
   - shorten the README to link to the manual,
   - verify that publishing a release rebuilds the site.

Optional, only with the user's OK: section transitions and accents (Phase 2).

## Waiting on the user

- OK to merge.
- Their own read of the copy (later).

## Not yet checked by hand (user: during use)

- Keyboard walk-through, VoiceOver, 200% zoom, a real phone, Safari and Firefox for the
  whole site.
- In particular:
  - the asymmetric hero V at 320px,
  - two-line headline wrapping on Windows and Linux,
  - video playback,
  - the squirrel with a real head pointer,
  - OS detection on Windows and Linux.
