# Website design

How the website looks and what it may say. It follows the desktop app's design
([`PRODUCT.md`](PRODUCT.md) "UI design", reference [`design-lab.html`](design-lab.html)) so the
two read as one product. Build mechanics are in [`WEBSITE.md`](WEBSITE.md). Changing the
visual direction needs the user's OK, as for the app.

## Brand

- **Character:** monochrome, quiet, practical; real numbers and plain words.
- **Name:** "Open Camera Mouse", written out in full; never "OCM" in visible text.
- **Logo:** the app icon, a solid black camera with a yellow C and M (`assets/icon/source.png`;
  PNG sizes in `assets/icon/png/`). The site uses copies in `website/public/`
  (`favicon.png` = 32px, `icon-128.png` shown at 32px for high-DPI screens). Use the icon
  unchanged, at square sizes, with no recolouring, outline or crop. Next to the wordmark it
  is decorative (`alt=""`).
- **Accents:** exactly the app's two. Neon green `#00ff00` only for the thin target-ring
  motif; yellow only as the fill of the primary call to action, with `#111111` text.
  Everything else is the grey scale below.

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
| `bg-yellow` + `text-on-yellow` | yellow / on-yellow | `#ffd60a` / `#111111` | `#f5c518` / `#111111` | primary call to action only |
| `rounded-frame` / `rounded-widget` | radius | 8px / 6px | same | panels, images / buttons, inputs |
| `font-sans` / `font-mono` | font | system UI stack / `ui-monospace` stack | same | no web fonts |

Tailwind's default colours are removed, so any other colour utility doesn't exist. Spacing
uses Tailwind's 4px scale.

### Measured contrast (WCAG 2.x formula)

| Pair | Carbon | Paper | Meets |
|---|---|---|---|
| text on bg | 20.1:1 | 16.0:1 | AAA text |
| muted on bg | 6.5:1 | 5.8:1 | AA text |
| muted on surface-2 | 5.6:1 | 6.9:1 | AA text |
| on-yellow on yellow | 13.4:1 | 11.6:1 | AA text |
| line-strong on bg | 2.0:1 | 2.0:1 | **below 3:1**: decorative only |
| yellow on bg | 14.9:1 | 1.4:1 | Paper: **below 3:1** |
| marker on bg | 15.3:1 | 1.2:1 | Paper: **below 3:1**, decorative only |

So: a control's boundary that matters (input outline, secondary button) uses `text`-coloured
borders, not `line-strong`; the Paper yellow button is identified by its label, and gets a
`border-text` outline if its edge needs to be visible; the marker never carries meaning.

## Typography and spacing

- System font stack (as in the app). Body 16px (`text-base`, line-height 1.5), help text
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

## Initial page structure

Built now: one layout (`BaseLayout`) with skip link, header (icon + name linking home,
"Source code" link), `<main>`, footer (licence link); a placeholder home page and a 404.

Proposed for the design task (needs the user's OK; content only from `README.md` and
`PRODUCT.md`):

1. Hero: name, the one-sentence description, one primary action (yellow).
2. How it works: the click-a-spot / track / move / dwell-click steps from the README "Use".
3. Privacy: everything runs on the device, no camera image leaves it.
4. Download per OS: links to the GitHub release; requirements and caveats linked from the
   README, not restated.
5. Footer: licence, source, issues.

## Constraints

- No claims the docs don't support: no invented features, compatibility, release dates,
  download numbers, testimonials, or medical/certification claims (a PRODUCT non-goal).
  Until a release is published, don't link to a release page as if it existed.
- No web fonts, icon fonts or icon libraries (inline SVG if an icon is needed), no
  third-party embeds, analytics or cookies.
- No JavaScript for anything CSS and HTML can do.
- Screenshots must show the real app and be regenerated when the UI changes; never show a
  real person's camera image without their consent (and never anything from diagnostic
  recordings).
