# Website design

How the website looks and what it may say. It follows the desktop app's design
([`PRODUCT.md`](PRODUCT.md) "UI design", reference [`design-lab.html`](design-lab.html)) so the
two read as one product. Build mechanics are in [`WEBSITE.md`](WEBSITE.md). Changing the
visual direction needs the user's OK, as for the app.

## Brand

- **Character:** monochrome, quiet, practical; real numbers and plain words.
- **Name:** "Open Camera Mouse", written out in full. One exception (user's call,
  2026-11-21): below `sm` the header wordmark reads "OCM" so the header fits one row; the
  full name stays in the link's accessible name. "OCM" appears nowhere else.
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
(`prefers-color-scheme`): Carbon when dark or unset, Paper when light, unless the visitor
picks one with the header switch (see "Theme switch").

| Utility | Token | Carbon | Paper | Use |
|---|---|---|---|---|
| `bg-bg` | bg | `#000000` | `#ececec` | page background |
| `bg-surface` / `bg-surface-2` | surface / surface-2 | `#0a0a0a` / `#151515` | `#f8f8f8` / `#ffffff` | panels, cards, code |
| `border-line` / `border-line-strong` | line / line-strong | `#262626` / `#3f3f3f` | `#d6d6d6` / `#a8a8a8` | separators / decorative outlines |
| `text-text` / `text-muted` | text / muted | `#fafafa` / `#8f8f8f` | `#111111` / `#5a5a5a` | body / help text |
| `*-marker` | marker | `#00ff00` | `#00ff00` | target-ring motif only |
| `bg-yellow` + `text-on-yellow` | yellow / on-yellow | `#ffd60a` / `#111111` | `#f5c518` / `#111111` | primary call to action |
| `.on-brand` (`brand` / `on-brand`) | brand / on-brand | `#ffd60a` / `#000000` | same | header, hero band and footer; also inverted (`bg-on-brand text-brand`) for the hero's primary button |
| `rounded-frame` / `rounded-widget` | radius | 8px / 6px | same | panels, images / buttons, inputs |
| `font-mono` (page default) / `font-sans` | font | system monospace stack / system UI stack | same | no web fonts; mono is the typewriter face for all text |

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

- **Typewriter for all text** (user's call, 2026-11-20): the page default is the system
  monospace stack (`font-mono`, set on `html`), so body, headings, buttons and labels are all
  mono. Small labels and navigation are additionally uppercase with `tracking-wide`.
  `font-sans` remains as a token but is unused. (A real typewriter face would be a
  self-hosted web font: needs the user's OK and an ADR-0023 amendment.) Mono runs about
  0.6em per character, so check long words in large headings at 320px. Body 16px (`text-base`, line-height 1.5), help text
  `text-sm`, headings `font-semibold` at `text-3xl`–`text-4xl` (h1) stepping down; no
  all-caps body text.
- Reading width `max-w-prose`; page column `max-w-5xl`, gutters `px-4` / `sm:px-6`.
- Sections separated by space (`py-12`) and 1px `border-line` rules, not boxes and shadows.

## Theme switch

A night/day switch at the end of the header (user's call, 2026-11-20): a
`<button role="switch">` named "Night theme", `aria-checked="true"` for Carbon (night) and
`false` for Paper (day), with `title` saying what a click does. It looks like a switch: a
pill track outlined in black with a sun (left) and a moon (right), and a black knob that
slides to the current side and shows that theme's icon in yellow (CSS
`group-aria-checked:`; reduced motion makes the slide instant). Icons are `aria-hidden`; the
target is at least 44px tall. A click flips
the theme by setting `data-theme="carbon|paper"` on `<html>` and stores it in
`localStorage` (`ocm-theme`); with no stored choice the theme follows the OS. A tiny
`is:inline` script in `BaseLayout` applies the stored choice before first paint (no flash).
Progressive enhancement: the button ships `hidden` and its script reveals it, so with
JavaScript off it isn't shown and the OS setting applies. Storage errors are ignored (the
switch still works for the page view). Not a cookie; nothing leaves the browser.

## Responsive

Mobile first. One column from 320px; extra columns from `md` up only where content is
parallel (e.g. per-OS download cards). No horizontal scrolling at 320px, usable at 200%
zoom. Navigation and calls to action at least 44×44px (`min-h-11`), because many visitors
point with their head or dwell-click. Images and screenshots carry `width`/`height`.

## Accessibility

WCAG 2.2 AA is a requirement, not a goal; the rules are in the `web-accessibility` skill.
In short: semantic landmarks (no skip link: removed at the user's request, 2026-11-20;
keyboard and screen-reader users move by landmarks and headings), visible 2px focus outline,
underlined links in text, status never by colour alone, alt text, reduced motion respected
(global rule in `global.css`), no autoplaying motion without a pause control.

## Header and hero

Every page **starts yellow**: `SiteHeader.astro` sits on the brand surface (`.on-brand`) with
no border or rule; `theme-color` is `#ffd60a`. On the home page the hero
(`src/components/Hero.astro`) continues the same surface directly under it, so header and
hero read as one yellow block. The first section under the hero has no top rule either.

Hero shape: the bottom edge is a **shallow asymmetric V** (`.angular-bottom`): both sides
sit `--cut` (`clamp(1.5rem, 5vw, 4rem)`) above the point, which sits `--apex` (30%) from
the left, so a short steeper left side and a long gentle right one (user's call,
2026-11-21; it was centred before). Built from `--cut`
and percentages, so it keeps its proportions from 320px up; content is padded by at least
`--cut` below. No other shapes in the hero: an earlier one-way diagonal with a blade and a
wedge was rejected as ugly (2026-11-20).

No tracking ring or crosshair in the hero (removed at the user's request).

From `lg` up the hero is two columns: the text, and on the right (`HeroSquirrel.astro`, at
most 20rem) a cute squirrel seen from the front, in black lines on the yellow (user's call,
2026-11-21, after a stepped pyramid and a cubist head were tried the same day and found too
spiky; first drawn side-on with curves, then from the front, then redrawn "more cute and
angular"): **faceted but not spiky**, so straight edges, obtuse angles and round joins; only
the eyes stay round. A big octagonal head on a chubby body, solid black triangle ears,
one big and one small eye (each with a shine), angular blush slashes, a small nose, a "w"
mouth with two buck teeth, a hexagonal acorn in angled paws, trapezoid feet, a bushy faceted
tail curling up its right with two solid black facets (the curl's tip and a band), sitting on a ground line. **No line crosses another**: parts in
front cover what's behind them with a yellow fill, and the face details stay inside the head
(the teeth meet the mouth at points only).
It watches the pointer, a nod to what the app does: the head tilts up to 12° toward the
pointer's side and shifts up to 6 units toward it, the face slides up to 8 more inside the
head, both pupils look toward it (each inside its own eye), and the tail sways up to 8° the
other way; it eases back to looking ahead when the pointer leaves the window. Pose maths in `src/lib/squirrel.ts`
(tested), shared by the build-time render and the script. `aria-hidden`, no focus, hidden
below `lg`. Being a drawing, it uses curves; the "Angular language" applies to layout shapes.

Header contents: logo and wordmark (home link), then a "Manual" text link (uppercase,
underlined while on a manual page, `aria-current="page"` on `/manual/`), the GitHub mark
(inline Octicons SVG, link named "Source code on GitHub") and the theme switch; no
"Download" link (the hero has it). Below `sm` it stays one row: the wordmark shows "OCM" (link name "OCM Open
Camera Mouse") and "Manual" becomes an outline book icon (inline SVG, `title` and
screen-reader text "Manual", a 2px bar under it on manual pages). A nav row
(Manual, How it works, Download, Open source) with a hamburger menu was tried and dropped
(2026-11-21): the other links only jumped to home-page sections. Instead the home page links
the manual from "How it works" ("Read the manual →") and "Open source".

Hero contents (copy from the user, 2026-11-20): the h1 "Computing without barriers" (no full stop) in
heavy uppercase (CSS `uppercase`, sentence case in the source so screen readers don't spell
it out), three lines below `lg`, two from `lg` up ("Computing without" / "barriers", a
smaller `text-6xl`, to keep the hero short; user's call, 2026-11-21); "Control your computer using head movements and an ordinary webcam. Free and open
source."; the primary action (black button, yellow text) and a secondary outlined one; then
one short download line, `v0.4.0 · Apple Silicon · Other platforms` (see "Downloads"). No
eyebrow line above the headline. Inside the yellow, text and the focus outline are black.

## Footer

Every page **ends yellow** too (2026-11-21, pending the user's approval): `SiteFooter.astro` is
on the brand surface with a shallow **Λ top edge** (`.angular-top`, the point `--cut` above
both sides), mirroring the hero's V, so header/hero and footer bookend the page. Contents:
logo and wordmark (not a link: the header has the home link), the hero's one-line description,
a `nav` named "Footer" with two labelled lists ("Manual": getting started, installation,
troubleshooting, questions; "Project": download, all releases, source code, report a problem),
then a 2px black rule and the licence line ("Free software under the MIT licence") and "No
cookies, no analytics." Links are underlined, black, `min-h-11`. Columns: one below `sm`, the
two lists side by side from `sm`, the description beside them from `md`. No script.

## Downloads

Each platform card (`#download`) shows its requirement, its notes, and one outlined button
per file (`border-2 border-text`, the file type as the label, "Download <OS>" for screen
readers), then the install-steps link; under the cards, a text link to the release's notes,
files and checksums. The hero's black button is the one primary download. With JavaScript,
the visitor's card gets a "Your system" label (text, not colour) and the hero button reads
"Download for <OS>" with `v<version> · <short requirement> · Other platforms` under it
(`short` in `platforms.ts`; without JS or for an unknown system, `v<version> · macOS,
Windows, Linux`);
on a phone, tablet or ChromeOS both say Open Camera Mouse runs on macOS, Windows and Linux
computers. The CPU isn't guessed, so the requirement (e.g. Apple Silicon) is always shown
next to a direct download.

## Manual pages

Normal padded column under the flat yellow header. From `md` up: a 13rem sidebar (small
uppercase "Manual" label, then the pages; the current one bold with a 4px `text` bar on the
left and `aria-current`, so not by colour) beside the article. Below `md` the same list sits
in a collapsed `<details>` ("Manual contents") above the page. Article: h1, the page's
description as a muted lead, an "On this page" list (4px `line` bar), the Markdown in
`.manual-prose` (`global.css`: `max-w-prose`, `h2` `text-2xl`, `h3` `text-xl`, `code` and
`kbd` boxed on `surface-2`, `kbd` with a `text` border, code blocks on `surface` that wrap,
tables with 1px `line` rows and a `text` rule under the header), then previous/next.
Tables stay narrow enough to fit 320px (no sideways scrolling).

## Angular language

Sharp, not rounded, and calm: straight edges and shallow angles (the hero's V, its point off-centre to the left)
in solid fills, no gradients or shadows. The footer's Λ echoes the hero's V (still centred), not a new shape. Fewer shapes beat more: the user rejected a
layered diagonal/blade/wedge hero as ugly.
Geometry is decorative (`aria-hidden`) and never carries information. Controls keep the
normal radius and rectangular hit areas.

## Motion

CSS, plus the one pointer-following hero figure below. Allowed: a one-time entrance
(rise/fade ≤ 500 ms, slide ≤ 700 ms, staggered by ≤ 250 ms), a small hover lift on buttons,
short transitions. The entrance plays on the **first page of a visit** only: BaseLayout's
inline head script sets `data-entered` on `<html>` before first paint on every later page and
on reload (`sessionStorage` `ocm-entered`), and `global.css` turns the animation off there, so
a reload no longer blanks the hero and fades it back in (2026-11-21). Not allowed: looping
animation, scroll parallax, scroll-jacking, anything that moves a control while it can be
pointed at or dwelled on (a hover lift is ≤ 2px and the hit area stays put). The global
`prefers-reduced-motion` rule in `global.css` collapses all of it, delays included.

**Hero squirrel** (user's request, 2026-11-21): the one exception to "CSS only", a decorative
figure that follows the pointer. It moves only while the pointer moves (eases to its goal in
`requestAnimationFrame`, then stops: no loop), never with reduced motion, on touch-only
screens (`any-hover: none`) or without JavaScript (then it looks ahead, still), and
it isn't a control. Details under "Header and hero".

## Page structure

Homepage sections and their purpose: `WEBSITE.md` "Homepage"; status in
`WEBSITE_ROADMAP.md`.

## Constraints

- No claims the docs don't support: no invented features, compatibility, release dates,
  download numbers, testimonials, or medical/certification claims (a PRODUCT non-goal).
  Versions and file links come from the latest release at build time, never typed by hand;
  without release data, link to `releases/latest`.
- Don't say on the site that the Windows or Linux builds haven't been tried on a real device
  (home page, download cards, manual, FAQ): the user's call, 2026-11-21. That caveat stays
  in the repo docs (`README.md`, `TESTING.md`). Other limits, like untested USB webcams,
  stay on the site.
- The showcase is a real recording of the app; never a mock-up or simulated tracking, and
  never a browser-based webcam tracker.
- No web fonts, icon fonts or icon libraries (inline SVG if an icon is needed), no
  third-party embeds, analytics or cookies.
- No JavaScript for anything CSS and HTML can do.
- Screenshots must show the real app and be regenerated when the UI changes; never show a
  real person's camera image without their consent (and never anything from diagnostic
  recordings).
