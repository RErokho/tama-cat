#!/usr/bin/env python3
"""Fix clean frames: hybrid size (fill band + primary~108) + color nudge + assemble."""
from __future__ import annotations

import math
import statistics as stats
import sys
from pathlib import Path

from PIL import Image

sys.path.insert(0, str(Path(__file__).resolve().parent))
from nudge_sprite_colors import dist, mean_rgb, nudge, resolve_anim_frames
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
    expected_fill_for,
    expected_fill_pct,
    expected_mass_h_for,
    load_profile,
    measure_image,
    save_profile,
    build_profile,
)

CLEAN = Path(__file__).resolve().parents[1] / "assets" / "sprites" / "cat" / "clean"
SHEET = Path(__file__).resolve().parents[1] / "assets" / "sprites" / "cat" / "sheet.json"
TARGET_PRIMARY = 108
TOP_ALIGN = {"clingTop-0", "clingTop-1", "hangDrop-0", "hangDrop-1"}
RIGHT_ALIGN = {"wallLean-0", "wallLean-1"}
GLOBAL_WARN = 6.0
ANIM_WARN = 3.5


def content_bbox(im: Image.Image, thr: int = 40):
    return im.split()[-1].point(lambda a: 255 if a >= thr else 0).getbbox()


def place(stem: str, content: Image.Image) -> Image.Image:
    nw, nh = content.size
    canvas = Image.new("RGBA", (FRAME_W, FRAME_H), (0, 0, 0, 0))
    if stem in TOP_ALIGN:
        x, y = (FRAME_W - nw) // 2, 1
    elif stem in RIGHT_ALIGN:
        x, y = FRAME_W - nw - 1, FRAME_H - nh - 2
    else:
        x, y = (FRAME_W - nw) // 2, FRAME_H - nh - 2
    canvas.paste(content, (x, max(0, y)), content)
    return harden_alpha(canvas, thr=40)


CANON_LOCK = {"idle-0", "idle-1", "idle-2", "blink-0"}  # keep base fill% stable


def rescale_hybrid(stem: str, profile: dict) -> str:
    path = CLEAN / f"{stem}.png"
    im = Image.open(path).convert("RGBA")
    im = keep_largest_blob(im)
    im = harden_alpha(im, thr=40)
    m = measure_image(im, stem)
    if stem in CANON_LOCK:
        return f"{stem}: locked fill={m.fill_pct:.1f}% prim={m.primary}"
    fmin, fmax = expected_fill_for(m.pose_class, profile)
    exp = expected_fill_pct(profile, m.pose_class)
    mh_min, mh_max = expected_mass_h_for(m.pose_class, profile)

    bb = content_bbox(im)
    if not bb:
        return f"{stem}: EMPTY"
    content = im.crop(bb)
    cw, ch = content.size
    F = max(0.01, m.fill_pct)
    s_lo = math.sqrt(fmin / F)
    s_hi = math.sqrt(fmax / F)
    # prefer primary → 108, else expected fill, else mass_h mid
    s_primary = TARGET_PRIMARY / max(1, m.primary)
    s_fill = math.sqrt(exp / F)
    mh_mid = (mh_min + mh_max) / 2.0
    s_mass = mh_mid / max(1, m.mass_h) if m.mass_h else 1.0
    # weight: stay in band, pull primary + fill + mass
    # Primary/fill first; mass only a light nudge (don't block shrink/grow).
    s_want = 0.55 * s_primary + 0.40 * s_fill + 0.05 * s_mass
    # Prefer expected fill when already near primary
    if abs(m.primary - TARGET_PRIMARY) <= 3:
        s_want = 0.25 * s_primary + 0.65 * s_fill + 0.10 * s_mass
    s = max(s_lo, min(s_hi, s_want))
    # also clamp to cell fit
    max_fit = min((FRAME_W - 4) / cw, (FRAME_H - 4) / ch)
    s = min(s, max_fit)

    near_primary = abs(m.primary - TARGET_PRIMARY) <= 2
    near_fill = abs(m.fill_pct - exp) <= 1.0
    if abs(s - 1.0) < 0.012 and near_primary and near_fill and fmin <= m.fill_pct <= fmax:
        return f"{stem}: size ok fill={m.fill_pct:.1f}% prim={m.primary} mh={m.mass_h}"

    nw = max(1, int(round(cw * s)))
    nh = max(1, int(round(ch * s)))
    resized = content.resize((nw, nh), Image.Resampling.LANCZOS)
    out = place(stem, resized)
    out.save(path)
    a = measure_image(out, stem)
    return (
        f"{stem}: s={s:.3f} fill {m.fill_pct:.1f}→{a.fill_pct:.1f}% "
        f"prim {m.primary}→{a.primary} mh {m.mass_h}→{a.mass_h} ({m.pose_class})"
    )


def color_fix_all() -> list[str]:
    import json

    meta = json.loads(SHEET.read_text(encoding="utf-8"))
    stems = list(meta["frameNames"])
    rows = {}
    for stem in stems:
        p = CLEAN / f"{stem}.png"
        if not p.exists():
            continue
        m = mean_rgb(Image.open(p).convert("RGBA"))
        if m:
            rows[stem] = {"r": m[0], "g": m[1], "b": m[2]}

    gr = stats.mean(x["r"] for x in rows.values())
    gg = stats.mean(x["g"] for x in rows.values())
    gb = stats.mean(x["b"] for x in rows.values())
    gmean = (gr, gg, gb)

    anim_means = {}
    stem_to_anim = {}
    for aname, adef in (meta.get("animations") or {}).items():
        fs = resolve_anim_frames(adef.get("frames"), stems)
        vals = [rows[s] for s in fs if s in rows]
        for s in fs:
            stem_to_anim.setdefault(s, aname)
        if len(vals) >= 2:
            anim_means[aname] = (
                stats.mean(v["r"] for v in vals),
                stats.mean(v["g"] for v in vals),
                stats.mean(v["b"] for v in vals),
            )

    changed = []
    for stem, x in rows.items():
        cur = (x["r"], x["g"], x["b"])
        dg = dist(cur, gmean)
        aname = stem_to_anim.get(stem)
        da = dist(cur, anim_means[aname]) if aname in anim_means else 0.0
        if dg < GLOBAL_WARN and da < ANIM_WARN:
            continue
        tr, tg, tb = gmean
        reason = "global"
        if aname in anim_means and da >= ANIM_WARN:
            ar, ag, ab = anim_means[aname]
            # pull toward both when needed
            w_anim = 0.5 if dg >= GLOBAL_WARN else 0.75
            tr = (1 - w_anim) * tr + w_anim * ar
            tg = (1 - w_anim) * tg + w_anim * ag
            tb = (1 - w_anim) * tb + w_anim * ab
            reason = f"global+anim:{aname}"
        dr, dgg, db = tr - cur[0], tg - cur[1], tb - cur[2]
        print(f"color {stem}: {reason} dG={dg:.1f} dA={da:.1f} Δ=({dr:+.1f},{dgg:+.1f},{db:+.1f})")
        path = CLEAN / f"{stem}.png"
        out = nudge(Image.open(path).convert("RGBA"), dr, dgg, db)
        out.save(path)
        changed.append(stem)
    return changed


def main() -> None:
    profile = load_profile()
    if not profile:
        raise SystemExit("missing scale_profile.json")
    stems = [FRAME_NAMES[i] for i in range(40)] + list(EXTRA_STEMS)
    print("=== size hybrid ===")
    for stem in stems:
        if (CLEAN / f"{stem}.png").exists():
            print(rescale_hybrid(stem, profile))
    # refresh profile after size so color uses new pixels; size bands stay from canon
    profile = build_profile(stems)
    save_profile(profile)
    print("=== color ===")
    changed = color_fix_all()
    print(f"color nudged: {len(changed)}")
    assemble(stems, ANIMATIONS)
    save_profile(build_profile(stems))
    print("sheet + profile refreshed")


if __name__ == "__main__":
    main()
