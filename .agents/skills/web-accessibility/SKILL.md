---
name: web-accessibility
description: WCAG 2.2 AA for the website — semantic HTML, landmarks, keyboard and visible focus, screen readers, contrast, target size, reduced motion, accessible patterns for native disclosure/dialog/navigation. Use for every change to website markup, styles or scripts. Wins over design on conflict. Not for the egui app's AccessKit labels (see egui-development).
---

## Objective

The users of this product often can't use a mouse; many navigate the web with a head
pointer, switch access, keyboard or a screen reader. The site must meet WCAG 2.2 AA with no
exceptions, and should be easy to use with dwell clicking (large, stable targets).

## When to use

Any markup, style, interaction or content change under `website/`.

## When not to use

Desktop app accessibility → `egui-development` and `docs/PRODUCT.md`.

## Rules

- **Structure:** `<html lang="en">`; one `h1`, headings in order with no skipped levels;
  landmarks `header`, `nav` (with `aria-label` when there's more than one), `main`
  (`id="main"`), `footer`. No skip link (user's decision, 2026-11-20): keep the
  header's tab stops few. Lists for lists, `<table>`
  with `<th scope>` for tables. Unique, descriptive `<title>` per page.
- **Native first:** links navigate, buttons act (`<button type="button">`); never a
  clickable `div`/`span`. Disclosure → `<details><summary>`; modal → `<dialog>` opened with
  `showModal()` (focus moves in, Esc closes, focus returns). Add ARIA only where no native
  element exists, and then follow the APG pattern exactly. No `tabindex` > 0.
- **Keyboard:** everything reachable and operable with Tab/Shift+Tab/Enter/Space/Esc, in
  visual order, no traps. Focus stays visible: the global `:focus-visible` outline (2px,
  `text` colour, 2px offset) must not be removed or hidden under sticky elements
  (2.4.11 Focus Not Obscured).
- **Targets:** at least 24×24 CSS px (2.5.8); aim for 44×44 (`min-h-11`) on navigation and
  calls to action, with space between adjacent targets, since dwell clicking and head
  pointers are imprecise. Nothing that activates on hover alone.
- **Contrast:** text 4.5:1 (3:1 for ≥ 24px or ≥ 18.66px bold); focus indicators, control
  borders and meaningful icons 3:1 against neighbours (1.4.11). The token pairs in
  `docs/WEBSITE_DESIGN.md` are precomputed; recompute any new pair. `muted` is for help
  text only. `yellow` is a fill under `on-yellow` text, never text on `bg`. `marker` green
  is decorative, never the only carrier of information.
- **Not colour alone:** links in running text are underlined; status uses text or an icon
  plus text, as in the app.
- **Images and media:** informative images get `alt` that says what they show; decorative
  ones `alt=""` (the logo next to the name). Video: captions, no autoplay with sound; any
  autoplaying motion longer than 5 s has a pause control (2.2.2).
- **Motion:** respect `prefers-reduced-motion: reduce` (the global rule in `global.css`
  neutralises animations and transitions; don't override it with `!important`). No
  scroll parallax, no flashing (> 3 per second). Pointer-following decoration (the hero
  pyramid) must stay still with reduced motion, on touch and without JS, and never move a control.
- **Forms (if ever):** visible `<label for>`, errors in text linked with
  `aria-describedby`, `autocomplete` on personal fields, no CAPTCHA puzzles.
- **Zoom and reflow:** usable at 200% text zoom and at 320 CSS px width (1.4.10); never
  `user-scalable=no` or `maximum-scale`.

## Deliverables and gates

Per change: keyboard-only walk-through (Tab order, focus visible), check in
both themes, 320px width and 200% zoom, VoiceOver (macOS) read-through of new content,
reduced-motion setting on. Automated: `bun run check` and `bun run build` pass; optionally
run axe or Lighthouse accessibility against `bun run preview` (no extra dependency
committed). Report which of these were done by hand and which were not.
