#!/usr/bin/env python3
"""Render the app icon PNGs (ADR-0021).

The art is the user's design (assets/icon/source.png, a 1254 px AI-generated raster)
redrawn as clean geometry so small sizes stay sharp. Line weight is 1.6x the source
("F" weight, user decision 2026-11-19), the same at every size.

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
STROKE = 32  # source units; the source art's stroke is ~20 on a 1254 canvas
ART = (279, 351, 976, 886)  # source art bounding box
FILL = 0.84  # art width / tile width
SS = 8  # supersampling factor
SIZES = [16, 24, 32, 48, 64, 128, 256, 512, 1024]
ICO_SIZES = [16, 24, 32, 48, 64, 128, 256]  # the .ico format stops at 256
OUT = Path(__file__).resolve().parent.parent / "assets" / "icon"


def art(size: int) -> Image.Image:
    """Line art on a full-bleed yellow square, `size` px."""
    w = STROKE
    n = size * SS
    art_w = ART[2] - ART[0] + w
    art_h = ART[3] - ART[1] + w
    scale = n * FILL / art_w
    ox = n / 2 - (ART[0] - w / 2 + art_w / 2) * scale
    oy = n / 2 - (ART[1] - w / 2 + art_h / 2) * scale

    def p(x, y):
        return (ox + x * scale, oy + y * scale)

    def arc(cx, cy, r, a0, a1, steps=16):
        return [
            (cx + r * math.cos(math.radians(a0 + (a1 - a0) * i / steps)),
             cy + r * math.sin(math.radians(a0 + (a1 - a0) * i / steps)))
            for i in range(steps + 1)
        ]

    im = Image.new("RGB", (n, n), YELLOW)
    d = ImageDraw.Draw(im)
    lw = max(1, round(w * scale))

    # Camera body with the viewfinder hump, one closed outline.
    x0, x1, top, bottom, hump, r = 289, 966, 454, 876, 361, 60
    pts = arc(x0 + r, top + r, r, 180, 270)
    pts += [(452, top), (465, top - 6), (505, hump + 25)]
    pts += arc(541, hump + 30, 30, 215, 270, 6)
    pts += [(712, hump)]
    pts += arc(712, hump + 30, 30, 270, 325, 6)
    pts += [(785, top - 6), (800, top)]
    pts += arc(x1 - r, top + r, r, 270, 360)
    pts += arc(x1 - r, bottom - r, r, 0, 90)
    pts += arc(x0 + r, bottom - r, r, 90, 180)
    pts.append(pts[0])
    d.line([p(*q) for q in pts], fill=INK, width=lw, joint="curve")

    def ring(cx, cy, rad, a0=0, a1=360):
        box = [*p(cx - rad - w / 2, cy - rad - w / 2), *p(cx + rad + w / 2, cy + rad + w / 2)]
        d.arc(box, a0, a1, fill=INK, width=lw)

    ring(626, 654, 181)  # lens
    ring(626, 652, 108, 34, 326)  # "C", open to the right
    m = [(584, 694), (584, 609), (628, 672), (672, 609), (672, 694)]
    d.line([p(*q) for q in m], fill=INK, width=lw, joint="curve")
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
