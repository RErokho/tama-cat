---
name: project-memory
description: Reads or updates topical project memory (decisions, issues, conventions). Use before architecture disputes and after locking a decision.
---

# Project memory

Paths: `.cursor/ai-dev-team/memory/` → `DECISIONS.md`, `KNOWN_ISSUES.md`, `CONVENTIONS.md`.

## Topical load (required)

Do **not** dump whole files into a handoff.

1. Grep/search for terms from the task goal (module, feature, API name).
2. Pass **only matching bullets** (or “none found”).
3. For a one-file UI tweak, skip memory unless a decision clearly applies.
4. Write only locked facts; no secrets; no chat dumps.
5. `CHECKPOINT.md` = compact baton only (`compact-checkpoint`).
