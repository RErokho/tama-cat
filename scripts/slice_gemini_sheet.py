#!/usr/bin/env python3
"""
Slice 8×5 Gemini sprite sheet, remove labels, professional green-screen key + despill,
height-lock into project cells, assemble sheet.json for the pet.
"""
from __future__ import annotations

import json
import shutil
from collections import deque
from pathlib import Path

from PIL import Image

from process_sprite_frame import (
    FRAME_H,
    FRAME_W,
    keep_largest_blob,
    harden_alpha,
)

ROOT = Path(__file__).resolve().parents[1]
SRC_DEFAULT = Path(
    r"C:\Users\ОтделЭиС\.cursor\projects\c-Users-Desktop-cat\assets"
    r"\Gemini_Generated_Image_qywa7lqywa7lqywa-7f46866b-c96d-4062-a5e0-5d939f838956.jpg"
)
OUT_RAW = ROOT / "assets" / "sprites" / "cat" / "raw" / "gemini"
OUT_CLEAN = ROOT / "assets" / "sprites" / "cat" / "clean"
OUT_SHEET = ROOT / "assets" / "sprites" / "cat"

COLS, ROWS = 8, 5

# Frame index → logical stem used by assemble / animations
FRAME_NAMES: dict[int, str] = {
    0: "idle-0",
    1: "idle-1",
    2: "idle-2",
    3: "blink-0",
    4: "sit-0",
    5: "sit-1",
    6: "look-0",
    7: "yawn-0",
    8: "walk-0",
    9: "walk-1",
    10: "walk-2",
    11: "walk-3",
    12: "walk-4",
    13: "walk-5",
    14: "turn-0",
    15: "alert-0",
    16: "run-0",
    17: "run-1",
    18: "run-2",
    19: "run-3",
    20: "run-4",
    21: "run-5",
    22: "jump-0",
    23: "jump-1",
    24: "fall-0",
    25: "fall-1",
    26: "land-0",
    27: "drag-0",
    28: "drag-1",
    29: "climb-0",
    30: "climb-1",
    31: "climb-2",
    32: "groom-0",
    33: "groom-1",
    34: "groom-2",
    35: "stretch-0",
    36: "happy-0",
    37: "annoyed-0",
    38: "pounce-0",
    39: "pounce-1",
}

# Animations consumed by the current engine (+ extras the player can use later)
ANIMATIONS: dict[str, dict] = {
    "idle": {"frames": ["idle-0", "idle-1", "idle-2", "idle-3"], "fps": 3},
    "blink": {"frames": ["blink-0", "blink-1", "blink-2"], "fps": 8},
    "sit": {"frames": ["sit-0", "sit-1", "sit-2"], "fps": 2},
    "look": {"frames": ["look-0", "look-1"], "fps": 2},
    "yawn": {"frames": ["yawn-0", "yawn-1", "yawn-2"], "fps": 4},
    # walk-2≈walk-1, walk-5≈walk-4 (IoU>~0.97) — dropped as dead frames
    "walk": {"frames": ["walk-0", "walk-1", "walk-2", "walk-3", "walk-4", "walk-5", "walk-6", "walk-7"], "fps": 12},
    "turn": {"frames": ["turn-0", "turn-1"], "fps": 4},
    "alert": {"frames": ["alert-0", "alert-1"], "fps": 3},
    "run": {"frames": ["run-0", "run-1", "run-2", "run-3", "run-4", "run-5"], "fps": 14},
    "jump": {"frames": ["jump-0", "jump-1", "jump-2", "jump-3"], "fps": 10},
    "fall": {"frames": ["fall-0", "fall-1", "fall-2", "fall-3"], "fps": 8},
    # Mid-air variants — MovementEngine may switch between these while falling
    "fallTumble": {"frames": ["fallTumble-0", "fallTumble-1", "fallTumble-2", "fallTumble-3"], "fps": 12},
    "fallCurl": {"frames": ["fallCurl-0", "fallCurl-1", "fallCurl-2"], "fps": 6},
    "fallFlail": {"frames": ["fallFlail-0", "fallFlail-1", "fallFlail-2", "fallFlail-3"], "fps": 10},
    "land": {"frames": ["land-0", "land-1"], "fps": 6},
    "drag": {"frames": ["drag-0", "drag-1", "drag-2"], "fps": 6},
    "climb": {"frames": ["climb-0", "climb-1", "climb-2", "climb-3"], "fps": 8},
    "groom": {"frames": ["groom-0", "groom-1", "groom-2", "groom-3"], "fps": 4},
    "sleep": {"frames": ["sleep-0", "sleep-1", "sleep-2"], "fps": 1},
    "stretch": {"frames": ["stretch-0", "stretch-1", "stretch-2", "stretch-3"], "fps": 4},
    "happy": {"frames": ["happy-0", "happy-1"], "fps": 3},
    "annoyed": {"frames": ["annoyed-0", "annoyed-1"], "fps": 3},
    "pounce": {"frames": ["pounce-0", "pounce-1", "pounce-2"], "fps": 10},
    "clingTop": {"frames": ["clingTop-0", "clingTop-1", "clingTop-2"], "fps": 4},
    "wallLean": {"frames": ["wallLean-0", "wallLean-1", "wallLean-2"], "fps": 3},
    "jumpUp": {"frames": ["jumpUp-0", "jumpUp-1"], "fps": 8},
    "jumpDown": {"frames": ["jumpDown-0", "jumpDown-1"], "fps": 8},
    "lookDown": {"frames": ["lookDown-0", "lookDown-1", "lookDown-2"], "fps": 3},
    "hangDrop": {"frames": ["hangDrop-0", "hangDrop-1", "hangDrop-2"], "fps": 3},
    "climbWindow": {"frames": ["climbWindow-0", "climbWindow-1", "climbWindow-2", "climbWindow-3"], "fps": 6},
    "rub": {"frames": ["rub-0", "rub-1"], "fps": 2},
    "pawKeyboard": {"frames": ["pawKeyboard-0", "pawKeyboard-1"], "fps": 1},
    "bellyUp": {"frames": ["bellyUp-0", "bellyUp-1"], "fps": 1},
    "slowBlink": {"frames": ["slowBlink-0", "slowBlink-1"], "fps": 2},
    "knead": {"frames": ["knead-0", "knead-1", "knead-2"], "fps": 3},
    "attentionPaw": {"frames": ["attentionPaw-0", "attentionPaw-1"], "fps": 2},
    "faceHide": {"frames": ["faceHide-0", "faceHide-1"], "fps": 1},
    "tailChase": {"frames": ["tailChase-0", "tailChase-1", "tailChase-2"], "fps": 4},
}

# Extra stems appended after the original 0–39 sheet cells
EXTRA_STEMS = [
    "tailChase-2",
    "tailChase-1",
    "faceHide-1",
    "attentionPaw-1",
    "knead-2",
    "knead-1",
    "slowBlink-1",
    "bellyUp-1",
    "pawKeyboard-1",
    "rub-1",
    "wallLean-2",
    "clingTop-0",
    "clingTop-1",
    "wallLean-0",
    "wallLean-1",
    "jumpUp-0",
    "jumpDown-0",
    "lookDown-0",
    "lookDown-1",
    "lookDown-2",
    "hangDrop-0",
    "hangDrop-1",
    "climbWindow-0",
    "climbWindow-1",
    "climbWindow-2",
    "idle-3",
    "blink-1",
    "blink-2",
    "sit-2",
    "sleep-0",
    "sleep-1",
    "sleep-2",
    "look-1",
    "yawn-1",
    "yawn-2",
    "alert-1",
    "annoyed-1",
    "happy-1",
    "turn-1",
    "stretch-1",
    "stretch-2",
    "stretch-3",
    "groom-3",
    "land-1",
    "fallFlail-3",
    "drag-2",
    "climb-3",
    "climbWindow-3",
    "clingTop-2",
    "hangDrop-2",
    "pounce-2",
    "fallCurl-2",
    "fall-3",
    "fall-2",
    "jumpDown-1",
    "jumpUp-1",
    "jump-3",
    "jump-2",
    "walk-6",
    "walk-7",
    "rub-0",
    "pawKeyboard-0",
    "bellyUp-0",
    "slowBlink-0",
    "knead-0",
    "attentionPaw-0",
    "faceHide-0",
    "tailChase-0",
    "fallTumble-0",
    "fallTumble-1",
    "fallTumble-2",
    "fallTumble-3",
    "fallCurl-0",
    "fallCurl-1",
    "fallFlail-0",
    "fallFlail-1",
    "fallFlail-2",
]


def cell_box(index: int, sheet_w: int, sheet_h: int) -> tuple[int, int, int, int]:
    col = index % COLS
    row = index // COLS
    x0 = int(round(col * sheet_w / COLS))
    x1 = int(round((col + 1) * sheet_w / COLS))
    y0 = int(round(row * sheet_h / ROWS))
    y1 = int(round((row + 1) * sheet_h / ROWS))
    return x0, y0, x1, y1


def green_screen_key(img: Image.Image) -> Image.Image:
    """
    Professional-style chroma key:
    - key amount from how much G exceeds R/B (screen difference)
    - soft edge then harden
    - despill: pull G toward max(R,B) proportional to key
    """
    rgba = img.convert("RGBA")
    w, h = rgba.size
    px = rgba.load()
    out = Image.new("RGBA", (w, h), (0, 0, 0, 0))
    opx = out.load()

    for y in range(h):
        for x in range(w):
            r, g, b, a = px[x, y]
            # Screen difference: positive when green dominates
            diff = g - max(r, b)
            # Secondary: absolute green high + low rb
            screen = (g - r) * 0.55 + (g - b) * 0.55

            # Hard key core
            if diff >= 28 and g >= 70:
                key = 1.0
            elif diff >= 14 and g >= 55:
                # soft ramp
                key = min(1.0, (diff - 14) / 20.0)
            elif screen >= 40 and g > r + 12 and g > b + 12:
                key = min(1.0, (screen - 40) / 50.0)
            else:
                key = 0.0

            # Near-pure chroma greens even if darkish
            if g >= 90 and r <= 55 and b <= 55 and g > r + 30 and g > b + 30:
                key = max(key, 0.92)

            alpha = int(round(255 * (1.0 - key)))
            if alpha < 20:
                opx[x, y] = (0, 0, 0, 0)
                continue

            # Despill: remove green cast on kept pixels
            spill = key if key > 0 else 0.0
            # Also despill edge pixels that still have green bias
            if g > r + 8 and g > b + 8:
                spill = max(spill, min(1.0, (g - max(r, b)) / 40.0))
            ng = int(round(g * (1.0 - 0.85 * spill) + max(r, b) * 0.85 * spill))
            ng = max(0, min(255, ng))
            # slight restore neutrality
            if spill > 0.15:
                # push toward luminance-preserving non-green
                target = int(0.5 * (r + b))
                ng = int(round(ng * (1.0 - 0.35 * spill) + target * 0.35 * spill))
            opx[x, y] = (r, ng, b, alpha)

    return out


def flood_residual_green(img: Image.Image) -> Image.Image:
    """Flood leftover green from borders only — never eat dark fur/paws."""
    rgba = img.convert("RGBA")
    w, h = rgba.size
    px = rgba.load()
    visited = [[False] * w for _ in range(h)]
    q: deque[tuple[int, int]] = deque()

    def is_kill(r: int, g: int, b: int, a: int) -> bool:
        if a < 12:
            return True
        # green only — black outline/fur must stay
        if g >= 55 and g > r + 16 and g > b + 16:
            return True
        return False

    def seed(x: int, y: int) -> None:
        r, g, b, a = px[x, y]
        if is_kill(r, g, b, a) and not visited[y][x]:
            visited[y][x] = True
            q.append((x, y))

    for x in range(w):
        seed(x, 0)
        seed(x, h - 1)
    for y in range(h):
        seed(0, y)
        seed(w - 1, y)

    while q:
        x, y = q.popleft()
        px[x, y] = (0, 0, 0, 0)
        for dx, dy in ((1, 0), (-1, 0), (0, 1), (0, -1)):
            nx, ny = x + dx, y + dy
            if nx < 0 or ny < 0 or nx >= w or ny >= h or visited[ny][nx]:
                continue
            r, g, b, a = px[nx, ny]
            if not is_kill(r, g, b, a):
                continue
            visited[ny][nx] = True
            q.append((nx, ny))
    return rgba


def scrub_edge_spill(img: Image.Image) -> Image.Image:
    rgba = img.convert("RGBA")
    px = rgba.load()
    w, h = rgba.size
    for y in range(h):
        for x in range(w):
            r, g, b, a = px[x, y]
            if a == 0:
                continue
            empty = 0
            for dx, dy in ((1, 0), (-1, 0), (0, 1), (0, -1)):
                nx, ny = x + dx, y + dy
                if nx < 0 or ny < 0 or nx >= w or ny >= h or px[nx, ny][3] == 0:
                    empty += 1
            if empty == 0:
                continue
            if g > r + 18 and g > b + 18 and g > 60:
                if empty >= 3 and (g - max(r, b)) >= 28:
                    px[x, y] = (0, 0, 0, 0)
                else:
                    px[x, y] = (r, min(g, max(r, b) + 4), b, a)
    return rgba


def fit_uniform_cell(keyed: Image.Image, cell_w: int, cell_h: int, ground_pad: int = 2) -> Image.Image:
    """
    Same scale for every frame (whole source cell, no bbox zoom/crop),
    then drop the opaque content so paws sit on the bottom ground line.
    (Centering left the cat floating after label space became empty.)
    """
    sw, sh = keyed.size
    scale = min(cell_w / max(1, sw), cell_h / max(1, sh))
    nw = max(1, int(round(sw * scale)))
    nh = max(1, int(round(sh * scale)))
    resized = keyed.resize((nw, nh), Image.Resampling.LANCZOS)

    # Place full cell, then shift by content bbox → feet on ground
    tmp = Image.new("RGBA", (cell_w, cell_h), (0, 0, 0, 0))
    x0 = (cell_w - nw) // 2
    y0 = (cell_h - nh) // 2
    tmp.paste(resized, (x0, y0), resized)

    bb = tmp.split()[-1].getbbox()
    canvas = Image.new("RGBA", (cell_w, cell_h), (0, 0, 0, 0))
    if not bb:
        return canvas
    content = tmp.crop(bb)
    cw, ch = content.size
    x = (cell_w - cw) // 2
    y = cell_h - ch - ground_pad
    if y < 0:
        # extremely tall — keep bottom, slight crop top only as last resort
        overflow = -y
        content = content.crop((0, overflow, cw, ch))
        ch = content.size[1]
        y = 0
    canvas.paste(content, (x, y), content)
    return harden_alpha(canvas, thr=40)


def process_cell(cell: Image.Image) -> Image.Image:
    # Keep full cell composition. Labels become separate blobs after key → drop them.
    keyed = green_screen_key(cell)
    keyed = flood_residual_green(keyed)
    keyed = scrub_edge_spill(keyed)
    keyed = keep_largest_blob(keyed)
    keyed = harden_alpha(keyed, thr=40)
    return fit_uniform_cell(keyed, FRAME_W, FRAME_H)


def assemble(stems_in_order: list[str], animations: dict[str, dict]) -> dict:
    sheet_frames: list[Image.Image] = []
    anim_out: dict[str, dict] = {}
    name_to_index: dict[str, int] = {}

    for stem in stems_in_order:
        path = OUT_CLEAN / f"{stem}.png"
        im = Image.open(path).convert("RGBA")
        if im.size != (FRAME_W, FRAME_H):
            canvas = Image.new("RGBA", (FRAME_W, FRAME_H), (0, 0, 0, 0))
            canvas.paste(im, (0, 0), im)
            im = canvas
        name_to_index[stem] = len(sheet_frames)
        sheet_frames.append(im)

    for name, spec in animations.items():
        idxs = [name_to_index[s] for s in spec["frames"] if s in name_to_index]
        if idxs:
            anim_out[name] = {"frames": idxs, "fps": spec["fps"]}

    sheet = Image.new("RGBA", (FRAME_W * len(sheet_frames), FRAME_H), (0, 0, 0, 0))
    for i, im in enumerate(sheet_frames):
        sheet.paste(im, (i * FRAME_W, 0), im)
    sheet.save(OUT_SHEET / "sheet.png")

    meta = {
        "image": "sheet.png",
        "frameWidth": FRAME_W,
        "frameHeight": FRAME_H,
        "scaleNote": "Gemini 8x5 equal cells; uniform letterbox scale; no bbox crop; chroma+despill",
        "source": "gemini-sheet-0-39",
        "animations": anim_out,
        "frameNames": stems_in_order,
        "missing": [],
    }
    (OUT_SHEET / "sheet.json").write_text(
        json.dumps(meta, indent=2, ensure_ascii=False), encoding="utf-8"
    )
    return meta


def main() -> None:
    src = SRC_DEFAULT
    if not src.exists():
        alt = ROOT / "assets" / "sprites" / "cat" / "raw" / "gemini_sheet.jpg"
        src = alt
    sheet = Image.open(src).convert("RGB")
    dest_src = ROOT / "assets" / "sprites" / "cat" / "raw" / "gemini_sheet.jpg"
    dest_src.parent.mkdir(parents=True, exist_ok=True)
    shutil.copy(src, dest_src)

    OUT_RAW.mkdir(parents=True, exist_ok=True)
    OUT_CLEAN.mkdir(parents=True, exist_ok=True)

    sw, sh = sheet.size
    stems: list[str] = []
    report = []
    for idx in range(COLS * ROWS):
        box = cell_box(idx, sw, sh)
        cell = sheet.crop(box)
        stem = FRAME_NAMES[idx]
        stems.append(stem)
        cell.save(OUT_RAW / f"{idx:02d}_{stem}.png")
        fitted = process_cell(cell)
        fitted.save(OUT_CLEAN / f"{stem}.png")
        bb = fitted.split()[-1].getbbox()
        bw = (bb[2] - bb[0]) if bb else 0
        bh = (bb[3] - bb[1]) if bb else 0
        report.append({"i": idx, "stem": stem, "body": [bw, bh]})
        print(f"{idx:02d} {stem}: {bw}x{bh}")

    meta = assemble(stems, ANIMATIONS)
    print(json.dumps({"frames": len(stems), "anims": list(meta["animations"])}, ensure_ascii=False))


if __name__ == "__main__":
    main()
