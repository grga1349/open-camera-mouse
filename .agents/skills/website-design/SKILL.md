---
name: website-design
description: Visual design of the Open Camera Mouse website — brand identity shared with the desktop app (monochrome, Carbon/Paper, two accents), design tokens, typography, spacing, responsive layout, content rules. Use when designing or changing how website pages look or what they say. Not for Astro mechanics (see astro-development) or accessibility rules (see web-accessibility; it overrides design on conflict).
---

## Objective

The site should read as the same product as the desktop app: quiet, monochrome, practical.
Source of truth: `docs/WEBSITE_DESIGN.md`, which derives from `docs/PRODUCT.md` ("UI
design", "Design tokens") and `docs/design-lab.html`. Changing the visual direction needs
the user's OK.

## When to use

New pages or sections, layout, typography, imagery, copy tone, token changes.

## When not to use

Component/build mechanics → `astro-development`. Contrast, focus, motion → `web-accessibility`.

## Rules

- **Colour:** only the token utilities (`bg-bg`, `bg-surface`, `bg-surface-2`, `border-line`,
  `border-line-strong`, `text-text`, `text-muted`). Two accents: `marker` (`#00ff00`) only
  for the target-ring motif; `yellow` as the fill of the one primary call to action per
  view (with `text-on-yellow`), and the fixed `brand` yellow as the hero surface
  (`.on-brand`, black content, black focus outline). No other hues, gradients or tinted
  greys.
- **Shape and motion:** angular accents (diagonals, wedges; `.angular-bottom`), decorative
  only. Motion per `WEBSITE_DESIGN.md` "Motion": one-time CSS entrances and small hover
  lifts, nothing looping, nothing that moves a control under the pointer. Colour photos/screenshots of the app are fine (the app's preview is in
  colour); chrome stays monochrome.
- **Themes:** Carbon (dark) and Paper (light) values from `docs/PRODUCT.md`, selected by
  `prefers-color-scheme`. Check every change in both.
- **Type:** the system stack (`font-sans`); no web fonts. Body ≥ 16px (`text-base`), help
  text `text-sm text-muted`, line length `max-w-prose`. Headings by size and weight
  (`font-semibold`), not colour. One `h1` per page.
- **Spacing and shape:** Tailwind's 4px scale; content column `max-w-5xl` with `px-4`
  (`sm:px-6`); vertical rhythm in steps of `py-12`/`mt-4`. Radius `rounded-frame` (8px)
  for panels and images, `rounded-widget` (6px) for buttons and inputs. 1px `border-line`
  separators, no shadows. Homepage sections use `Section.astro`.
- **Responsive:** mobile first, single column at 320px with no horizontal scroll; add
  columns at `sm`/`md`/`lg` only. Text stays readable at 200% zoom and 400% reflow.
  Images get `width`/`height` to avoid layout shift.
- **Logo:** the app icon (`website/public/icon-128.png`, from `assets/icon/`); next to the
  wordmark it is decorative (`alt=""`). Don't redraw, recolour or crop it.
- **Content:** state only what `README.md` and `docs/PRODUCT.md` support. No invented
  features, platform or compatibility claims, download counts, testimonials, release dates
  or medical claims (a non-goal in PRODUCT). Link to the README or the release page
  instead of copying install steps. Show real numbers, as the app does.

## Deliverables and gates

Tokens changed in `src/styles/global.css` and `docs/WEBSITE_DESIGN.md` together; both
themes and widths 320/768/1280px checked by hand (say so in the report);
`astro-development` and `web-accessibility` gates pass.
