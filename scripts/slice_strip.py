#!/usr/bin/env python3
"""Slice a horizontal animation strip into equal cells, then height-lock process each."""
from __future__ import annotations

import argparse
import json
from pathlib import Path

from PIL import Image

from process_sprite_frame import FRAME_H, FRAME_W, CANON_BODY_H, process


def slice_raw(path: Path, frames: int, out_dir: Path, prefix: str) -> list[Path]:
    img = Image.open(path).convert("RGBA")
    w, h = img.size
    cell_w = w // frames
    out_dir.mkdir(parents=True, exist_ok=True)
    paths: list[Path] = []
    for i in range(frames):
        crop = img.crop((i * cell_w, 0, (i + 1) * cell_w, h))
        p = out_dir / f"{prefix}-{i}.png"
        crop.save(p)
        paths.append(p)
    return paths


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("strip")
    ap.add_argument("--frames", type=int, required=True)
    ap.add_argument("--prefix", required=True)
    ap.add_argument("--flip", action="store_true", help="flip every cell (if whole strip faces left)")
    ap.add_argument("--raw-dir", default="assets/sprites/cat/raw")
    ap.add_argument("--clean-dir", default="assets/sprites/cat/clean")
    args = ap.parse_args()

    root = Path(__file__).resolve().parents[1]
    raw_dir = root / args.raw_dir
    clean_dir = root / args.clean_dir
    sliced = slice_raw(Path(args.strip), args.frames, raw_dir / "_sliced", args.prefix)

    results = []
    for i, p in enumerate(sliced):
        # also keep jpg/png raw copy for traceability
        dest_raw = raw_dir / f"{args.prefix}-{i}.png"
        Image.open(p).save(dest_raw)
        stats = process(
            dest_raw,
            clean_dir / f"{args.prefix}-{i}.png",
            cell_w=FRAME_W,
            cell_h=FRAME_H,
            body_h=CANON_BODY_H,
            flip=args.flip,
        )
        results.append(stats)
    print(json.dumps({"prefix": args.prefix, "frames": results}, ensure_ascii=False, indent=2))


if __name__ == "__main__":
    main()
