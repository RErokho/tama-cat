#!/usr/bin/env python3
"""Assemble per-action horizontal strips + master sheet.json / sheet.png."""
from __future__ import annotations

import json
from pathlib import Path

from PIL import Image

ROOT = Path(__file__).resolve().parents[1]
CLEAN = ROOT / "assets" / "sprites" / "cat" / "clean"
OUT = ROOT / "assets" / "sprites" / "cat"
CELL_W = 200
CELL_H = 128

# Frame order per action row (files in clean/)
ROWS: dict[str, list[str]] = {
    "idle": ["idle-0", "idle-1"],
    "sit": ["sit-0", "sit-1"],
    "walk": ["walk-0", "walk-1", "walk-2", "walk-3"],
    "run": ["run-0", "run-1", "run-2", "run-3", "run-4", "run-5"],
    "sleep": ["sleep-0", "sleep-1"],
    "fall": ["fall-0"],
    "drag": ["drag-0"],
    "climb": ["climb-0", "climb-1"],
    "groom": ["groom-0", "groom-1"],
}

FPS = {
    "idle": 3,
    "sit": 1,
    "walk": 10,
    "run": 14,
    "sleep": 2,
    "fall": 1,
    "drag": 4,
    "climb": 8,
    "groom": 4,
}


def load_cell(stem: str) -> Image.Image | None:
    p = CLEAN / f"{stem}.png"
    if not p.exists():
        return None
    im = Image.open(p).convert("RGBA")
    if im.size != (CELL_W, CELL_H):
        canvas = Image.new("RGBA", (CELL_W, CELL_H), (0, 0, 0, 0))
        x = (CELL_W - im.width) // 2
        y = CELL_H - im.height
        canvas.paste(im, (max(0, x), max(0, y)), im)
        return canvas
    return im


def main():
    sheet_frames: list[Image.Image] = []
    animations: dict = {}
    index = 0
    missing: list[str] = []

    for action, stems in ROWS.items():
        frames_idx: list[int] = []
        strip_imgs: list[Image.Image] = []
        for stem in stems:
            im = load_cell(stem)
            if im is None:
                missing.append(stem)
                im = Image.new("RGBA", (CELL_W, CELL_H), (0, 0, 0, 0))
            strip_imgs.append(im)
            frames_idx.append(index)
            sheet_frames.append(im)
            index += 1

        strip = Image.new("RGBA", (CELL_W * len(strip_imgs), CELL_H), (0, 0, 0, 0))
        for i, im in enumerate(strip_imgs):
            strip.paste(im, (i * CELL_W, 0), im)
        strip.save(OUT / f"strip_{action}.png")
        # Sit: single static frame — dual sit cells looked like sit↔stand flicker
        if action == "sit" and frames_idx:
            animations[action] = {"frames": [frames_idx[0]], "fps": FPS.get(action, 1)}
        else:
            animations[action] = {"frames": frames_idx, "fps": FPS.get(action, 6)}

    animations["blink"] = animations.get("idle", {"frames": [0], "fps": 2})

    sheet = Image.new("RGBA", (CELL_W * len(sheet_frames), CELL_H), (0, 0, 0, 0))
    for i, im in enumerate(sheet_frames):
        sheet.paste(im, (i * CELL_W, 0), im)
    sheet.save(OUT / "sheet.png")

    meta = {
        "image": "sheet.png",
        "frameWidth": CELL_W,
        "frameHeight": CELL_H,
        "canonBodyH": 112,
        "scaleNote": "Height-locked canon; wider poses keep body_h; facing RIGHT; flip in renderer",
        "animations": animations,
        "missing": missing,
    }
    (OUT / "sheet.json").write_text(json.dumps(meta, indent=2, ensure_ascii=False), encoding="utf-8")
    print(json.dumps({"frames": len(sheet_frames), "missing": missing, "actions": list(ROWS)}, ensure_ascii=False))


if __name__ == "__main__":
    main()
