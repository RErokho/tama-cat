#!/usr/bin/env python3
"""Normalize clean sprite cells so max(bw,bh) ≈ TARGET_PRIMARY, then reassemble sheet."""
from __future__ import annotations

import sys
from pathlib import Path

from PIL import Image

sys.path.insert(0, str(Path(__file__).resolve().parent))
from process_sprite_frame import harden_alpha, keep_largest_blob
from slice_gemini_sheet import (
    ANIMATIONS,
    EXTRA_STEMS,
    FRAME_H,
    FRAME_NAMES,
    FRAME_W,
    assemble,
)

TARGET_PRIMARY = 108
TOP_ALIGN = {"clingTop-0", "clingTop-1", "hangDrop-0", "hangDrop-1"}
RIGHT_ALIGN = {"wallLean-0", "wallLean-1"}
CLEAN = Path(__file__).resolve().parents[1] / "assets" / "sprites" / "cat" / "clean"


def content_bbox(im: Image.Image, thr: int = 40):
    return im.split()[-1].point(lambda a: 255 if a >= thr else 0).getbbox()


def normalize(stem: str) -> None:
    path = CLEAN / f"{stem}.png"
    im = Image.open(path).convert("RGBA")
    im = keep_largest_blob(im)
    im = harden_alpha(im, thr=40)
    bb = content_bbox(im)
    if not bb:
        print(stem, "EMPTY")
        return
    content = im.crop(bb)
    bw, bh = content.size
    primary = max(bw, bh)
    scale = TARGET_PRIMARY / max(1, primary)
    nw = max(1, int(round(bw * scale)))
    nh = max(1, int(round(bh * scale)))
    fit = min(1.0, (FRAME_W - 4) / nw, (FRAME_H - 4) / nh)
    nw = max(1, int(round(nw * fit)))
    nh = max(1, int(round(nh * fit)))
    resized = content.resize((nw, nh), Image.Resampling.LANCZOS)
    canvas = Image.new("RGBA", (FRAME_W, FRAME_H), (0, 0, 0, 0))
    if stem in TOP_ALIGN:
        x, y = (FRAME_W - nw) // 2, 1
    elif stem in RIGHT_ALIGN:
        x, y = FRAME_W - nw - 1, FRAME_H - nh - 2
    else:
        x, y = (FRAME_W - nw) // 2, FRAME_H - nh - 2
    canvas.paste(resized, (x, y), resized)
    harden_alpha(canvas, thr=40).save(path)
    print(f"{stem}: {bw}x{bh} -> {nw}x{nh}")


def main() -> None:
    stems = [FRAME_NAMES[i] for i in range(40)] + list(EXTRA_STEMS)
    for stem in stems:
        if (CLEAN / f"{stem}.png").exists():
            normalize(stem)
    meta = assemble(stems, ANIMATIONS)
    print("sheet frames", len(meta["frameNames"]))
    # refresh fill/mass card after normalize
    from sprite_metrics import build_profile, save_profile

    profile = build_profile(stems)
    path = save_profile(profile)
    print(f"scale_profile {path} frames={len(profile['frames'])}")


if __name__ == "__main__":
    main()
