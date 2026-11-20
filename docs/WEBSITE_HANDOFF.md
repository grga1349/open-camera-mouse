# Website handoff (temporary)

Session-to-session handoff for the website story on `feature/website`. Rewrite it at the end
of every website session. **Delete this file before merging to `main`** (roadmap Phase 6).
Plan and status: [`WEBSITE_ROADMAP.md`](WEBSITE_ROADMAP.md).

- **State:** Phases 1–2 first pass on `feature/website`, pushed, not merged; Pages not enabled.
  The homepage builds (`bun run build`) and previews at
  `http://localhost:4321/open-camera-mouse/` (`bun run preview`; `bunx astro preview stop`
  ends a server left running).
- **Next task:** Phase 4, OS-aware downloads. Start with the pure module in `src/lib/` and
  its `bun test` tests, reusing `src/lib/platforms.ts`; then the client script for hero and
  download section.
- **Decide first (ask the user):** the visitor's browser calling `api.github.com` (first
  third-party request): record it in ADR-0023 and the privacy wording before shipping.
- **Waiting on the user:** hero copy ("Your head. Your mouse." is a draft), display
  typeface yes/no, demo recording and screenshots.
- **Not yet checked by hand:** 768px, 200% zoom, keyboard walk-through, VoiceOver,
  reduced motion.
