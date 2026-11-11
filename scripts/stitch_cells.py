#!/usr/bin/env python3
"""
Build an EXACT strip: width = cell * N, height = cell.
Input: clean/action-0.png ... clean/action-(N-1).png (any size) → resized into cell×cell → stitch.
This is how we get true 128*5 x 128 even when the generator can't.
"""
from __future__ import annotations

import argparse
import json
from pathlib import Path

from PIL import Image

ROOT = Path(__file__).resolve().parents[1]
CLEAN = ROOT / "assets" / "sprites" / "cat" / "clean"
OUT = ROOT / "assets" / "sprites" / "cat"


def fit_cell(im: Image.Image, cell: int) -> Image.Image:
    im = im.convert("RGBA")
    # keep aspect, pad to square cell, bottom-center
    bw, bh = im.size
    scale = min(cell / bw, cell / bh)
    nw, nh = max(1, int(bw * scale)), max(1, int(bh * scale))
    resized = im.resize((nw, nh), Image.Resampling.LANCZOS)
    canvas = Image.new("RGBA", (cell, cell), (0, 0, 0, 0))
    x = (cell - nw) // 2
    y = cell - nh
    canvas.paste(resized, (x, y), resized)
    return canvas


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("prefix")
    ap.add_argument("--frames", type=int, required=True)
    ap.add_argument("--cell", type=int, default=128)
    args = ap.parse_args()

    cells = []
    for i in range(args.frames):
        p = CLEAN / f"{args.prefix}-{i}.png"
        if not p.exists():
            raise SystemExit(f"missing {p}")
        cells.append(fit_cell(Image.open(p), args.cell))

    w, h = args.cell * args.frames, args.cell
    strip = Image.new("RGBA", (w, h), (0, 0, 0, 0))
    for i, c in enumerate(cells):
        strip.paste(c, (i * args.cell, 0), c)
    out = OUT / f"strip_{args.prefix}_{w}x{h}.png"
    strip.save(out)
    print(json.dumps({"out": str(out), "size": [w, h], "cell": args.cell, "frames": args.frames}))


if __name__ == "__main__":
    main()
