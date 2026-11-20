# Website handoff (temporary)

Session-to-session handoff for the website story on `feature/website`. Rewrite it at the end
of every website session. **Delete this file before merging to `main`** (roadmap Phase 6).
Plan and status: [`WEBSITE_ROADMAP.md`](WEBSITE_ROADMAP.md).

## Latest session (2026-11-20, fifth): Phase 5 slice 1, manual infrastructure

- Content collection `manual` (`src/content.config.ts`, glob over `src/content/manual/*.md`,
  frontmatter `title`/`description`/`order`); `/manual/` contents and
  `/manual/[slug].astro` with sidebar (`ManualNav.astro`: `<details>` below `md`), "On this
  page" (h2s, shown when ≥ 2), prev/next. `src/lib/manual-order.ts` (sort, unique order,
  neighbours) with 5 `bun test` tests (17 total).
- Pages: getting started, system requirements (macOS 11 minimum from the packager config in
  `apps/desktop/Cargo.toml`), installation (per-OS sections; ids match
  `platforms.ts` `installAnchor`, renamed from `readmeAnchor`).
- Homepage download cards and lead link the manual instead of the README; header has a
  "Manual" link. `markdown.syntaxHighlight: false`; code blocks wrap (no sideways scroll box).
- `check-paths.mjs` now resolves relative links and checks `#fragment` ids; verified it
  fails on a misspelt page and a stale anchor.
- Verified: check 0/0/0, 17 tests, build + path check (86 URLs); headless Chrome: Carbon
  and Paper at 1280px, both pages at 320px.

## Earlier sessions (2026-11-20)

Fourth: header GitHub mark + night/day switch, user's hero copy. Third: OS-aware downloads
with release data at build time (ADR-0023 amendment 2), `release.ts`/`detect-platform.ts`,
release-triggered rebuild in `website.yml`.

## State

- Phase 1 done, 2 mostly, 3 partly, 4 mostly, 5 slice 1 of 4 (see roadmap). Pages not enabled.
- `bun run build` then `bun run preview` → `http://localhost:4321/open-camera-mouse/` (a
  preview from an older session may already run on `:4399`; `bunx astro preview stop`).
- Headless check: Chrome `--headless=new --force-prefers-reduced-motion
  --window-size=W,H --blink-settings=preferredColorScheme=0|1 --screenshot=…`; `#anchor`
  screenshots come out blank, take a tall window and crop; under ~500px use a 320px `<iframe>`.

## Next

**Phase 5 slice 2**: first-time setup and permissions, using the pointer, selecting a
tracking point, recentering, clicking and dwell clicking, mouse handover. Sources:
`PRODUCT.md`, `BEHAVIOR.md` and the app code (`apps/desktop`), not guesses; set `order`
after installation (4+). Release-triggered rebuild still only verifiable after merge.

## Waiting on the user

A real typewriter web font yes/no;
demo recording and screenshots.

## Not yet checked by hand

Manual: keyboard walk-through (sidebar, `<details>` at small widths, "On this page",
prev/next), VoiceOver read of the tables and `kbd`, 200% zoom.
200% zoom, keyboard walk-through (night/day switch click + Space/Enter, new file buttons, "Other platforms"),
VoiceOver (file buttons read "Download macOS Disk image (.dmg)"), real reduced-motion
setting, Windows/Linux browsers and a real phone (detection + unsupported message).
