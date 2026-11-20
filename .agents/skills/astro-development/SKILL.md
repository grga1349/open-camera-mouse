---
name: astro-development
description: Astro 7 static site in website/ — component conventions, Tailwind v4 CSS-first theme, base-path-safe URLs for the GitHub Pages project site, zero-JS by default, Bun build and verification. Use for any change under website/ or .github/workflows/website.yml. Not for visual decisions (see website-design) or accessibility rules (see web-accessibility); not for the egui app.
---

## Objective

Keep `website/` a static, dependency-light Astro site that builds identically locally and in
CI and works under `https://grga1349.github.io/open-camera-mouse/`. Read `docs/WEBSITE.md`
and ADR-0023 first.

## When to use

Pages, layouts, components, styles, assets, scripts or the website workflow.

## When not to use

Brand, layout and token choices → `website-design`. Rust/egui UI → `egui-development`.

## Rules

- **Static only.** `output: "static"`; no adapters, SSR, API routes, `server:` islands or
  `client:*` framework islands. No React/Vue/Svelte/Alpine, component or icon libraries, CMS,
  web fonts, analytics. A new package needs an ADR-0023 amendment.
- **Components:** `src/components/PascalCase.astro`, one per file, typed `interface Props`
  in the frontmatter, `<slot />` for content. Pages in `src/pages/` are routes; shared page
  chrome lives only in `src/layouts/BaseLayout.astro` (it takes `title?`, a required
  `description`, and `fullBleed` for pages built from full-width sections). Shared data in
  `src/lib/*.ts` (e.g. `platforms.ts`), not in inline `<script>`s.
- **Markdown content** (the manual, Phase 5): Astro's built-in content collections
  (`src/content.config.ts`, `glob` loader over `src/content/<name>/*.md`, a zod schema),
  rendered by a `[...slug].astro` page with `getStaticPaths`. No docs framework.
- **Tests** for pure logic in `src/lib/`: `bun test` (built in; `*.test.ts` next to the
  module), no test-framework package. Keep DOM wiring thin and separate from the logic.
- **URLs:** never write a root-relative `"/…"` string. Internal pages and `public/` files go
  through `url()` from `src/lib/url.ts` (`url("/")`, `url("docs/")`, `url("favicon.png")`);
  page paths end in `/` (`trailingSlash: "always"`). Images processed by Astro
  (`import img from "../assets/x.png"` + `<Image>`) get the base automatically. External
  links use full `https://` URLs.
- **Whitespace:** Astro's HTML compression removes a line break between text and a tag,
  so `the\n<a>` renders "the<a>". Keep the space on the same line as the text
  (`the <a …`), breaking lines inside the tag instead; check `dist/` for glued words.
- **Tailwind v4:** CSS-first. `src/styles/global.css` holds `@import "tailwindcss"`, the
  Carbon/Paper custom properties and one `@theme inline` block; there is no
  `tailwind.config.*`. The default palette is removed (`--color-*: initial`), so only token
  utilities exist (`bg-bg`, `bg-surface`, `text-text`, `text-muted`, `border-line`,
  `bg-yellow text-on-yellow`, `rounded-frame`/`rounded-widget`). New tokens go in
  `@theme` and in `docs/WEBSITE_DESIGN.md` together. Prefer utilities; use `@layer base`
  for element defaults, not ad-hoc `<style>` per page.
- **Build-time data:** external data (the latest release) is read in the frontmatter at
  build time through `src/lib/` and baked into HTML; the visitor's browser makes no
  third-party request (ADR-0023 amendment 2). Secrets/tokens stay in CI steps, not the build.
- **JavaScript:** ship none for static UI. Use native elements first (`<details>`, `<dialog>`,
  anchors, `:target`, CSS). When behaviour really needs JS, use a plain `<script>` in the
  component (Astro bundles it as a module), progressive enhancement only: the content and
  links must work with JS off. No inline event handler attributes, no `is:inline` unless
  there's a measured reason.
- **Assets:** static files in `website/public/` (copied as-is, referenced via `url()`).
  The app icon there is a copy of `assets/icon/png/`; recopy when the icon changes.

## Deliverables and gates

From `website/` (Bun ≥ 1.3, version pinned by `packageManager` in `package.json`):

```sh
bun install --frozen-lockfile   # lockfile must not change unless deps changed on purpose
bun run check                   # astro check: types + .astro diagnostics, 0 errors/warnings
bun run test                    # bun test for src/lib/*.test.ts (excluded from astro check)
bun run build                   # astro build + scripts/check-paths.mjs (base path, dead links)
bun run preview                 # optional: http://localhost:4321/open-camera-mouse/
```

Then inspect `dist/`: no `.js` files unless the change added a script on purpose, every
internal URL starts with `/open-camera-mouse/`, text reads correctly. Commit `bun.lock`
with any dependency change; never commit `dist/`, `.astro/` or `node_modules/`.
