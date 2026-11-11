#!/usr/bin/env python3
"""Sprite occupancy / fill metrics for Desktop Cat.

Industry practice (sprite batch normalize, body-extent scale):
  - do NOT drive zoom only from full bbox (stretch poses shrink the cat);
  - use opaque fill % + alpha-mass height (band covering ~80% of opaque mass);
  - keep a scale_profile.json ("card") and reuse targets when fitting new frames.
"""
from __future__ import annotations

import json
import math
from dataclasses import asdict, dataclass
from pathlib import Path
from typing import Iterable

from PIL import Image

ROOT = Path(__file__).resolve().parents[1]
CLEAN = ROOT / "assets" / "sprites" / "cat" / "clean"
PROFILE_PATH = ROOT / "assets" / "sprites" / "cat" / "scale_profile.json"
FRAME_W = 200
FRAME_H = 128
CELL_AREA = FRAME_W * FRAME_H
ALPHA_THR = 40
MASS_FRAC = 0.80


@dataclass
class FrameMetrics:
    stem: str
    opaque: int
    fill_pct: float
    bw: int
    bh: int
    primary: int
    aspect: float
    mass_h: int
    mass_w: int
    bbox_fill_pct: float
    pose_class: str  # thin | sparse | tall | compact | long


# Open / stretched actions: same aspect as sit can still cover much less of the cell.
SPARSE_STEM_PREFIXES = (
    "climb",
    "climbWindow",
    "jump",
    "jumpUp",
    "jumpDown",
    "fall",
    "fallTumble",
    "fallCurl",
    "fallFlail",
    "lookDown",
    "hangDrop",
)


def pose_class(
    aspect: float,
    bbox_fill_pct: float | None = None,
    stem: str = "",
) -> str:
    """Classify silhouette for expected cell fill%.

    Fill% is always opaque/(cell_w*cell_h) — same fraction for any cell size.
    Pose mult adjusts the *expected* fraction when the cat silhouette differs
    (cling thin, climb/jump sparse, walk long, sit tall).
    """
    prefix = stem.rsplit("-", 1)[0] if stem else ""
    if aspect < 0.55 or prefix in ("clingTop", "hangDrop"):
        return "thin"
    # Named sparse poses win over raw aspect (lookDown/hangDrop/climb…)
    if prefix in SPARSE_STEM_PREFIXES:
        return "sparse"
    # Sitting / upright cycles stay compact even if a frame is a bit elongated
    if prefix in (
        "knead",
        "pawKeyboard",
        "attentionPaw",
        "faceHide",
        "slowBlink",
        "wallLean",
        "tailChase",
    ):
        return "compact"
    # Locomotion / elongated cycles stay one fill band across all frames
    if prefix in ("walk", "run", "stretch", "pounce", "land", "drag", "rub", "bellyUp") or aspect >= 1.45:
        return "long"
    if bbox_fill_pct is not None and bbox_fill_pct < 34.0:
        return "sparse"
    if aspect < 0.75:
        return "tall"
    return "compact"


def _mass_extent(counts: list[int], total: int, frac: float = MASS_FRAC) -> int:
    """Smallest contiguous span covering `frac` of mass along one axis."""
    if total <= 0:
        return 0
    n = len(counts)
    best = n
    need = frac * total
    for i0 in range(n):
        acc = 0
        for i1 in range(i0, n):
            acc += counts[i1]
            if acc >= need:
                best = min(best, i1 - i0 + 1)
                break
    return best


def measure_image(im: Image.Image, stem: str = "", thr: int = ALPHA_THR) -> FrameMetrics:
    rgba = im.convert("RGBA")
    px = rgba.load()
    w, h = rgba.size
    opaque = 0
    rows = [0] * h
    cols = [0] * w
    for y in range(h):
        for x in range(w):
            if px[x, y][3] >= thr:
                opaque += 1
                rows[y] += 1
                cols[x] += 1
    alpha = rgba.split()[-1].point(lambda a: 255 if a >= thr else 0)
    bb = alpha.getbbox()
    if not bb:
        return FrameMetrics(stem, 0, 0.0, 0, 0, 0, 0.0, 0, 0, 0.0, "compact")
    bw, bh = bb[2] - bb[0], bb[3] - bb[1]
    primary = max(bw, bh)
    aspect = bw / max(1, bh)
    mass_h = _mass_extent(rows[bb[1] : bb[3]], opaque)
    mass_w = _mass_extent(cols[bb[0] : bb[2]], opaque)
    cell_area = w * h
    fill_pct = 100.0 * opaque / max(1, cell_area)
    bbox_fill = 100.0 * opaque / max(1, bw * bh)
    return FrameMetrics(
        stem=stem,
        opaque=opaque,
        fill_pct=round(fill_pct, 3),
        bw=bw,
        bh=bh,
        primary=primary,
        aspect=round(aspect, 3),
        mass_h=mass_h,
        mass_w=mass_w,
        bbox_fill_pct=round(bbox_fill, 3),
        pose_class=pose_class(aspect, bbox_fill, stem),
    )


def measure_path(path: Path, stem: str | None = None) -> FrameMetrics:
    return measure_image(Image.open(path), stem=stem or path.stem)


def _percentile(vals: list[float], p: float) -> float:
    if not vals:
        return 0.0
    s = sorted(vals)
    if len(s) == 1:
        return float(s[0])
    i = (len(s) - 1) * p
    lo = int(math.floor(i))
    hi = int(math.ceil(i))
    if lo == hi:
        return float(s[lo])
    return float(s[lo] * (hi - i) + s[hi] * (i - lo))


def bands_from_frames(frames: dict[str, dict]) -> tuple[dict, dict]:
    """Calibrate pose bands from current clean set (p10..p90 + pad)."""
    by: dict[str, list[dict]] = {
        "thin": [],
        "sparse": [],
        "tall": [],
        "compact": [],
        "long": [],
    }
    for d in frames.values():
        by.setdefault(d["pose_class"], []).append(d)

    fill_bands: dict[str, dict] = {}
    mass_bands: dict[str, dict] = {}
    # sensible floors if a class is sparse
    defaults = {
        "compact": {"fill": (11.0, 22.0), "mass": (40, 80)},
        "tall": {"fill": (14.0, 22.0), "mass": (60, 90)},
        "thin": {"fill": (6.0, 12.0), "mass": (60, 90)},
        "sparse": {"fill": (8.0, 15.5), "mass": (40, 80)},
        "long": {"fill": (9.0, 15.5), "mass": (26, 48)},
    }
    for cls, items in by.items():
        dfill, dmass = defaults.get(cls, defaults["compact"]).values()
        if len(items) < 3:
            fill_bands[cls] = {"min": dfill[0], "max": dfill[1]}
            mass_bands[cls] = {"min": dmass[0], "max": dmass[1]}
            continue
        fills = [float(x["fill_pct"]) for x in items]
        masses = [float(x["mass_h"]) for x in items]
        fill_bands[cls] = {
            "min": round(max(5.0, _percentile(fills, 0.10) - 1.0), 2),
            "max": round(_percentile(fills, 0.90) + 1.0, 2),
        }
        mass_bands[cls] = {
            "min": int(max(16, _percentile(masses, 0.10) - 4)),
            "max": int(_percentile(masses, 0.90) + 4),
        }
    for cls in ("thin", "sparse", "tall", "compact", "long"):
        fill_bands.setdefault(cls, {"min": defaults[cls]["fill"][0], "max": defaults[cls]["fill"][1]})
        mass_bands.setdefault(cls, {"min": defaults[cls]["mass"][0], "max": defaults[cls]["mass"][1]})
    return fill_bands, mass_bands


# How much a pose may differ from standing fill% (same % of ANY cell size).
# Example: base 16% of 200×128 == 16% of 120×200 (opaque count scales with area).
POSE_FILL_MULT = {
    "compact": 1.00,  # idle / standing
    "tall": 1.10,  # sit / look / yawn
    "thin": 0.55,  # cling / hang
    "sparse": 0.72,  # climb / jump / fall / wallLean
    "long": 0.75,  # walk / run
}


def default_targets_from_canon(canon: FrameMetrics, frames: dict[str, dict] | None = None) -> dict:
    """Targets derived from a compact standing frame (usually idle-0).

    `fill_pct` is a universal occupancy fraction of the cell area — independent of
    cell pixel size. Example: 30% of 128×128 and 30% of 120×200 means the same
    relative fill; only absolute opaque count scales with cell_w*cell_h.
    """
    if frames:
        _fill_bands_unused, mass_h_bands = bands_from_frames(frames)
    else:
        mass_h_bands = {
            "compact": {"min": 40, "max": 80},
            "tall": {"min": 60, "max": 90},
            "thin": {"min": 60, "max": 90},
            "sparse": {"min": 40, "max": 80},
            "long": {"min": 26, "max": 48},
        }
    base = round(canon.fill_pct, 2)
    # Expected absolute fill% bands = base * pose_mult ± FILL_TOL
    fill_tol = 3.5
    fill_bands: dict[str, dict] = {}
    for cls, mult in POSE_FILL_MULT.items():
        mid = base * mult
        fill_bands[cls] = {
            "min": round(max(4.0, mid - fill_tol), 2),
            "max": round(mid + fill_tol, 2),
            "expected": round(mid, 2),
            "mult": mult,
        }
    return {
        "canon_stem": canon.stem,
        # Universal: opaque / (cell_w * cell_h) * 100 — same for any cell size
        "fill_pct": base,
        "fill_tol_pct": fill_tol,
        "pose_fill_mult": dict(POSE_FILL_MULT),
        "primary_px": canon.primary,
        "mass_h_px": canon.mass_h,
        "mass_w_px": canon.mass_w,
        "fill_bands": fill_bands,
        "mass_h_bands": mass_h_bands,
        "cell_invariant_note": (
            "target opaque = fill_pct/100 * cell_w * cell_h "
            "(e.g. same 16% on 200x128 or 128x128)"
        ),
    }


def expected_fill_pct(profile: dict, pose: str) -> float:
    """Expected fill% of the cell for this pose (canon fill × pose mult)."""
    base = float(profile["targets"]["fill_pct"])
    mults = profile["targets"].get("pose_fill_mult") or POSE_FILL_MULT
    return base * float(mults.get(pose, 1.0))


def zoom_for_target_fill(
    content_opaque: int,
    content_w: int,
    content_h: int,
    target_fill_pct: float,
    cell_w: int = FRAME_W,
    cell_h: int = FRAME_H,
    margin: int = 4,
) -> float:
    """Scale factor so placed content ≈ target_fill_pct of the cell.

    fill ≈ opaque * scale^2 / cell_area  =>  scale = sqrt(target_fill * cell / opaque)
    then clamp so content fits in cell with margin.
    """
    if content_opaque <= 0:
        return 1.0
    cell_area = cell_w * cell_h
    scale = math.sqrt((target_fill_pct / 100.0) * cell_area / content_opaque)
    max_fit = min((cell_w - margin) / max(1, content_w), (cell_h - margin) / max(1, content_h))
    return float(min(scale, max_fit))


def zoom_for_mass_h(
    content_mass_h: int,
    content_w: int,
    content_h: int,
    target_mass_h: int,
    cell_w: int = FRAME_W,
    cell_h: int = FRAME_H,
    margin: int = 4,
) -> float:
    """Preferred zoom driver: match alpha-mass height (body-extent style)."""
    if content_mass_h <= 0:
        return 1.0
    scale = target_mass_h / float(content_mass_h)
    max_fit = min((cell_w - margin) / max(1, content_w), (cell_h - margin) / max(1, content_h))
    return float(min(scale, max_fit))


def build_profile(stems: Iterable[str], canon_stem: str = "idle-0") -> dict:
    frames: dict[str, dict] = {}
    canon: FrameMetrics | None = None
    for stem in stems:
        path = CLEAN / f"{stem}.png"
        if not path.exists():
            continue
        m = measure_path(path, stem)
        frames[stem] = asdict(m)
        if stem == canon_stem:
            canon = m
    if canon is None and frames:
        # fallback: first compact frame
        for stem, d in frames.items():
            if d["pose_class"] == "compact":
                canon = FrameMetrics(**d)
                break
        if canon is None:
            first = next(iter(frames.values()))
            canon = FrameMetrics(**first)
    assert canon is not None
    targets = default_targets_from_canon(canon, frames)
    return {
        "version": 1,
        "cell": [FRAME_W, FRAME_H],
        "targets": targets,
        "frames": frames,
        "notes": (
            "fill_pct = opaque/cell. mass_h = tight band with 80% opaque mass. "
            "Zoom prefer mass_h (body-extent); fill_pct is QA + secondary zoom. "
            "Bands auto-calibrated from clean set (p10..p90). "
            "long poses (walk/run) naturally have lower fill%."
        ),
    }


def load_profile(path: Path = PROFILE_PATH) -> dict | None:
    if not path.exists():
        return None
    return json.loads(path.read_text(encoding="utf-8"))


def save_profile(profile: dict, path: Path = PROFILE_PATH) -> Path:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(profile, ensure_ascii=False, indent=2), encoding="utf-8")
    return path


def expected_fill_for(pose: str, profile: dict) -> tuple[float, float]:
    """Return (min,max) absolute fill% band for pose — cell-size invariant."""
    bands = profile["targets"]["fill_bands"]
    b = bands.get(pose) or bands["compact"]
    if "expected" in b:
        return float(b["min"]), float(b["max"])
    # legacy profiles without expected
    exp = expected_fill_pct(profile, pose)
    tol = float(profile["targets"].get("fill_tol_pct", 3.5))
    return exp - tol, exp + tol


def expected_mass_h_for(pose: str, profile: dict) -> tuple[int, int]:
    bands = profile["targets"]["mass_h_bands"]
    b = bands.get(pose) or bands["compact"]
    return int(b["min"]), int(b["max"])
