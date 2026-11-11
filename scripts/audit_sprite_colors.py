#!/usr/bin/env python3
"""Mean RGB of opaque pixels per clean frame; flag global/anim outliers."""
from __future__ import annotations

import argparse
import json
import math
import statistics as stats
from pathlib import Path

from PIL import Image

ROOT = Path(__file__).resolve().parents[1]
CLEAN = ROOT / "assets" / "sprites" / "cat" / "clean"
SHEET = ROOT / "assets" / "sprites" / "cat" / "sheet.json"

GLOBAL_WARN = 6.0
GLOBAL_BAD = 10.0
ANIM_WARN = 3.5
ANIM_BAD = 7.0


def mean_rgb(path: Path, a_thr: int = 40):
    im = Image.open(path).convert("RGBA")
    px = im.load()
    w, h = im.size
    sr = sg = sb = n = 0
    for y in range(h):
        for x in range(w):
            r, g, b, a = px[x, y]
            if a < a_thr:
                continue
            if g >= 85 and g > r + 22 and g > b + 22:
                continue
            sr += r
            sg += g
            sb += b
            n += 1
    if not n:
        return None
    return sr / n, sg / n, sb / n, n


def dist(a, b):
    return math.sqrt(sum((a[i] - b[i]) ** 2 for i in range(3)))


def luma(r, g, b):
    return 0.2126 * r + 0.7152 * g + 0.0722 * b


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--json", action="store_true")
    args = ap.parse_args()
    meta = json.loads(SHEET.read_text(encoding="utf-8"))
    stems = list(meta["frameNames"])
    anims = meta["animations"]
    rows = []
    for stem in stems:
        p = CLEAN / f"{stem}.png"
        m = mean_rgb(p) if p.exists() else None
        if not m:
            print("MISSING/EMPTY", stem)
            continue
        r, g, b, n = m
        rows.append({"stem": stem, "r": r, "g": g, "b": b, "L": luma(r, g, b), "n": n})

    gr = stats.mean(x["r"] for x in rows)
    gg = stats.mean(x["g"] for x in rows)
    gb = stats.mean(x["b"] for x in rows)
    gL = stats.mean(x["L"] for x in rows)
    for x in rows:
        x["d_global"] = dist((x["r"], x["g"], x["b"]), (gr, gg, gb))
        x["dL"] = abs(x["L"] - gL)

    bad: list[str] = []
    print(f"GLOBAL mean RGB=({gr:.1f},{gg:.1f},{gb:.1f}) L={gL:.1f} frames={len(rows)}")
    print(
        f"GLOBAL stdev R={stats.pstdev(x['r'] for x in rows):.1f} "
        f"G={stats.pstdev(x['g'] for x in rows):.1f} "
        f"B={stats.pstdev(x['b'] for x in rows):.1f}"
    )
    print("\n| stem | RGB | L | dRGB | dL | flag |")
    print("|---|---|---|---|---|---|")
    for x in sorted(rows, key=lambda z: -z["d_global"]):
        flag = ""
        if x["d_global"] >= GLOBAL_BAD or x["dL"] >= 14:
            flag = "BAD"
            bad.append(x["stem"])
        elif x["d_global"] >= GLOBAL_WARN or x["dL"] >= 10:
            flag = "warn"
        print(
            f"| {x['stem']} | ({x['r']:.0f},{x['g']:.0f},{x['b']:.0f}) | "
            f"{x['L']:.0f} | {x['d_global']:.1f} | {x['dL']:.1f} | {flag} |"
        )

    print("\n### Per animation")
    for aname, adef in sorted(anims.items()):
        raw_frames = adef.get("frames") or []
        frames: list[str] = []
        for f in raw_frames:
            if isinstance(f, int):
                if 0 <= f < len(stems):
                    frames.append(stems[f])
            else:
                frames.append(str(f))
        subset = [x for x in rows if x["stem"] in frames]
        if len(subset) < 2:
            continue
        ar = stats.mean(x["r"] for x in subset)
        ag = stats.mean(x["g"] for x in subset)
        ab = stats.mean(x["b"] for x in subset)
        aL = stats.mean(x["L"] for x in subset)
        maxd = 0.0
        print(f"\n**{aname}** mean RGB=({ar:.1f},{ag:.1f},{ab:.1f})")
        for x in sorted(
            subset, key=lambda z: -dist((z["r"], z["g"], z["b"]), (ar, ag, ab))
        ):
            d = dist((x["r"], x["g"], x["b"]), (ar, ag, ab))
            dL = abs(x["L"] - aL)
            maxd = max(maxd, d)
            flag = ""
            if d >= ANIM_BAD or dL >= 12:
                flag = "BAD"
                bad.append(x["stem"])
            elif d >= ANIM_WARN or dL >= 8:
                flag = "warn"
            print(f"- {x['stem']}: dRGB={d:.1f} dL={dL:.1f} {flag}".rstrip())
        print(f"  max_dRGB={maxd:.1f}")

    uniq = sorted(set(bad))
    print("\n### BAD stems to re-render")
    print(", ".join(uniq) if uniq else "(none)")
    if args.json:
        print(json.dumps({"global": [gr, gg, gb], "bad": uniq}, indent=2))


if __name__ == "__main__":
    main()
