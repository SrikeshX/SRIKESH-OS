#!/usr/bin/env python3
"""Generates modern 1920x1080 Aurora Bloom wallpaper for SRIKESH OS Hybrid."""
import math
import sys
from PIL import Image, ImageDraw, ImageFilter

def generate_aurora_wallpaper(out_path: str, width: int = 1920, height: int = 1080):
    # Render at 480x270 for fast smooth gradients, then upscale with Lanczos
    w, h = 480, 270
    img = Image.new("RGB", (w, h))
    px = img.load()

    for y in range(h):
        ny = y / h
        for x in range(w):
            nx = x / w
            # Deep midnight slate base
            r = 10 + 14 * (1.0 - ny)
            g = 18 + 26 * (1.0 - ny)
            b = 38 + 44 * (1.0 - ny)

            # Central luminous cobalt/cyan aurora bloom
            dx1 = (nx - 0.52) * 1.45
            dy1 = (ny - 0.44) * 1.65
            d1 = math.sqrt(dx1 * dx1 + dy1 * dy1)
            bloom1 = math.exp(-d1 * d1 * 4.2)

            # Secondary sapphire/violet ribbon swirl
            wave = 0.46 + 0.14 * math.sin(nx * 4.8 + 0.6) - 0.08 * math.cos(nx * 2.3)
            d_wave = abs(ny - wave)
            ribbon = math.exp(-d_wave * d_wave * 48.0) * (0.4 + 0.6 * math.sin(nx * math.pi))

            # Subtle warm violet-indigo accent on lower right
            dx2 = (nx - 0.72) * 1.5
            dy2 = (ny - 0.62) * 1.8
            d2 = math.sqrt(dx2 * dx2 + dy2 * dy2)
            bloom2 = math.exp(-d2 * d2 * 5.0)

            # Highlight ribbon crest
            crest = math.exp(-abs(ny - (wave - 0.03)) ** 2 * 180.0) * math.sin(nx * math.pi)

            r += bloom1 * 38 + ribbon * 44 + bloom2 * 65 + crest * 70
            g += bloom1 * 125 + ribbon * 130 + bloom2 * 45 + crest * 165
            b += bloom1 * 205 + ribbon * 215 + bloom2 * 165 + crest * 225

            px[x, y] = (
                max(0, min(255, int(r))),
                max(0, min(255, int(g))),
                max(0, min(255, int(b))),
            )

    resample = getattr(getattr(Image, "Resampling", Image), "LANCZOS")
    img = img.resize((width, height), resample)
    img = img.filter(ImageFilter.GaussianBlur(radius=1.5))
    img.save(out_path, "PNG", optimize=True)
    print(f"[WALLPAPER] Generated {out_path} ({width}x{height})")

if __name__ == "__main__":
    out = sys.argv[1] if len(sys.argv) > 1 else "aurora_bloom.png"
    generate_aurora_wallpaper(out)
