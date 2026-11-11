---
name: quality-gates
description: Picks verification by task type, cheapest commands, and refuses to close without evidence. Use before verified/commit-gate.
---

# Quality gates

Protocol: `gates.core.json`. Commands: `gates.project.json`. Runner: `run-gates`.

## Cheapest menu

1. Read `project-*.mdc` verify map + `gates.project.json` (or `discover-gates`).
2. Prefer `regression-awareness` → `run-gates --only …`.
3. UI → browser vs scenario.
4. No command → say so; do not invent scripts.
5. Evidence → `verification-report.json` (check `summary` in context; full log on disk only).

## By kind

| Kind | Check |
| --- | --- |
| point-edit | Team Lead |
| behavior | requirement + `qa-engineer` |
| multi-file | + `code-reviewer` (not for one-line Lead fixes) |
| security | `security-analyst` and/or `/review-security` |
| ui | browser |
| data | `database-architect` (no live migration) |
| regression-risk | `/review-bugbot` |
| packets / contract / fix-loop | validate + max 2 attempts |

No `completed` without closed gates/report.
