# Conventions

Project coding and docs conventions that agents must follow.

- Domain OOP-style modules in `crates/cat-core` (`PetCore`, Movement, Behavior, World).
- Do not invent `AnimClip` values without updating `cat-core` types + `sheet.json` + callers.
- After sprite/sheet changes: `cargo run -p desktop-cat` (assets loaded from `assets/sprites/cat`).
- Prefer `docs/BEHAVIOR.md` over inventing new life-need systems.
- Python sprite tools live in `scripts/`; keep clean cells ground-aligned unless cling/hang (top-align).
- Project notes only under this project’s `.cursor/`.
