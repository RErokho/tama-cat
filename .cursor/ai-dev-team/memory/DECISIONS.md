# Decisions

Durable project decisions. One bullet or short block per decision. Do not re-ask these in chat.

- Product: Windows desktop living cat (Shimeji-like). Not a tamagotchi (no hunger/shop/leveling).
- Primary implementation: **Rust** Cargo workspace at **repo root** (`Cargo.toml` + `crates/`). Electron/TS/npm removed.
- Behavior summary: `docs/BEHAVIOR.md`.
- Sprite sheet is the runtime source of truth: `assets/sprites/cat/sheet.png` + `sheet.json`; frame cell 200×128.
- Facing: assets face RIGHT; renderer/main flips by `facing`.
- Window tops use WinAPI EnumWindows with z-order occlusion; climbable `windowSide` only if some top segment is visible.
- Display clamp is per-monitor work area, not a multi-monitor union AABB.
- Sprite generation: Cursor GenerateImage + local chroma pipeline — not Pollinations/cloud image APIs.
- Verify gate: `cargo build -p desktop-cat`.
- Autonomous jump/climb height caps: up 140px, down 320px, climb 160px (`crates/cat-core/src/config.rs`).
