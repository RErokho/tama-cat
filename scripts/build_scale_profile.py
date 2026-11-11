#!/usr/bin/env python3
"""Build / refresh assets/sprites/cat/scale_profile.json (fill + mass card)."""
from __future__ import annotations

import argparse
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from slice_gemini_sheet import EXTRA_STEMS, FRAME_NAMES  # noqa: E402
from sprite_metrics import (  # noqa: E402
    PROFILE_PATH,
    build_profile,
    save_profile,
)


def official_stems() -> list[str]:
    return [FRAME_NAMES[i] for i in range(40)] + list(EXTRA_STEMS)


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--canon", default="idle-0")
    ap.add_argument("-o", "--output", type=Path, default=PROFILE_PATH)
    args = ap.parse_args()
    profile = build_profile(official_stems(), canon_stem=args.canon)
    path = save_profile(profile, args.output)
    t = profile["targets"]
    print(f"wrote {path}")
    print(
        f"canon={t['canon_stem']} fill={t['fill_pct']}% "
        f"primary={t['primary_px']} mass_h={t['mass_h_px']}"
    )
    print(f"frames={len(profile['frames'])}")
    # quick pose summary
    by: dict[str, list[float]] = {
        "thin": [],
        "sparse": [],
        "tall": [],
        "compact": [],
        "long": [],
    }
    for d in profile["frames"].values():
        by.setdefault(d["pose_class"], []).append(d["fill_pct"])
    for k, vals in by.items():
        if not vals:
            continue
        print(f"  {k:8s} n={len(vals):2d} fill% {min(vals):.1f}..{max(vals):.1f}")
    t = profile["targets"]
    print(f"cell-invariant base fill={t['fill_pct']}% ±{t.get('fill_tol_pct', 3.5)}")
    print(f"pose_fill_mult={t.get('pose_fill_mult')}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
