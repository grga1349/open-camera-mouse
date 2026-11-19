#!/usr/bin/env python3
"""Render the app icon PNGs (ADR-0021).

The art is the user's design (assets/icon/source.png, a 1254 px AI-generated raster, second
version 2026-11-19: solid black camera, yellow "C" around "M" and a small ring) redrawn as
clean geometry, measured from the source, so small sizes stay sharp.

Writes into assets/icon/:
  png/icon-<N>.png        full-bleed square (Windows .ico, Linux, eframe window icon)
  macos/icon-<N>.png      Apple grid: 824/1024 rounded square, transparent margin (.icns)
  windows/icon.ico        full-bleed art at 16-256 px, each size drawn, not scaled (NSIS)

Dev tool only (not run in CI or the build): needs Python 3 + Pillow, e.g.
  python3 -m venv /tmp/venv && /tmp/venv/bin/pip install pillow==11.3.0
  /tmp/venv/bin/python scripts/render-icon.py
"""
import math
from pathlib import Path

from PIL import Image, ImageDraw

YELLOW = (255, 209, 0)
INK = (0, 0, 0)
ART = (184, 225, 1068, 950)  # source art bounding box (camera incl. viewfinder hump)
FILL = 0.84  # art width / tile width
SS = 8  # supersampling factor
SIZES = [16, 24, 32, 48, 64, 128, 256, 512, 1024]
ICO_SIZES = [16, 24, 32, 48, 64, 128, 256]  # the .ico format stops at 256
OUT = Path(__file__).resolve().parent.parent / "assets" / "icon"


def art(size: int) -> Image.Image:
    """The camera on a full-bleed yellow square, `size` px. Coordinates are source px."""
    n = size * SS
    scale = n * FILL / (ART[2] - ART[0])
    ox = n / 2 - (ART[0] + ART[2]) / 2 * scale
    oy = n / 2 - (ART[1] + ART[3]) / 2 * scale

    def p(x, y):
        return (ox + x * scale, oy + y * scale)

    def box(x0, y0, x1, y1):
        return [*p(x0, y0), *p(x1, y1)]

    def disk(cx, cy, r, fill):
        d.ellipse(box(cx - r, cy - r, cx + r, cy + r), fill=fill)

    im = Image.new("RGB", (n, n), YELLOW)
    d = ImageDraw.Draw(im)

    # Body.
    d.rounded_rectangle(box(184, 345, 1068, 950), radius=round(123 * scale), fill=INK)

    def corner(v, a, b, r, inner, outer):
        """Round the corner at `v` (edges towards `a` and `b`) with a radius-`r` arc:
        the wedge outside the arc gets `outer`, the arc's disk `inner`."""
        u1 = ((a[0] - v[0]), (a[1] - v[1]))
        u2 = ((b[0] - v[0]), (b[1] - v[1]))
        l1, l2 = math.hypot(*u1), math.hypot(*u2)
        u1, u2 = (u1[0] / l1, u1[1] / l1), (u2[0] / l2, u2[1] / l2)
        half = math.acos(u1[0] * u2[0] + u1[1] * u2[1]) / 2
        t = r / math.tan(half)
        bis = (u1[0] + u2[0], u1[1] + u2[1])
        lb = math.hypot(*bis)
        c = (v[0] + bis[0] / lb * r / math.sin(half), v[1] + bis[1] / lb * r / math.sin(half))
        t1 = (v[0] + u1[0] * t, v[1] + u1[1] * t)
        t2 = (v[0] + u2[0] * t, v[1] + u2[1] * t)
        d.polygon([p(*v), p(*t1), p(*t2)], fill=outer)
        disk(*c, r, inner)

    # Viewfinder hump: trapezoid on the body, rounded top corners, small concave fillets.
    top, base = 225, 345
    bl, tl, tr, br = (405, base), (483, top), (769, top), (847, base)
    d.polygon([p(bl[0], base + 1), p(*tl), p(*tr), p(br[0], base + 1)], fill=INK)
    corner(tl, bl, tr, 58, INK, YELLOW)
    corner(tr, tl, br, 58, INK, YELLOW)
    corner(bl, (bl[0] - 100, base), tl, 18, YELLOW, INK)
    corner(br, (br[0] + 100, base), tr, 18, YELLOW, INK)

    # Small ring, top left.
    disk(302, 490, 66, YELLOW)
    disk(302, 490, 33, INK)

    # "C": an annulus, open to the right, with round ends.
    cx, cy, ro, ri = 626, 635, 231, 175
    rm, half = (ro + ri) / 2, (ro - ri) / 2
    a0, a1 = 40, 320  # PIL angles: clockwise from 3 o'clock
    d.pieslice(box(cx - ro, cy - ro, cx + ro, cy + ro), a0, a1, fill=YELLOW)
    disk(cx, cy, ri, INK)
    for a in (a0, a1):
        disk(cx + rm * math.cos(math.radians(a)), cy + rm * math.sin(math.radians(a)), half, YELLOW)

    # "M": two stems and a V, stroke 47.
    sw = 47
    for x0 in (535, 688):
        d.rounded_rectangle(box(x0, 548, x0 + sw, 723), radius=round(12 * scale), fill=YELLOW)
    mx, vy = 635, 668
    d.line([p(558, 572), p(mx, vy), p(711, 572)], fill=YELLOW, width=round(sw * scale),
           joint="curve")
    disk(mx, vy, sw / 2, YELLOW)
    return im.resize((size, size), Image.LANCZOS)


def macos(size: int) -> Image.Image:
    """Apple icon grid: 824/1024 rounded square (radius ~22.5%), centred, transparent margin."""
    tile = round(size * 824 / 1024)
    off = (size - tile) // 2
    big = size * SS
    mask = Image.new("L", (big, big), 0)
    t = tile * SS
    o = off * SS
    ImageDraw.Draw(mask).rounded_rectangle([o, o, o + t, o + t], radius=round(t * 0.225), fill=255)
    mask = mask.resize((size, size), Image.LANCZOS)
    out = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    out.paste(art(tile), (off, off))
    out.putalpha(mask)
    return out


def main():
    (OUT / "png").mkdir(parents=True, exist_ok=True)
    (OUT / "macos").mkdir(parents=True, exist_ok=True)
    for s in SIZES:
        art(s).save(OUT / "png" / f"icon-{s}.png", optimize=True)
        macos(s).save(OUT / "macos" / f"icon-{s}.png", optimize=True)
    (OUT / "windows").mkdir(parents=True, exist_ok=True)
    ico = [art(s).convert("RGBA") for s in ICO_SIZES]  # image-rs (cargo-packager) needs RGBA
    ico[-1].save(OUT / "windows" / "icon.ico", sizes=[(s, s) for s in ICO_SIZES],
                 append_images=ico[:-1])
    print(f"wrote {len(SIZES) * 2} PNGs and icon.ico to {OUT}")


if __name__ == "__main__":
    main()
