#!/usr/bin/env python3
"""Chroma-key, edge clean, size-stable scale, pad to cell."""
from __future__ import annotations

import argparse
import json
from collections import deque
from pathlib import Path

from PIL import Image

FRAME_H = 128
FRAME_W = 200
CANON_BODY_H = 112
GROUND_PAD = 4
MIN_BODY_W = 96
ALPHA_HARD = 80


def is_green(r: int, g: int, b: int) -> bool:
    if g >= 85 and g > r + 22 and g > b + 22:
        return True
    if g >= 130 and r < 130 and b < 130 and g >= r + 15 and g >= b + 15:
        return True
    if g > 150 and g >= r and g >= b and (g - min(r, b)) > 18:
        return True
    return False


def luma(r: int, g: int, b: int) -> float:
    return 0.2126 * r + 0.7152 * g + 0.0722 * b


def spread(r: int, g: int, b: int) -> int:
    return max(r, g, b) - min(r, g, b)


def is_bg_pixel(r: int, g: int, b: int, a: int) -> bool:
    """Background only — must NOT match dark tortoiseshell fur."""
    if a == 0:
        return True
    if is_green(r, g, b):
        return True
    # Pure black / charcoal studio backdrop (tight — fur has texture/warmth)
    L = luma(r, g, b)
    sp = spread(r, g, b)
    if L <= 14 and sp <= 10:
        return True
    if L <= 22 and sp <= 6:
        return True
    return False


def chroma_key(img: Image.Image) -> Image.Image:
    rgba = img.convert("RGBA")
    px = rgba.load()
    w, h = rgba.size
    for y in range(h):
        for x in range(w):
            r, g, b, a = px[x, y]
            if is_green(r, g, b):
                px[x, y] = (0, 0, 0, 0)
    return rgba


def flood_bg(img: Image.Image) -> Image.Image:
    rgba = img.convert("RGBA")
    w, h = rgba.size
    px = rgba.load()
    visited = [[False] * w for _ in range(h)]
    q: deque[tuple[int, int]] = deque()

    def try_seed(x: int, y: int) -> None:
        r, g, b, a = px[x, y]
        if is_bg_pixel(r, g, b, a) and not visited[y][x]:
            visited[y][x] = True
            q.append((x, y))

    for x in range(w):
        try_seed(x, 0)
        try_seed(x, h - 1)
    for y in range(h):
        try_seed(0, y)
        try_seed(w - 1, y)

    while q:
        x, y = q.popleft()
        px[x, y] = (0, 0, 0, 0)
        for dx, dy in ((1, 0), (-1, 0), (0, 1), (0, -1)):
            nx, ny = x + dx, y + dy
            if nx < 0 or ny < 0 or nx >= w or ny >= h or visited[ny][nx]:
                continue
            r, g, b, a = px[nx, ny]
            if not is_bg_pixel(r, g, b, a):
                continue
            visited[ny][nx] = True
            q.append((nx, ny))
    return rgba


def scrub_edge_halos(img: Image.Image) -> Image.Image:
    """Remove dark/green anti-alias rings stuck to the silhouette."""
    rgba = img.convert("RGBA")
    px = rgba.load()
    w, h = rgba.size
    kill: list[tuple[int, int]] = []
    for y in range(h):
        for x in range(w):
            r, g, b, a = px[x, y]
            if a == 0:
                continue
            empty = 0
            for dx, dy in ((1, 0), (-1, 0), (0, 1), (0, -1), (1, 1), (-1, -1), (1, -1), (-1, 1)):
                nx, ny = x + dx, y + dy
                if nx < 0 or ny < 0 or nx >= w or ny >= h or px[nx, ny][3] == 0:
                    empty += 1
            if empty == 0:
                continue
            L = luma(r, g, b)
            sp = spread(r, g, b)
            if is_green(r, g, b):
                kill.append((x, y))
            elif empty >= 4 and L < 40 and sp < 18:
                kill.append((x, y))
            elif empty >= 3 and g > r + 18 and g > b + 18 and g > 60:
                kill.append((x, y))
            # White / bright fringe (common GenerateImage anti-alias outline)
            elif empty >= 1 and r >= 175 and g >= 175 and b >= 175 and a < 240:
                kill.append((x, y))
            elif empty >= 2 and L >= 190 and sp < 35:
                kill.append((x, y))
            elif empty >= 1 and L >= 210 and a < 200:
                kill.append((x, y))
    for x, y in kill:
        px[x, y] = (0, 0, 0, 0)
    return rgba


def despill_green(img: Image.Image) -> Image.Image:
    rgba = img.convert("RGBA")
    px = rgba.load()
    w, h = rgba.size
    for y in range(h):
        for x in range(w):
            r, g, b, a = px[x, y]
            if a == 0:
                continue
            edge = False
            for dx, dy in ((1, 0), (-1, 0), (0, 1), (0, -1)):
                nx, ny = x + dx, y + dy
                if nx < 0 or ny < 0 or nx >= w or ny >= h or px[nx, ny][3] == 0:
                    edge = True
                    break
            if g > r + 20 and g > b + 20 and g > 70:
                if edge and g > r + 32 and g > b + 32:
                    px[x, y] = (0, 0, 0, 0)
                else:
                    px[x, y] = (r, min(g, max(r, b) + 5), b, a)
    return rgba


def keep_largest_blob(img: Image.Image, min_keep_frac: float = 0.02) -> Image.Image:
    rgba = img.convert("RGBA")
    w, h = rgba.size
    px = rgba.load()
    visited = [[False] * w for _ in range(h)]
    blobs: list[list[tuple[int, int]]] = []
    for y in range(h):
        for x in range(w):
            if visited[y][x] or px[x, y][3] < 8:
                continue
            comp: list[tuple[int, int]] = []
            q: deque[tuple[int, int]] = deque([(x, y)])
            visited[y][x] = True
            while q:
                cx, cy = q.popleft()
                comp.append((cx, cy))
                for dx, dy in ((1, 0), (-1, 0), (0, 1), (0, -1)):
                    nx, ny = cx + dx, cy + dy
                    if nx < 0 or ny < 0 or nx >= w or ny >= h or visited[ny][nx]:
                        continue
                    if px[nx, ny][3] < 8:
                        continue
                    visited[ny][nx] = True
                    q.append((nx, ny))
            blobs.append(comp)
    if not blobs:
        return rgba
    blobs.sort(key=len, reverse=True)
    total = sum(len(b) for b in blobs)
    keep: set[tuple[int, int]] = set(blobs[0])
    # keep secondary blobs only if large (e.g. separated paw) — drop film scraps
    for b in blobs[1:]:
        if len(b) >= max(80, int(total * min_keep_frac)):
            keep.update(b)
    for y in range(h):
        for x in range(w):
            if (x, y) not in keep:
                px[x, y] = (0, 0, 0, 0)
    return rgba


def harden_alpha(img: Image.Image, thr: int = ALPHA_HARD) -> Image.Image:
    rgba = img.convert("RGBA")
    px = rgba.load()
    w, h = rgba.size
    for y in range(h):
        for x in range(w):
            r, g, b, a = px[x, y]
            if a < thr:
                px[x, y] = (0, 0, 0, 0)
            else:
                px[x, y] = (r, g, b, 255)
    return rgba


def bbox_content(img: Image.Image):
    return img.split()[-1].getbbox()


def fit_stable(
    img: Image.Image,
    cell_w: int,
    cell_h: int,
    body_h: int,
    min_body_w: int = MIN_BODY_W,
) -> Image.Image:
    box = bbox_content(img)
    if not box:
        return Image.new("RGBA", (cell_w, cell_h), (0, 0, 0, 0))
    cropped = img.crop(box)
    cw, ch = cropped.size
    scale = body_h / max(1, ch)
    nw = max(1, int(cw * scale))
    nh = max(1, int(ch * scale))

    max_w = int(cell_w * 0.98)
    max_h = cell_h - GROUND_PAD

    # Narrow upright poses (drag/climb): fit whole body in cell (no head crop)
    if nw < min_body_w:
        contain = min(max_h / max(1, ch), max_w / max(1, cw))
        nw = max(1, int(cw * contain))
        nh = max(1, int(ch * contain))

    resized = cropped.resize((nw, nh), Image.Resampling.LANCZOS)
    if nw > max_w:
        left = (nw - max_w) // 2
        resized = resized.crop((left, 0, left + max_w, nh))
        nw, nh = resized.size
    if nh > max_h:
        # last resort only — keep vertical center (head+paws)
        top = (nh - max_h) // 2
        resized = resized.crop((0, top, nw, top + max_h))
        nw, nh = resized.size

    canvas = Image.new("RGBA", (cell_w, cell_h), (0, 0, 0, 0))
    x = (cell_w - nw) // 2
    y = cell_h - nh - GROUND_PAD
    canvas.paste(resized, (x, max(0, y)), resized)
    return harden_alpha(canvas)


def process(
    path: Path,
    out: Path,
    cell_w: int = FRAME_W,
    cell_h: int = FRAME_H,
    body_h: int = CANON_BODY_H,
    flip: bool = False,
    use_profile: bool = False,
    rotate_180: bool = False,
    top_align: bool = False,
) -> dict:
    img = Image.open(path)
    keyed = chroma_key(img)
    keyed = flood_bg(keyed)
    keyed = scrub_edge_halos(keyed)
    keyed = despill_green(keyed)
    keyed = keep_largest_blob(keyed)
    keyed = harden_alpha(keyed)
    if flip:
        keyed = keyed.transpose(Image.Transpose.FLIP_LEFT_RIGHT)
    if rotate_180:
        # Front-paw-up hang → hind-paw-up inverted hang (head toward bottom).
        keyed = keyed.transpose(Image.Transpose.ROTATE_180)

    zoom_meta: dict = {
        "use_profile": use_profile,
        "rotate_180": rotate_180,
        "top_align": top_align,
    }
    if use_profile:
        # Late import: metrics card drives zoom (mass_h preferred, fill secondary).
        from sprite_metrics import (  # noqa: WPS433
            expected_fill_pct,
            load_profile,
            measure_image,
            pose_class,
            zoom_for_mass_h,
            zoom_for_target_fill,
        )

        profile = load_profile()
        pre = measure_image(keyed, stem=path.stem)
        box = bbox_content(keyed)
        if profile and box and pre.opaque > 0:
            content = keyed.crop(box)
            cw, ch = content.size
            asp = cw / max(1, ch)
            pre_bbox_fill = (
                100.0 * pre.opaque / max(1, cw * ch) if cw and ch else None
            )
            pclass = pose_class(asp, pre_bbox_fill, path.stem)
            bands = profile["targets"]["mass_h_bands"].get(pclass) or profile["targets"][
                "mass_h_bands"
            ]["compact"]
            target_mh = int((bands["min"] + bands["max"]) / 2)
            scale_m = zoom_for_mass_h(pre.mass_h, cw, ch, target_mh, cell_w, cell_h)
            # Same fill% of THIS cell (any WxH): opaque ≈ expected% * cell_w * cell_h
            target_fill = expected_fill_pct(profile, pclass)
            scale_f = zoom_for_target_fill(pre.opaque, cw, ch, target_fill, cell_w, cell_h)
            # Fill is primary (cell-invariant %); mass_h secondary
            scale = 0.80 * scale_f + 0.20 * scale_m
            body_h = max(8, int(round(ch * scale)))
            zoom_meta.update(
                {
                    "pose_class": pclass,
                    "pre_fill_pct": pre.fill_pct,
                    "pre_mass_h": pre.mass_h,
                    "target_mass_h": target_mh,
                    "target_fill_pct": round(target_fill, 2),
                    "cell": [cell_w, cell_h],
                    "scale_mass": round(scale_m, 4),
                    "scale_fill": round(scale_f, 4),
                    "scale": round(scale, 4),
                    "body_h_used": body_h,
                }
            )

    if top_align:
        fitted = fit_top(keyed, cell_w, cell_h, body_h)
    else:
        fitted = fit_stable(keyed, cell_w, cell_h, body_h)
    out.parent.mkdir(parents=True, exist_ok=True)
    fitted.save(out)
    box = bbox_content(fitted)
    bh = (box[3] - box[1]) if box else 0
    bw = (box[2] - box[0]) if box else 0
    return {
        "out": str(out),
        "bbox": box,
        "body_h": bh,
        "body_w": bw,
        "cell": [cell_w, cell_h],
        "flipped": flip,
        "zoom": zoom_meta,
    }


def fit_top(img: Image.Image, cell_w: int, cell_h: int, body_h: int) -> Image.Image:
    """Fit content and pin to top of cell (cling/hang hind-grip)."""
    box = bbox_content(img)
    if not box:
        return Image.new("RGBA", (cell_w, cell_h), (0, 0, 0, 0))
    content = img.crop(box)
    cw, ch = content.size
    scale = body_h / max(1, ch)
    nw = max(1, int(round(cw * scale)))
    nh = max(1, int(round(ch * scale)))
    fit = min(1.0, (cell_w - 4) / nw, (cell_h - 4) / nh)
    nw = max(1, int(round(nw * fit)))
    nh = max(1, int(round(nh * fit)))
    resized = content.resize((nw, nh), Image.Resampling.LANCZOS)
    canvas = Image.new("RGBA", (cell_w, cell_h), (0, 0, 0, 0))
    x = (cell_w - nw) // 2
    y = 1
    canvas.paste(resized, (x, y), resized)
    return harden_alpha(canvas)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("input")
    ap.add_argument("-o", "--output", required=True)
    ap.add_argument("--cell-w", type=int, default=FRAME_W)
    ap.add_argument("--cell-h", type=int, default=FRAME_H)
    ap.add_argument("--body-h", type=int, default=CANON_BODY_H)
    ap.add_argument("--flip", action="store_true")
    ap.add_argument(
        "--rotate-180",
        action="store_true",
        help="Rotate content 180 (front-hang → hind-hang inverted)",
    )
    ap.add_argument(
        "--top-align",
        action="store_true",
        help="Pin content to top of cell (cling/hang)",
    )
    ap.add_argument(
        "--use-profile",
        action="store_true",
        help="Zoom from assets/sprites/cat/scale_profile.json (mass_h + fill pct)",
    )
    args = ap.parse_args()
    stats = process(
        Path(args.input),
        Path(args.output),
        cell_w=args.cell_w,
        cell_h=args.cell_h,
        body_h=args.body_h,
        flip=args.flip,
        use_profile=args.use_profile,
        rotate_180=args.rotate_180,
        top_align=args.top_align,
    )
    print(json.dumps(stats, ensure_ascii=False))


if __name__ == "__main__":
    main()
