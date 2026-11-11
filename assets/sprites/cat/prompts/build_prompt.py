#!/usr/bin/env python3
LOCK = """REALISTIC desktop-pet sprite of the EXACT cat in the reference photo (not pixel art, not cartoon chibi). Tortoiseshell/calico short-hair kitten: asymmetrical split face (ginger cheek + dark side), white chest bib, white paw socks, amber-green eyes, dark tail. Soft clean cutout look like a photo-based sticker. Full body. Face ONE direction: nose toward the RIGHT, tail toward the LEFT. Flat solid chroma green #00FF00 background ONLY — no floor, no shadow, no props, no text. Character fills most of the frame, paws near bottom. Clear separate paws and visible tail."""


def build(pose: str, emotion: str, continuity: str) -> str:
    return (
        f"{LOCK}\n"
        f"POSE: {pose}\n"
        f"EMOTION: {emotion}.\n"
        f"CONTINUITY: {continuity}\n"
        f"OUTPUT: single square 1:1 frame."
    )


if __name__ == "__main__":
    print(build(
        "standing idle, all four paws on ground, head level",
        "neutral calm",
        "MASTER idle-0 — establish markings and scale for all later frames.",
    ))
