# Website design

How the website looks and what it may say. It follows the desktop app's design
([`PRODUCT.md`](PRODUCT.md) "UI design", reference [`design-lab.html`](design-lab.html)) so the
two read as one product. Build mechanics are in [`WEBSITE.md`](WEBSITE.md). Changing the
visual direction needs the user's OK, as for the app.

## Brand

- **Character:** monochrome, quiet, practical; real numbers and plain words.
- **Name:** "Open Camera Mouse", written out in full; never "OCM" in visible text.
- **Logo:** on the website, a one-colour SVG mark: the camera with C and M cut out
  (`website/src/assets/logo.svg`, one even-odd path, `fill="currentColor"`), traced from the
  user's transparent PNG (2026-11-20). Imported as an Astro SVG component, drawn black on the
  yellow header, so the yellow shows through the C and M. Next to the wordmark it is
  decorative (`aria-hidden`). Don't redraw or recolour it except through `currentColor`. The
  app icon (black camera, yellow C/M, `assets/icon/`) stays the app's icon and the favicon
  (`website/public/favicon.png`).
- **Wordmark:** "OPEN CAMERA MOUSE" in the header: `font-black`, uppercase via CSS,
  slight tracking.
- **Accents:** the app's two. Neon green `#00ff00` only for the thin target-ring motif.
  Yellow as the fill of the primary call to action (`#111111` text) and, on the website
  only, as the **brand surface** of the hero band (`#ffd60a` with black, both themes;
  approved 2026-11-20). Everything else is the grey scale below.

## Design tokens

Defined once in `website/src/styles/global.css`; the values are the app's
(`apps/desktop/src/theme.rs`, `PRODUCT.md`). The theme follows the visitor's OS setting
(`prefers-color-scheme`): Carbon when dark or unset, Paper when light. There is no theme
switch yet (it would need JavaScript and storage; decide in the design task).

| Utility | Token | Carbon | Paper | Use |
|---|---|---|---|---|
| `bg-bg` | bg | `#000000` | `#ececec` | page background |
| `bg-surface` / `bg-surface-2` | surface / surface-2 | `#0a0a0a` / `#151515` | `#f8f8f8` / `#ffffff` | panels, cards, code |
| `border-line` / `border-line-strong` | line / line-strong | `#262626` / `#3f3f3f` | `#d6d6d6` / `#a8a8a8` | separators / decorative outlines |
| `text-text` / `text-muted` | text / muted | `#fafafa` / `#8f8f8f` | `#111111` / `#5a5a5a` | body / help text |
| `*-marker` | marker | `#00ff00` | `#00ff00` | target-ring motif only |
| `bg-yellow` + `text-on-yellow` | yellow / on-yellow | `#ffd60a` / `#111111` | `#f5c518` / `#111111` | primary call to action |
| `.on-brand` (`brand` / `on-brand`) | brand / on-brand | `#ffd60a` / `#000000` | same | hero band only; also inverted (`bg-on-brand text-brand`) for the hero's primary button |
| `rounded-frame` / `rounded-widget` | radius | 8px / 6px | same | panels, images / buttons, inputs |
| `font-sans` / `font-mono` | font | system UI stack / system monospace stack | same | no web fonts; mono for typewriter-style accents |

Tailwind's default colours are removed, so any other colour utility doesn't exist. Spacing
uses Tailwind's 4px scale.

### Measured contrast (WCAG 2.x formula)

| Pair | Carbon | Paper | Meets |
|---|---|---|---|
| text on bg | 20.1:1 | 16.0:1 | AAA text |
| muted on bg | 6.5:1 | 5.8:1 | AA text |
| muted on surface-2 | 5.6:1 | 6.9:1 | AA text |
| on-yellow on yellow | 13.4:1 | 11.6:1 | AA text |
| on-brand on brand (and inverse) | 14.9:1 | 14.9:1 | AAA text, focus outline |
| line-strong on bg | 2.0:1 | 2.0:1 | **below 3:1**: decorative only |
| yellow on bg | 14.9:1 | 1.4:1 | Paper: **below 3:1** |
| marker on bg | 15.3:1 | 1.2:1 | Paper: **below 3:1**, decorative only |

So: a control's boundary that matters (input outline, secondary button) uses `text`-coloured
borders, not `line-strong`; the Paper yellow button is identified by its label, and gets a
`border-text` outline if its edge needs to be visible; the marker never carries meaning.

## Typography and spacing

- System font stack (as in the app). Typewriter-style accents in `font-mono`, uppercase with
  `tracking-wide`, used sparingly: navigation, the hero's small top line, step numbers and
  similar labels; never body text or headings. (A real typewriter face would be a
  self-hosted web font: needs the user's OK and an ADR-0023 amendment.) Body 16px (`text-base`, line-height 1.5), help text
  `text-sm`, headings `font-semibold` at `text-3xl`–`text-4xl` (h1) stepping down; no
  all-caps body text.
- Reading width `max-w-prose`; page column `max-w-5xl`, gutters `px-4` / `sm:px-6`.
- Sections separated by space (`py-12`) and 1px `border-line` rules, not boxes and shadows.

## Responsive

Mobile first. One column from 320px; extra columns from `md` up only where content is
parallel (e.g. per-OS download cards). No horizontal scrolling at 320px, usable at 200%
zoom. Navigation and calls to action at least 44×44px (`min-h-11`), because many visitors
point with their head or dwell-click. Images and screenshots carry `width`/`height`.

## Accessibility

WCAG 2.2 AA is a requirement, not a goal; the rules are in the `web-accessibility` skill.
In short: semantic landmarks with a skip link (in the layout), visible 2px focus outline,
underlined links in text, status never by colour alone, alt text, reduced motion respected
(global rule in `global.css`), no autoplaying motion without a pause control.

## Header and hero

Every page **starts yellow**: `SiteHeader.astro` sits on the brand surface (`.on-brand`) with
no border or rule; `theme-color` is `#ffd60a`. On the home page the hero
(`src/components/Hero.astro`) continues the same surface directly under it, so header and
hero read as one yellow block. The first section under the hero has no top rule either.

Hero shape: the bottom edge is a **shallow symmetric V** (`.angular-bottom`): both sides
sit `--cut` (`clamp(1.5rem, 5vw, 4rem)`) above the point in the middle. Built from `--cut`
and percentages, so it keeps its proportions from 320px up; content is padded by at least
`--cut` below. No other shapes in the hero: an earlier one-way diagonal with a blade and a
wedge was rejected as ugly (2026-11-20).

No tracking ring or crosshair in the hero (removed at the user's request).

Contents: a small mono uppercase line (free, open source, licence), the h1 headline in heavy
uppercase (CSS `uppercase`, sentence case in the source so screen readers don't spell it
out), one sentence on what it does and that it runs locally, then the primary action (black
button, yellow text) and a secondary outlined one. Inside the yellow, text and the focus
outline are black. Copy is a draft until the user approves it.

## Angular language

Sharp, not rounded, and calm: straight edges and shallow, symmetric angles (the hero's V)
in solid fills, no gradients or shadows. Fewer shapes beat more: the user rejected a
layered diagonal/blade/wedge hero as ugly.
Geometry is decorative (`aria-hidden`) and never carries information. Controls keep the
normal radius and rectangular hit areas.

## Motion

CSS only. Allowed: a one-time entrance (rise/fade ≤ 500 ms, slide ≤ 700 ms, staggered by
≤ 250 ms), a small hover lift on buttons, short transitions. Not allowed: looping
animation, parallax, scroll-jacking, anything that moves a control while it can be pointed
at or dwelled on (a hover lift is ≤ 2px and the hit area stays put). The global
`prefers-reduced-motion` rule in `global.css` collapses all of it, delays included.

## Page structure

Homepage sections and their purpose: `WEBSITE.md` "Homepage"; status in
`WEBSITE_ROADMAP.md`.

## Constraints

- No claims the docs don't support: no invented features, compatibility, release dates,
  download numbers, testimonials, or medical/certification claims (a PRODUCT non-goal).
  Link to `releases/latest`, never to a hard-coded version.
- The showcase is a real recording of the app; never a mock-up or simulated tracking, and
  never a browser-based webcam tracker.
- No web fonts, icon fonts or icon libraries (inline SVG if an icon is needed), no
  third-party embeds, analytics or cookies.
- No JavaScript for anything CSS and HTML can do.
- Screenshots must show the real app and be regenerated when the UI changes; never show a
  real person's camera image without their consent (and never anything from diagnostic
  recordings).
