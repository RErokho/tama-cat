#!/usr/bin/env python3
"""QA one animation's clean frames before accepting a regen batch.

Checks:
  - all stems exist, unique, not used by other anims in sheet.json (optional)
  - no bright fringe/halo near opaque edge
  - consecutive frames: fill%/mass_h/primary/mean-RGB not wildly different
  - optional: reject if mean RGB far from idle-0 canon
"""
from __future__ import annotations

import argparse
import json
import math
import sys
from pathlib import Path

from PIL import Image

sys.path.insert(0, str(Path(__file__).resolve().parent))
from sprite_metrics import measure_path

ROOT = Path(__file__).resolve().parents[1]
CLEAN = ROOT / "assets" / "sprites" / "cat" / "clean"
SHEET = ROOT / "assets" / "sprites" / "cat" / "sheet.json"
PLAN = ROOT / "assets" / "sprites" / "cat" / "prompts" / "anim_regen_plan.json"
ALPHA = 40


def mean_rgb(path: Path):
    im = Image.open(path).convert("RGBA")
    px = im.load()
    w, h = im.size
    sr = sg = sb = n = 0
    for y in range(h):
        for x in range(w):
            r, g, b, a = px[x, y]
            if a < ALPHA:
                continue
            sr += r
            sg += g
            sb += b
            n += 1
    if not n:
        return None
    return sr / n, sg / n, sb / n


def halo_score(path: Path) -> float:
    """Bright near-transparent fringe ratio (white outline detector)."""
    im = Image.open(path).convert("RGBA")
    px = im.load()
    w, h = im.size
    fringe = 0
    edge = 0
    for y in range(h):
        for x in range(w):
            r, g, b, a = px[x, y]
            if a < 8 or a >= 220:
                continue
            # semi-transparent pixel adjacent to opaque?
            neigh = False
            for dy in (-1, 0, 1):
                for dx in (-1, 0, 1):
                    nx, ny = x + dx, y + dy
                    if 0 <= nx < w and 0 <= ny < h and px[nx, ny][3] >= 220:
                        neigh = True
                        break
                if neigh:
                    break
            if not neigh:
                continue
            edge += 1
            if r > 200 and g > 200 and b > 200:
                fringe += 1
            elif (r + g + b) / 3 > 190 and a < 160:
                fringe += 1
    if edge <= 0:
        return 0.0
    return fringe / edge


def dist(a, b):
    return math.sqrt(sum((a[i] - b[i]) ** 2 for i in range(3)))


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("anim")
    ap.add_argument("--max-halo", type=float, default=0.08)
    ap.add_argument("--max-fill-delta", type=float, default=4.0)
    ap.add_argument("--max-primary-delta", type=int, default=14)
    ap.add_argument("--max-drgb-consec", type=float, default=12.0)
    ap.add_argument("--max-drgb-canon", type=float, default=14.0)
    ap.add_argument("--json", action="store_true")
    args = ap.parse_args()

    plan = json.loads(PLAN.read_text(encoding="utf-8"))
    spec = plan["animations"].get(args.anim)
    if not spec:
        print(f"unknown anim {args.anim}")
        return 2
    stems = list(spec["frames"])
    issues = []
    rows = []

    # uniqueness across plan
    owner = {}
    for aname, adef in plan["animations"].items():
        for s in adef["frames"]:
            if s in owner and owner[s] != aname:
                issues.append(("reuse_plan", s, f"{owner[s]} and {aname}"))
            owner[s] = aname

    canon = mean_rgb(CLEAN / "idle-0.png") if (CLEAN / "idle-0.png").exists() else None

    prev = None
    for stem in stems:
        path = CLEAN / f"{stem}.png"
        if not path.exists():
            issues.append(("missing", stem, str(path)))
            continue
        m = measure_path(path, stem)
        rgb = mean_rgb(path)
        halo = halo_score(path)
        row = {
            "stem": stem,
            "fill": m.fill_pct,
            "primary": m.primary,
            "mass_h": m.mass_h,
            "pose": m.pose_class,
            "rgb": None if not rgb else [round(x, 1) for x in rgb],
            "halo": round(halo, 4),
        }
        rows.append(row)
        if halo > args.max_halo:
            issues.append(("halo", stem, f"{halo:.3f}>{args.max_halo}"))
        if canon and rgb and args.anim != "idle":
            d = dist(rgb, canon)
            if d > args.max_drgb_canon:
                issues.append(("color_canon", stem, f"dRGB={d:.1f}"))
        if prev:
            if abs(m.fill_pct - prev["fill"]) > args.max_fill_delta:
                issues.append(
                    (
                        "consec_fill",
                        stem,
                        f"{prev['fill']:.1f}->{m.fill_pct:.1f}",
                    )
                )
            if abs(m.primary - prev["primary"]) > args.max_primary_delta:
                issues.append(
                    (
                        "consec_primary",
                        stem,
                        f"{prev['primary']}->{m.primary}",
                    )
                )
            if prev.get("rgb") and rgb:
                d = dist(rgb, tuple(prev["rgb"]))
                if d > args.max_drgb_consec:
                    issues.append(("consec_color", stem, f"dRGB={d:.1f}"))
        prev = {"fill": m.fill_pct, "primary": m.primary, "rgb": rgb}

    ok = not issues
    report = {
        "anim": args.anim,
        "ok": ok,
        "desc": spec.get("desc"),
        "frames": rows,
        "issues": [{"rule": a, "stem": b, "detail": c} for a, b, c in issues],
    }
    if args.json:
        print(json.dumps(report, ensure_ascii=False, indent=2))
    else:
        print(f"anim={args.anim} ok={ok} frames={len(rows)} issues={len(issues)}")
        print("desc:", spec.get("desc"))
        for r in rows:
            print(
                f"  {r['stem']:16} fill={r['fill']:5.1f}% prim={r['primary']:3d} "
                f"mh={r['mass_h']:3d} halo={r['halo']:.3f} RGB={r['rgb']}"
            )
        for it in report["issues"]:
            print(f"  [BAD] {it['stem']}: {it['rule']} {it['detail']}")
    return 0 if ok else 1


if __name__ == "__main__":
    raise SystemExit(main())
