# Website handoff (temporary)

Session-to-session handoff for the website story on `feature/website`. Rewrite it at the end
of every website session. **Delete this file before merging to `main`** (roadmap Phase 6).
Plan and status: [`WEBSITE_ROADMAP.md`](WEBSITE_ROADMAP.md).

## Last session (2026-11-20, third): Phase 4 OS-aware downloads

- **Decision (user):** release data at **build time**, not from the visitor's browser
  (ADR-0023 amendment 2). The site still makes no third-party request, so no privacy note.
- `src/lib/release.ts` (API JSON → version + per-platform files, strict name/URL match) and
  `src/lib/detect-platform.ts` (OS only; phones/tablets/ChromeOS = unsupported; iPad
  desktop mode via `maxTouchPoints`), 12 `bun test` tests; `bun run test` script, in CI.
  Test files are excluded from `astro check` (typing `bun:test` would need `@types/bun`).
- `src/lib/latest-release.ts`: reads `src/data/latest-release.json` (gitignored, written by a
  `gh api` step in `website.yml`) via `import.meta.glob` (no Node types in the project);
  otherwise anonymous fetch; failure → `null` → old "Go to the latest release" fallback.
- Hero: static "Version X for macOS, Windows and Linux."; with JS, "Download for <OS>"
  pointing at the file, with the requirement and "Other systems". Cards: outlined
  per-file buttons, "Your system" label, unsupported-device message. `DownloadEnhancer.astro`
  is the only client script added (inlined by Astro).
- `website.yml`: `release: published` → `rebuild` job dispatches the workflow on `main`
  (tag refs can't deploy to `github-pages`); skipped until `WEBSITE_DEPLOY` is `true`.
- Verified: check 0/0/0, 12 tests, build + path check with live API, with the CI file, and
  with a bad file (fallback); headless Chrome on macOS: hero links the `.dmg`, "Your system"
  on macOS, Paper download section, 320px both columns fine. A headless iPhone UA still
  showed macOS because Chrome keeps the UA-CH platform; covered by unit tests instead.

## State

- Phase 1 done, 2 mostly, 3 partly, 4 mostly (see roadmap). Pages not enabled.
- `bun run build` then `bun run preview` → `http://localhost:4321/open-camera-mouse/` (a
  preview from an older session may already run on `:4399`; `bunx astro preview stop`).
- Headless check: Chrome `--headless=new --force-prefers-reduced-motion
  --window-size=W,H --blink-settings=preferredColorScheme=0|1 --screenshot=…`; `#anchor`
  screenshots come out blank, take a tall window and crop; under ~500px use a 320px `<iframe>`.

## Next

Phase 4 leftovers are deferred (download page → Phase 5; release-triggered rebuild can only
be verified after merge). Suggested next: **Phase 5, user manual** (content collection,
install steps per OS incl. what the `/download/` page would have held), or Phase 3 content
if the user has screenshots.

## Waiting on the user

Hero copy ("Your head. Your mouse." is a draft); a real typewriter web font yes/no;
demo recording and screenshots.

## Not yet checked by hand

200% zoom, keyboard walk-through (theme switch, new file buttons, "Other systems"),
VoiceOver (file buttons read "Download macOS Disk image (.dmg)"), real reduced-motion
setting, Windows/Linux browsers and a real phone (detection + unsupported message).
