---
name: regression-awareness
description: Maps changed files to packages and recommends a minimal gate set instead of running the whole monorepo. Use after implementation before run-gates.
---

# Regression awareness

## Purpose

Run only the checks hit by the diff (especially public surfaces).

## Command

```bash
node .cursor/ai-dev-team/quality/regression-awareness.js .
```

Writes `.cursor/ai-dev-team/quality/regression-report.json` with `recommended_gates` and a `run_hint`.

## Steps

1. Generate the regression report from the current git diff.
2. Pass `--only` those gate ids to `run-gates` (skill `run-gates`).
3. If `public_surface_changed` is true, do not drop test/build when those gates exist in `gates.project.json`.
4. Record evidence in the Task Contract.

## Limit

If no gates match, use `quality-gates` / browser — do not invent commands.
