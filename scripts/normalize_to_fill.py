#!/usr/bin/env python3
"""Rescale clean frames so fill% ≈ expected_fill_pct(pose) for this cell size.

fill% = opaque / (cell_w * cell_h) — same fraction for any WxH.
Pose class (thin/sparse/tall/compact/long) sets the expected fraction via
scale_profile.json pose_fill_mult.
"""
from __future__ import annotations

import argparse
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
from sprite_metrics import (
    expected_fill_pct,
    load_profile,
    measure_image,
    zoom_for_target_fill,
)

CLEAN = Path(__file__).resolve().parents[1] / "assets" / "sprites" / "cat" / "clean"
TOP_ALIGN = {"clingTop-0", "clingTop-1", "hangDrop-0", "hangDrop-1"}
RIGHT_ALIGN = {"wallLean-0", "wallLean-1"}
TOL = 0.8  # skip if already within ±pp of expected


def content_bbox(im: Image.Image, thr: int = 40):
    return im.split()[-1].point(lambda a: 255 if a >= thr else 0).getbbox()


def place(stem: str, content: Image.Image, cell_w: int, cell_h: int) -> Image.Image:
    nw, nh = content.size
    canvas = Image.new("RGBA", (cell_w, cell_h), (0, 0, 0, 0))
    if stem in TOP_ALIGN:
        x, y = (cell_w - nw) // 2, 1
    elif stem in RIGHT_ALIGN:
        x, y = cell_w - nw - 1, cell_h - nh - 2
    else:
        x, y = (cell_w - nw) // 2, cell_h - nh - 2
    canvas.paste(content, (x, max(0, y)), content)
    return harden_alpha(canvas, thr=40)


def normalize_stem(stem: str, profile: dict, force: bool = False) -> str:
    path = CLEAN / f"{stem}.png"
    if not path.exists():
        return f"{stem}: MISSING"
    im = Image.open(path).convert("RGBA")
    im = keep_largest_blob(im)
    im = harden_alpha(im, thr=40)
    cell_w, cell_h = im.size
    m = measure_image(im, stem)
    target = expected_fill_pct(profile, m.pose_class)
    if not force and abs(m.fill_pct - target) <= TOL:
        return f"{stem}: ok fill={m.fill_pct:.1f}% exp={target:.1f}% ({m.pose_class})"
    bb = content_bbox(im)
    if not bb:
        return f"{stem}: EMPTY"
    content = im.crop(bb)
    cw, ch = content.size
    # opaque count in cropped content ≈ all opaque
    scale = zoom_for_target_fill(m.opaque, cw, ch, target, cell_w, cell_h)
    nw = max(1, int(round(cw * scale)))
    nh = max(1, int(round(ch * scale)))
    fit = min(1.0, (cell_w - 4) / nw, (cell_h - 4) / nh)
    nw = max(1, int(round(nw * fit)))
    nh = max(1, int(round(nh * fit)))
    resized = content.resize((nw, nh), Image.Resampling.LANCZOS)
    out = place(stem, resized, cell_w, cell_h)
    out.save(path)
    after = measure_image(out, stem)
    return (
        f"{stem}: {m.fill_pct:.1f}%→{after.fill_pct:.1f}% "
        f"exp={target:.1f}% {m.pose_class} primary={after.primary}"
    )


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("stems", nargs="*", help="stems to fix; default=all official")
    ap.add_argument("--force", action="store_true")
    ap.add_argument("--no-assemble", action="store_true")
    args = ap.parse_args()
    profile = load_profile()
    if not profile:
        raise SystemExit("scale_profile.json missing — run build_scale_profile.py")
    stems = args.stems or [FRAME_NAMES[i] for i in range(40)] + list(EXTRA_STEMS)
    for stem in stems:
        print(normalize_stem(stem, profile, force=args.force))
    if not args.no_assemble:
        all_stems = [FRAME_NAMES[i] for i in range(40)] + list(EXTRA_STEMS)
        assemble(all_stems, ANIMATIONS)
        from sprite_metrics import build_profile, save_profile

        save_profile(build_profile(all_stems))
        print("sheet + scale_profile refreshed")


if __name__ == "__main__":
    main()
