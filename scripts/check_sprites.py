#!/usr/bin/env python3
"""
Sprite gate for Desktop Cat.

Checks clean cells + sheet against project rules:
  - required stems present, cell 200×128
  - primary size max(bw,bh) ≈ 108
  - fill% / mass_h vs scale_profile.json (pose bands)
  - mean RGB consistency (global + per animation)
  - residual chroma green
  - multi-blob / likely props
  - top-align (clingTop/hangDrop), right-align (wallLean)
  - sheet.json ↔ clean consistency

Usage:
  python scripts/check_sprites.py
  python scripts/check_sprites.py --new
  python scripts/check_sprites.py --stems lookDown-0 jumpDown-0
  scripts\\check_sprites.bat
"""
from __future__ import annotations

import argparse
import json
import math
import statistics as stats
import sys
import time
from collections import deque
from dataclasses import asdict, dataclass, field
from pathlib import Path

from PIL import Image

sys.path.insert(0, str(Path(__file__).resolve().parent))
from process_sprite_frame import is_green  # noqa: E402
from slice_gemini_sheet import (  # noqa: E402
    ANIMATIONS,
    EXTRA_STEMS,
    FRAME_H,
    FRAME_NAMES,
    FRAME_W,
)
from sprite_metrics import (  # noqa: E402
    PROFILE_PATH,
    build_profile,
    expected_fill_for,
    expected_fill_pct,
    expected_mass_h_for,
    load_profile,
    measure_path,
    save_profile,
)

ROOT = Path(__file__).resolve().parents[1]
CLEAN = ROOT / "assets" / "sprites" / "cat" / "clean"
SHEET_JSON = ROOT / "assets" / "sprites" / "cat" / "sheet.json"
SHEET_PNG = ROOT / "assets" / "sprites" / "cat" / "sheet.png"

TARGET_PRIMARY = 108
PRIMARY_WARN = 10  # fill-led sizing; primary is advisory
PRIMARY_BAD = 14  # only extreme outliers (still warn-level in practice)
MASS_H_BAND_SLACK_WARN = 2  # outside calibrated band → warn
MASS_H_BAND_SLACK_BAD = 7  # further outside band → bad
# fill% of cell (opaque/(w*h)) — same % for any cell size; tol ≈ 3–4 pp
FILL_BAND_SLACK_WARN = 2.0
FILL_BAND_SLACK_BAD = 0.0  # band already includes ±3.5 around expected
ANIM_FILL_SPREAD_WARN = 2.5  # same pose_class within anim
ANIM_FILL_SPREAD_BAD = 3.5
ANIM_MASS_H_SPREAD_WARN = 5  # vs anim median (same pose_class only)
ANIM_MASS_H_SPREAD_BAD = 7
# gallop/walk gather frames are naturally flatter — allow more mass_h spread
ANIM_MASS_H_SPREAD_BAD_LONG = 10
ANIM_MASS_H_SPREAD_WARN_LONG = 7

# color — strongly tightened (Euclidean dRGB on opaque mean)
GLOBAL_WARN = 6.0
GLOBAL_BAD = 10.0
ANIM_WARN = 3.5
ANIM_BAD = 7.0
LUMA_WARN = 4.0
LUMA_BAD = 7.0

GREEN_WARN = 40
GREEN_BAD = 200
SECOND_BLOB_RATIO_WARN = 0.12
SECOND_BLOB_RATIO_BAD = 0.22
TOP_GAP_BAD = 6
RIGHT_GAP_BAD = 6
ALPHA_THR = 40
NEAR_DUP_IOU = 0.96  # consecutive frames this similar are dead weight
# intentional micro-motion (blink/idle) — do not flag
NEAR_DUP_ALLOW = {
    ("idle-0", "idle-1"),
    ("idle-1", "idle-2"),
    ("idle-0", "blink-0"),
    ("blink-0", "idle-0"),
    ("sit-0", "sit-1"),
    ("sit-1", "sit-0"),
}

TOP_ALIGN = {"clingTop-0", "clingTop-1", "hangDrop-0", "hangDrop-1"}
RIGHT_ALIGN = {"wallLean-0", "wallLean-1"}


@dataclass
class Issue:
    stem: str
    rule: str
    level: str  # warn | bad
    detail: str


@dataclass
class Report:
    ok: bool
    global_rgb: list[float] = field(default_factory=list)
    checked: list[str] = field(default_factory=list)
    issues: list[Issue] = field(default_factory=list)
    summary: str = ""


def official_stems() -> list[str]:
    return [FRAME_NAMES[i] for i in range(40)] + list(EXTRA_STEMS)


def resolve_anim_frames(raw_frames, stems: list[str]) -> list[str]:
    out: list[str] = []
    for f in raw_frames or []:
        if isinstance(f, int):
            if 0 <= f < len(stems):
                out.append(stems[f])
        else:
            out.append(str(f))
    return out


def content_bbox(im: Image.Image, thr: int = ALPHA_THR):
    return im.split()[-1].point(lambda a: 255 if a >= thr else 0).getbbox()


def mean_rgb(im: Image.Image, a_thr: int = ALPHA_THR):
    px = im.load()
    w, h = im.size
    sr = sg = sb = n = 0
    green = 0
    for y in range(h):
        for x in range(w):
            r, g, b, a = px[x, y]
            if a < a_thr:
                continue
            if is_green(r, g, b):
                green += 1
                continue
            sr += r
            sg += g
            sb += b
            n += 1
    if not n:
        return None, green
    return (sr / n, sg / n, sb / n, n), green


def dist(a, b) -> float:
    return math.sqrt(sum((a[i] - b[i]) ** 2 for i in range(3)))


def luma(r: float, g: float, b: float) -> float:
    return 0.2126 * r + 0.7152 * g + 0.0722 * b


def alpha_mask(path: Path, thr: int = ALPHA_THR) -> list[list[bool]] | None:
    if not path.exists():
        return None
    im = Image.open(path).convert("RGBA")
    px = im.load()
    w, h = im.size
    return [[px[x, y][3] >= thr for x in range(w)] for y in range(h)]


def mask_iou(a: list[list[bool]], b: list[list[bool]]) -> float:
    h = min(len(a), len(b))
    w = min(len(a[0]), len(b[0])) if h else 0
    inter = union = 0
    for y in range(h):
        for x in range(w):
            aa, bb = a[y][x], b[y][x]
            if aa or bb:
                union += 1
            if aa and bb:
                inter += 1
    return inter / union if union else 1.0


def second_blob_ratio(im: Image.Image, thr: int = ALPHA_THR) -> float:
    """Area of 2nd-largest opaque blob / largest. Props often leave a 2nd mass."""
    px = im.load()
    w, h = im.size
    seen = [[False] * w for _ in range(h)]
    areas: list[int] = []

    def opaque(x: int, y: int) -> bool:
        r, g, b, a = px[x, y]
        return a >= thr and not is_green(r, g, b)

    for y in range(h):
        for x in range(w):
            if seen[y][x] or not opaque(x, y):
                continue
            q: deque[tuple[int, int]] = deque([(x, y)])
            seen[y][x] = True
            area = 0
            while q:
                cx, cy = q.popleft()
                area += 1
                for nx, ny in (
                    (cx - 1, cy),
                    (cx + 1, cy),
                    (cx, cy - 1),
                    (cx, cy + 1),
                ):
                    if 0 <= nx < w and 0 <= ny < h and not seen[ny][nx] and opaque(nx, ny):
                        seen[ny][nx] = True
                        q.append((nx, ny))
            areas.append(area)
    if len(areas) < 2:
        return 0.0
    areas.sort(reverse=True)
    if areas[0] <= 0:
        return 0.0
    return areas[1] / areas[0]


def pick_stems(args: argparse.Namespace) -> tuple[list[str], str]:
    """Returns (stems, mode) where mode is all|stems|new."""
    base = official_stems()
    if args.stems:
        return list(dict.fromkeys(args.stems)), "stems"
    if args.new:
        sheet_mtime = SHEET_PNG.stat().st_mtime if SHEET_PNG.exists() else 0.0
        cutoff = time.time() - args.new_hours * 3600
        newer: list[str] = []
        for stem in base:
            p = CLEAN / f"{stem}.png"
            if not p.exists():
                newer.append(stem)
                continue
            mt = p.stat().st_mtime
            # touched after last sheet assemble, or within --new-hours
            if mt > sheet_mtime + 1 or mt >= cutoff:
                newer.append(stem)
        if not newer:
            return base, "new-fallback-all"
        return newer, "new"
    return base, "all"


def check(stems: list[str], verbose: bool = True, rebuild_profile: bool = False) -> Report:
    issues: list[Issue] = []
    checked: list[str] = []
    color_rows: list[dict] = []

    profile = load_profile()
    if profile is None or rebuild_profile:
        profile = build_profile(official_stems())
        save_profile(profile)
        if verbose:
            print(f"scale_profile → {PROFILE_PATH}")

    # --- sheet presence ---
    if not SHEET_JSON.exists():
        issues.append(Issue("*", "sheet", "bad", f"missing {SHEET_JSON}"))
    if not SHEET_PNG.exists():
        issues.append(Issue("*", "sheet", "bad", f"missing {SHEET_PNG}"))

    sheet_stems: list[str] = []
    sheet_anims = dict(ANIMATIONS)
    if SHEET_JSON.exists():
        meta = json.loads(SHEET_JSON.read_text(encoding="utf-8"))
        sheet_stems = list(meta.get("frameNames") or [])
        sheet_anims = meta.get("animations") or sheet_anims
        for stem in official_stems():
            if stem not in sheet_stems:
                issues.append(Issue(stem, "sheet_frames", "bad", "not listed in sheet.json frameNames"))
        # junk stems in sheet (outside official)
        official = set(official_stems())
        for stem in sheet_stems:
            if stem not in official and not stem.startswith("_"):
                issues.append(Issue(stem, "sheet_junk", "warn", "frame in sheet but not in official roster"))

        for aname, adef in sheet_anims.items():
            for f in adef.get("frames") or []:
                if isinstance(f, int):
                    if not (0 <= f < len(sheet_stems)):
                        issues.append(
                            Issue("*", "anim_index", "bad", f"{aname}: frame index {f} out of range")
                        )
                else:
                    if str(f) not in sheet_stems and str(f) not in official:
                        issues.append(
                            Issue(str(f), "anim_ref", "bad", f"{aname}: unknown frame name")
                        )

    # --- per-frame geometry / props / chroma ---
    for stem in stems:
        path = CLEAN / f"{stem}.png"
        checked.append(stem)
        if not path.exists():
            issues.append(Issue(stem, "missing", "bad", f"no file {path}"))
            continue
        im = Image.open(path).convert("RGBA")
        if im.size != (FRAME_W, FRAME_H):
            issues.append(
                Issue(stem, "cell_size", "bad", f"got {im.size}, want {FRAME_W}x{FRAME_H}")
            )

        bb = content_bbox(im)
        if not bb:
            issues.append(Issue(stem, "empty", "bad", "no opaque content"))
            continue
        bw, bh = bb[2] - bb[0], bb[3] - bb[1]
        primary = max(bw, bh)
        # Cell-invariant fill% is the hard size gate. primary~108 is advisory
        # only when fill is also outside its band (otherwise fill-led by design).
        m_early = measure_path(path, stem)
        fmin_e, fmax_e = expected_fill_for(m_early.pose_class, profile)
        fill_ok = fmin_e <= m_early.fill_pct <= fmax_e
        d_primary = abs(primary - TARGET_PRIMARY)
        if d_primary >= PRIMARY_WARN and not fill_ok:
            issues.append(
                Issue(
                    stem,
                    "primary_size",
                    "warn",
                    f"max(bw,bh)={primary} (want ~{TARGET_PRIMARY}; fill-led)",
                )
            )

        if stem in TOP_ALIGN and bb[1] > TOP_GAP_BAD:
            issues.append(
                Issue(stem, "top_align", "bad", f"content top y={bb[1]} > {TOP_GAP_BAD}")
            )
        if stem in RIGHT_ALIGN and (FRAME_W - bb[2]) > RIGHT_GAP_BAD:
            issues.append(
                Issue(
                    stem,
                    "right_align",
                    "bad",
                    f"right gap={FRAME_W - bb[2]} > {RIGHT_GAP_BAD}",
                )
            )

        mean, green = mean_rgb(im)
        if green >= GREEN_BAD:
            issues.append(Issue(stem, "chroma_green", "bad", f"{green} green pixels left"))
        elif green >= GREEN_WARN:
            issues.append(Issue(stem, "chroma_green", "warn", f"{green} green pixels left"))

        # fill% = opaque/(cell_w*cell_h) — same target % on any cell size
        m = m_early
        fmin, fmax = expected_fill_for(m.pose_class, profile)
        exp = expected_fill_pct(profile, m.pose_class)
        if m.fill_pct < fmin or m.fill_pct > fmax:
            issues.append(
                Issue(
                    stem,
                    "fill_pct",
                    "bad",
                    f"{m.fill_pct:.1f}% pose={m.pose_class} expected≈{exp:.1f}% "
                    f"band={fmin:.1f}..{fmax:.1f} (cell-relative %)",
                )
            )
        elif abs(m.fill_pct - exp) >= FILL_BAND_SLACK_WARN:
            issues.append(
                Issue(
                    stem,
                    "fill_pct",
                    "warn",
                    f"{m.fill_pct:.1f}% pose={m.pose_class} expected≈{exp:.1f}% "
                    f"(Δ{abs(m.fill_pct - exp):.1f})",
                )
            )
        mh_min, mh_max = expected_mass_h_for(m.pose_class, profile)
        if m.mass_h < mh_min - MASS_H_BAND_SLACK_BAD or m.mass_h > mh_max + MASS_H_BAND_SLACK_BAD:
            issues.append(
                Issue(
                    stem,
                    "mass_h",
                    "bad",
                    f"mass_h={m.mass_h} pose={m.pose_class} band={mh_min}..{mh_max}",
                )
            )
        elif (
            m.mass_h < mh_min - MASS_H_BAND_SLACK_WARN
            or m.mass_h > mh_max + MASS_H_BAND_SLACK_WARN
        ):
            issues.append(
                Issue(
                    stem,
                    "mass_h",
                    "warn",
                    f"mass_h={m.mass_h} pose={m.pose_class} band={mh_min}..{mh_max}",
                )
            )

        ratio = second_blob_ratio(im)
        if ratio >= SECOND_BLOB_RATIO_BAD:
            issues.append(
                Issue(stem, "prop_blob", "bad", f"2nd blob ratio={ratio:.2f} (likely prop)")
            )
        elif ratio >= SECOND_BLOB_RATIO_WARN:
            issues.append(
                Issue(stem, "prop_blob", "warn", f"2nd blob ratio={ratio:.2f}")
            )

        if mean:
            r, g, b, n = mean
            color_rows.append(
                {
                    "stem": stem,
                    "r": r,
                    "g": g,
                    "b": b,
                    "L": luma(r, g, b),
                    "n": n,
                    "bw": bw,
                    "bh": bh,
                    "primary": primary,
                    "green": green,
                    "blob2": ratio,
                }
            )

    # --- color consistency (vs all official that exist, not only --stems) ---
    all_rows: list[dict] = []
    for stem in official_stems():
        path = CLEAN / f"{stem}.png"
        if not path.exists():
            continue
        im = Image.open(path).convert("RGBA")
        mean, _ = mean_rgb(im)
        if not mean:
            continue
        r, g, b, n = mean
        all_rows.append({"stem": stem, "r": r, "g": g, "b": b, "L": luma(r, g, b), "n": n})

    global_rgb: list[float] = []
    if all_rows:
        gr = stats.mean(x["r"] for x in all_rows)
        gg = stats.mean(x["g"] for x in all_rows)
        gb = stats.mean(x["b"] for x in all_rows)
        gL = stats.mean(x["L"] for x in all_rows)
        global_rgb = [gr, gg, gb]
        focus = set(stems)
        for x in all_rows:
            if x["stem"] not in focus:
                continue
            d = dist((x["r"], x["g"], x["b"]), (gr, gg, gb))
            dL = abs(x["L"] - gL)
            if d >= GLOBAL_BAD or dL >= LUMA_BAD:
                issues.append(
                    Issue(
                        x["stem"],
                        "color_global",
                        "bad",
                        f"dRGB={d:.1f} dL={dL:.1f} RGB=({x['r']:.0f},{x['g']:.0f},{x['b']:.0f})",
                    )
                )
            elif d >= GLOBAL_WARN or dL >= LUMA_WARN:
                issues.append(
                    Issue(
                        x["stem"],
                        "color_global",
                        "warn",
                        f"dRGB={d:.1f} dL={dL:.1f} RGB=({x['r']:.0f},{x['g']:.0f},{x['b']:.0f})",
                    )
                )

        # per-animation: use sheet anims if present else canon ANIMATIONS
        anim_source = sheet_anims if sheet_anims else ANIMATIONS
        name_list = sheet_stems if sheet_stems else official_stems()
        by_stem = {x["stem"]: x for x in all_rows}
        for aname, adef in sorted(anim_source.items()):
            frames = resolve_anim_frames(adef.get("frames"), name_list)
            # only flag if any focused stem is in this anim
            if focus.isdisjoint(frames):
                continue
            subset = [by_stem[s] for s in frames if s in by_stem]
            if len(subset) < 2:
                continue
            ar = stats.mean(x["r"] for x in subset)
            ag = stats.mean(x["g"] for x in subset)
            ab = stats.mean(x["b"] for x in subset)
            aL = stats.mean(x["L"] for x in subset)
            for x in subset:
                if x["stem"] not in focus:
                    continue
                d = dist((x["r"], x["g"], x["b"]), (ar, ag, ab))
                dL = abs(x["L"] - aL)
                if d >= ANIM_BAD or dL >= LUMA_BAD:
                    issues.append(
                        Issue(
                            x["stem"],
                            "color_anim",
                            "bad",
                            f"{aname}: dRGB={d:.1f} dL={dL:.1f}",
                        )
                    )
                elif d >= ANIM_WARN or dL >= LUMA_WARN:
                    issues.append(
                        Issue(
                            x["stem"],
                            "color_anim",
                            "warn",
                            f"{aname}: dRGB={d:.1f} dL={dL:.1f}",
                        )
                    )

    # within-animation mass_h + fill% spread — only same pose_class
    anim_source = sheet_anims if sheet_anims else ANIMATIONS
    name_list = sheet_stems if sheet_stems else official_stems()
    focus = set(stems)
    for aname, adef in sorted(anim_source.items()):
        frames = resolve_anim_frames(adef.get("frames"), name_list)
        if len(frames) < 2:
            continue
        by_class: dict[str, list[tuple[str, int, float]]] = {}
        for stem in frames:
            p = CLEAN / f"{stem}.png"
            if not p.exists():
                continue
            m = measure_path(p, stem)
            by_class.setdefault(m.pose_class, []).append((stem, m.mass_h, m.fill_pct))
        for pclass, rows in by_class.items():
            if len(rows) < 2:
                continue
            med_mh = stats.median(v for _, v, _ in rows)
            med_fill = stats.median(v for _, _, v in rows)
            bad_lim = (
                ANIM_MASS_H_SPREAD_BAD_LONG
                if pclass == "long"
                else ANIM_MASS_H_SPREAD_BAD
            )
            warn_lim = (
                ANIM_MASS_H_SPREAD_WARN_LONG
                if pclass == "long"
                else ANIM_MASS_H_SPREAD_WARN
            )
            for stem, mh, fill in rows:
                if stem not in focus:
                    continue
                drift = abs(mh - med_mh)
                if drift >= bad_lim:
                    issues.append(
                        Issue(
                            stem,
                            "anim_size",
                            "bad",
                            f"{aname}/{pclass}: mass_h={mh} vs median {med_mh:.0f} "
                            f"(Δ{drift:.0f}≥{bad_lim})",
                        )
                    )
                elif drift >= warn_lim:
                    issues.append(
                        Issue(
                            stem,
                            "anim_size",
                            "warn",
                            f"{aname}/{pclass}: mass_h={mh} vs median {med_mh:.0f} (Δ{drift:.0f})",
                        )
                    )
                dfill = abs(fill - med_fill)
                if dfill >= ANIM_FILL_SPREAD_BAD:
                    issues.append(
                        Issue(
                            stem,
                            "anim_fill",
                            "bad",
                            f"{aname}/{pclass}: fill={fill:.1f}% vs median {med_fill:.1f}% "
                            f"(Δ{dfill:.1f}≥{ANIM_FILL_SPREAD_BAD})",
                        )
                    )
                elif dfill >= ANIM_FILL_SPREAD_WARN:
                    issues.append(
                        Issue(
                            stem,
                            "anim_fill",
                            "warn",
                            f"{aname}/{pclass}: fill={fill:.1f}% vs median {med_fill:.1f}% "
                            f"(Δ{dfill:.1f})",
                        )
                    )

    # consecutive near-duplicate frames inside animations
    mask_cache: dict[str, list[list[bool]] | None] = {}
    for aname, adef in sorted(anim_source.items()):
        frames = resolve_anim_frames(adef.get("frames"), name_list)
        if len(frames) < 2:
            continue
        for i in range(len(frames) - 1):
            a, b = frames[i], frames[i + 1]
            if (a, b) in NEAR_DUP_ALLOW or (b, a) in NEAR_DUP_ALLOW:
                continue
            if a not in mask_cache:
                mask_cache[a] = alpha_mask(CLEAN / f"{a}.png")
            if b not in mask_cache:
                mask_cache[b] = alpha_mask(CLEAN / f"{b}.png")
            ma, mb = mask_cache[a], mask_cache[b]
            if not ma or not mb:
                continue
            if a == b:
                issues.append(
                    Issue(a, "anim_dup", "bad", f"{aname}: exact consecutive duplicate")
                )
                continue
            j = mask_iou(ma, mb)
            if j >= NEAR_DUP_IOU:
                issues.append(
                    Issue(
                        f"{a}|{b}",
                        "anim_near_dup",
                        "bad",
                        f"{aname}[{i}→{i+1}] IoU={j:.3f} — drop one frame",
                    )
                )

    bad_n = sum(1 for i in issues if i.level == "bad")
    warn_n = sum(1 for i in issues if i.level == "warn")
    ok = bad_n == 0
    summary = f"checked={len(checked)} bad={bad_n} warn={warn_n}"
    if global_rgb:
        summary += f" global_rgb=({global_rgb[0]:.0f},{global_rgb[1]:.0f},{global_rgb[2]:.0f})"

    if verbose:
        print(f"Desktop Cat sprite check — {summary}")
        if color_rows and len(stems) <= 12:
            print("\nFocused frames:")
            for x in color_rows:
                print(
                    f"  {x['stem']:16s} primary={x['primary']:3d} "
                    f"RGB=({x['r']:.0f},{x['g']:.0f},{x['b']:.0f}) "
                    f"green={x['green']} blob2={x['blob2']:.2f}"
                )
        if issues:
            print("\nIssues:")
            for i in sorted(issues, key=lambda z: (0 if z.level == "bad" else 1, z.stem, z.rule)):
                print(f"  [{i.level.upper():4s}] {i.stem:16s} {i.rule}: {i.detail}")
        else:
            print("\nNo issues.")
        print(
            "\nRules: cell 200x128 | primary~108 | fill%/mass_h bands | color dRGB | "
            "no green/props | cling top / wall right"
        )
        print(
            "Fix: re-render → process_sprite_frame.py [--use-profile] → "
            "normalize_sprite_sizes.py → build_scale_profile / check"
        )

    return Report(ok=ok, global_rgb=global_rgb, checked=checked, issues=issues, summary=summary)


def main() -> int:
    ap = argparse.ArgumentParser(description="Check Desktop Cat sprites against project rules")
    ap.add_argument("--stems", nargs="+", help="Check only these stems")
    ap.add_argument(
        "--new",
        action="store_true",
        help="Check frames newer than sheet.png (or touched recently); falls back to all",
    )
    ap.add_argument("--new-hours", type=float, default=24.0, help="With --new: also include files touched within N hours")
    ap.add_argument("--json", type=Path, help="Write machine report JSON")
    ap.add_argument("--quiet", action="store_true")
    ap.add_argument("--fail-warn", action="store_true", help="Exit 1 on warnings too")
    ap.add_argument(
        "--rebuild-profile",
        action="store_true",
        help="Rebuild assets/sprites/cat/scale_profile.json from clean cells",
    )
    args = ap.parse_args()

    stems, mode = pick_stems(args)
    if not args.quiet:
        print(f"mode={mode} stems={len(stems)}")
        if mode == "new-fallback-all":
            print("(no files newer than sheet / within --new-hours; checking all)")

    report = check(stems, verbose=not args.quiet, rebuild_profile=args.rebuild_profile)
    if args.json:
        payload = {
            "ok": report.ok,
            "summary": report.summary,
            "global_rgb": report.global_rgb,
            "checked": report.checked,
            "issues": [asdict(i) for i in report.issues],
        }
        args.json.parent.mkdir(parents=True, exist_ok=True)
        args.json.write_text(json.dumps(payload, ensure_ascii=False, indent=2), encoding="utf-8")
        if not args.quiet:
            print(f"\nWrote {args.json}")

    if not report.ok:
        return 1
    if args.fail_warn and any(i.level == "warn" for i in report.issues):
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
