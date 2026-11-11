---
name: adapt-team-to-project
description: Adapts rules to the open or named project when the user asks to adapt rules, adapt the team, or connect. Writes project-*.mdc; does not run an install script.
---

# Adapt rules

Do not run an install script. Write rule files yourself.

## Steps

1. Find the project — details in `references/steps.md`.
2. **Manifest scan first** (package.json, lockfiles, Cargo.toml, go.mod, docker-compose*, workspace files) — extract real scripts/verify commands.
3. Study with read-only analysts in parallel when the area is large.
4. Write short `project-*.mdc` including discovered verify commands (or explicit “no runner”).
5. Seed memory stubs + `gates.project.json` per `profile/layout.json` (no essay profile dumps).
6. Without an explicit full rebuild, only fix stale facts.
7. Reply brief: project path, changed rules, verify map.

Do not rewrite core team rules or the constitution. Files only under the target project’s `.cursor/`.
