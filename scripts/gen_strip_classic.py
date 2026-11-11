#!/usr/bin/env python3
"""
Generate one animation STRIP with EXACT pixel size via Pollinations Classic API.
Example: 5 frames → width=640 height=128 (each cell 128x128).

No API key. Then slice + height-lock into clean cells.
"""
from __future__ import annotations

import argparse
import json
import urllib.parse
import urllib.request
from pathlib import Path

from PIL import Image

# import sibling
import sys

sys.path.insert(0, str(Path(__file__).resolve().parent))
from process_sprite_frame import CANON_BODY_H, FRAME_H, FRAME_W, process  # noqa: E402
from slice_strip import slice_raw  # noqa: E402

ROOT = Path(__file__).resolve().parents[1]
RAW = ROOT / "assets" / "sprites" / "cat" / "raw"
CLEAN = ROOT / "assets" / "sprites" / "cat" / "clean"
REF = ROOT / "assets" / "reference" / "cat-prototype.png"

CELL = 128  # generation cell; process may upscale into FRAME_W x FRAME_H

PASSPORT = (
    "realistic cutout sticker of the EXACT tortoiseshell calico kitten from a reference photo: "
    "asymmetrical split face ginger+dark, white chest bib, white paw socks, amber eyes, dark tail. "
    "NOT pixel art, NOT cartoon chibi. Soft photo-realistic illustration."
)

# Each action: list of per-frame pose lines (length = frame count)
STRIPS: dict[str, list[str]] = {
    "idle": [
        "standing idle, weight on all four separate white-sock paws planted, head level, tail soft up",
        "standing idle variation: soft blink, tip of tail lifts slightly, paws still four distinct planted",
    ],
    "sit": [
        "sitting upright, two front white-sock paws planted separately, hind legs tucked, tail curled",
        "sitting, soft blink, same pose, ears relaxed, front paws still two clear socks",
    ],
    "walk": [
        "walk contact: RIGHT fore + LEFT hind forward planted; other two back; FOUR distinct paws",
        "walk pass: legs gather under body; FOUR distinct white-sock paws visible spaced",
        "walk contact opposite: LEFT fore + RIGHT hind forward; FOUR distinct paws",
        "walk collect: return toward first contact; FOUR distinct paws",
    ],
    "run": [
        "run stretch: body elongated, lead forepaw forward, hind extend; FOUR distinct paws",
        "run gather: legs under body mid-gallop; FOUR distinct paws",
        "run opposite stretch: other diagonal pair extended; FOUR distinct paws",
        "run gather compact; FOUR distinct paws",
        "run stretch again milder; FOUR distinct paws",
        "run closing gather toward cycle start; FOUR distinct paws",
    ],
    "sleep": [
        "sleeping curled loaf, eyes closed, paws tucked, tail around body",
        "sleeping, tiny ear twitch, same curl, eyes closed",
    ],
    "fall": [
        "falling through air, legs splayed, surprised eyes, FOUR paws visible outstretched",
    ],
    "drag": [
        "held dangling, paws hanging down separately, surprised; FOUR paws visible",
    ],
    "climb": [
        "climbing up, front paws reaching up separately, hind pushing; FOUR paws",
        "climb pull-up, body higher, paws on vertical edge; FOUR paws",
    ],
    "groom": [
        "sitting, licking front right paw, other paws planted; paws readable",
        "sitting, wiping face with licked paw; other paws planted",
    ],
}


def build_prompt(action: str, frames: list[str], cell: int) -> str:
    n = len(frames)
    w, h = cell * n, cell
    lines = [
        f"OUTPUT IMAGE SIZE must be exactly {w} pixels wide by {h} pixels tall.",
        f"LAYOUT: a horizontal filmstrip of EXACTLY {n} equal square panels.",
        f"Each panel is exactly {cell}x{cell} pixels.",
        f"Panels are side-by-side with NO gaps, OR separated only by 1px of pure #00FF00.",
        "All panels share ONE ground line near the bottom.",
        "Same character scale in every panel (head size consistent).",
        "ALL cats face RIGHT (nose to the right, tail to the left).",
        "Background of the ENTIRE image is flat solid chroma green #00FF00 only — no floor, no shadow, no text, no borders except green.",
        PASSPORT,
        "PANEL CONTENTS (left to right):",
    ]
    for i, pose in enumerate(frames, 1):
        lines.append(f"Panel {i}/{n}: {pose}.")
    lines.append("Do not add extra panels. Do not crop panels unevenly.")
    return " ".join(lines)


def fetch_strip(prompt: str, width: int, height: int, out: Path, seed: int = 7) -> Path:
    q = urllib.parse.quote(prompt)
    url = (
        f"https://image.pollinations.ai/prompt/{q}"
        f"?width={width}&height={height}&nologo=true&seed={seed}"
    )
    out.parent.mkdir(parents=True, exist_ok=True)
    print("GET", width, "x", height, "→", out.name)
    req = urllib.request.Request(url, headers={"User-Agent": "desktop-cat/0.1"})
    with urllib.request.urlopen(req, timeout=180) as resp:
        data = resp.read()
    out.write_bytes(data)
    # verify size; if wrong, resample canvas carefully
    im = Image.open(out)
    if im.size != (width, height):
        print("warn: got", im.size, "resizing canvas to", (width, height))
        im = im.convert("RGBA").resize((width, height), Image.Resampling.LANCZOS)
        im.save(out)
    return out


def process_action(action: str, cell: int = CELL, flip: bool = False) -> dict:
    frames = STRIPS[action]
    n = len(frames)
    w, h = cell * n, cell
    prompt = build_prompt(action, frames, cell)
    strip_path = RAW / f"strip_{action}_{w}x{h}.png"
    fetch_strip(prompt, w, h, strip_path)

    sliced = slice_raw(strip_path, n, RAW / "_sliced", action)
    results = []
    for i, p in enumerate(sliced):
        dest_raw = RAW / f"{action}-{i}.png"
        Image.open(p).save(dest_raw)
        stats = process(
            dest_raw,
            CLEAN / f"{action}-{i}.png",
            cell_w=FRAME_W,
            cell_h=FRAME_H,
            body_h=CANON_BODY_H,
            flip=flip,
        )
        results.append(stats)
    return {"action": action, "requested": [w, h], "frames": results, "prompt_preview": prompt[:240]}


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("action", choices=[*STRIPS.keys(), "all"])
    ap.add_argument("--cell", type=int, default=CELL)
    ap.add_argument("--flip", action="store_true")
    args = ap.parse_args()

    actions = list(STRIPS) if args.action == "all" else [args.action]
    out = []
    for a in actions:
        out.append(process_action(a, cell=args.cell, flip=args.flip))
    print(json.dumps(out, ensure_ascii=False, indent=2))


if __name__ == "__main__":
    main()
