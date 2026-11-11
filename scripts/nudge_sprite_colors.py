#!/usr/bin/env python3
"""Shift opaque mean RGB of clean frames toward global (or anim) target."""
from __future__ import annotations

import argparse
import json
import math
import statistics as stats
import sys
from pathlib import Path

from PIL import Image

sys.path.insert(0, str(Path(__file__).resolve().parent))
from slice_gemini_sheet import ANIMATIONS, EXTRA_STEMS, FRAME_NAMES, assemble

ROOT = Path(__file__).resolve().parents[1]
CLEAN = ROOT / "assets" / "sprites" / "cat" / "clean"
SHEET = ROOT / "assets" / "sprites" / "cat" / "sheet.json"
ALPHA_THR = 40


def mean_rgb(im: Image.Image):
    px = im.load()
    w, h = im.size
    sr = sg = sb = n = 0
    for y in range(h):
        for x in range(w):
            r, g, b, a = px[x, y]
            if a < ALPHA_THR:
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


def nudge(im: Image.Image, dr: float, dg: float, db: float) -> Image.Image:
    out = im.copy()
    px = out.load()
    w, h = out.size
    for y in range(h):
        for x in range(w):
            r, g, b, a = px[x, y]
            if a < ALPHA_THR:
                continue
            if g >= 85 and g > r + 22 and g > b + 22:
                continue
            nr = max(0, min(255, int(round(r + dr))))
            ng = max(0, min(255, int(round(g + dg))))
            nb = max(0, min(255, int(round(b + db))))
            px[x, y] = (nr, ng, nb, a)
    return out


def resolve_anim_frames(raw_frames, stems: list[str]) -> list[str]:
    out: list[str] = []
    for f in raw_frames or []:
        if isinstance(f, int):
            if 0 <= f < len(stems):
                out.append(stems[f])
        else:
            out.append(str(f))
    return out


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("stems", nargs="*", help="limit to these stems")
    ap.add_argument("--global-bad", type=float, default=10.0)
    ap.add_argument("--anim-bad", type=float, default=7.0)
    ap.add_argument("--strength", type=float, default=1.0, help="1.0 = full delta")
    ap.add_argument("--dry-run", action="store_true")
    args = ap.parse_args()

    meta = json.loads(SHEET.read_text(encoding="utf-8"))
    stems = list(meta["frameNames"])
    rows = {}
    for stem in stems:
        p = CLEAN / f"{stem}.png"
        if not p.exists():
            continue
        m = mean_rgb(Image.open(p).convert("RGBA"))
        if m:
            rows[stem] = {"r": m[0], "g": m[1], "b": m[2], "n": m[3]}

    gr = stats.mean(x["r"] for x in rows.values())
    gg = stats.mean(x["g"] for x in rows.values())
    gb = stats.mean(x["b"] for x in rows.values())
    target_global = (gr, gg, gb)
    print(f"global mean RGB=({gr:.1f},{gg:.1f},{gb:.1f})")

    anim_means: dict[str, tuple[float, float, float]] = {}
    for aname, adef in (meta.get("animations") or {}).items():
        fs = resolve_anim_frames(adef.get("frames"), stems)
        vals = [rows[s] for s in fs if s in rows]
        if len(vals) >= 2:
            anim_means[aname] = (
                stats.mean(v["r"] for v in vals),
                stats.mean(v["g"] for v in vals),
                stats.mean(v["b"] for v in vals),
            )

    stem_to_anim: dict[str, str] = {}
    for aname, adef in (meta.get("animations") or {}).items():
        for s in resolve_anim_frames(adef.get("frames"), stems):
            stem_to_anim.setdefault(s, aname)

    want = set(args.stems) if args.stems else None
    changed = []
    for stem, x in rows.items():
        if want is not None and stem not in want:
            continue
        cur = (x["r"], x["g"], x["b"])
        dg = dist(cur, target_global)
        target = target_global
        reason = "global"
        aname = stem_to_anim.get(stem)
        if aname and aname in anim_means:
            am = anim_means[aname]
            da = dist(cur, am)
            # Prefer anim center when anim outlier is the main issue
            if da >= args.anim_bad and da >= dg * 0.55:
                target = am
                reason = f"anim:{aname}"
            elif dg < args.global_bad and da < args.anim_bad:
                continue
        elif dg < args.global_bad:
            continue

        if dist(cur, target_global) < args.global_bad and (
            not aname
            or aname not in anim_means
            or dist(cur, anim_means[aname]) < args.anim_bad
        ):
            continue

        # Blend: pull to global always when global-bad; also pull to anim mean
        tr, tg, tb = target_global
        if aname and aname in anim_means and dist(cur, anim_means[aname]) >= args.anim_bad:
            ar, ag, ab = anim_means[aname]
            tr = 0.55 * tr + 0.45 * ar
            tg = 0.55 * tg + 0.45 * ag
            tb = 0.55 * tb + 0.45 * ab
            reason = f"global+anim:{aname}"

        dr = (tr - cur[0]) * args.strength
        dgv = (tg - cur[1]) * args.strength
        db = (tb - cur[2]) * args.strength
        before = dist(cur, target_global)
        print(
            f"{stem}: {reason} dRGB={before:.1f} "
            f"Δ=({dr:+.1f},{dgv:+.1f},{db:+.1f})"
        )
        if args.dry_run:
            continue
        path = CLEAN / f"{stem}.png"
        im = Image.open(path).convert("RGBA")
        out = nudge(im, dr, dgv, db)
        out.save(path)
        after = mean_rgb(out)
        if after:
            print(
                f"  → RGB=({after[0]:.0f},{after[1]:.0f},{after[2]:.0f}) "
                f"dRGB={dist(after[:3], target_global):.1f}"
            )
        changed.append(stem)

    if changed and not args.dry_run:
        all_stems = [FRAME_NAMES[i] for i in range(40)] + list(EXTRA_STEMS)
        assemble(all_stems, ANIMATIONS)
        print(f"nudged {len(changed)} frames, sheet refreshed")
    elif not changed:
        print("nothing to nudge")


if __name__ == "__main__":
    main()
