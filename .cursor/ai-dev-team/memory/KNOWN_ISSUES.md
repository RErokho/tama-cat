# Known issues

Known defects, footguns, and “do not try again” notes for this project.

- Do not re-register/unregister koffi EnumWindows callbacks every poll — reuse one callback (`native-win32.ts`); frequent register caused intermittent native errors.
- `sleep` animation must not reuse `stretch` frames (looks like broken stretch).
- Props in generated frames (monitors, ledges, poles) must be rejected/regenerated before assemble.
- `clingTop` with large top padding in the cell looks like a white gap at the screen top — keep top-aligned content.
- Graphify CLI is not installed here; do not invent a graph — read files. No `graphify-out/graph.json` yet.
- Do not reintroduce Electron/`package.json` unless the user explicitly asks.
