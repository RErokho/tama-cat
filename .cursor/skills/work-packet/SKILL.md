---
name: work-packet
description: Splits non-point work into file-owned packets so parallel writers never share paths. Use before multi-writer or multi-layer coding waves; supports contract-first depends_on.
---

# Work packets

## Purpose

**Ownership**, not decomposition. `task-decomposition` decides *what* work slices exist; this skill assigns **file ownership** so parallel writers never share paths. Supports contract-first `depends_on`.

## Trigger

More than one writer, more than one layer, overlapping paths, or a new API/types boundary.

## Input

Task id, Task Contract, candidate paths, assignees.

## Who

Team Lead. Uses skill `tasks` / `task-contract` for the parent JSON.

## Steps

1. If the boundary/API/types change: create a **contract packet** first (`kind: "contract"`, architect assignee, allowed_files = interface/type paths). Implement packets list that packet in `depends_on`.
2. List hotspot files. Each file → exactly one packet owner.
3. Write `packets` on the task:

```json
{
  "id": "PKT-1",
  "kind": "contract",
  "title": "User API types",
  "assignee": "software-architect",
  "depends_on": [],
  "allowed_files": ["src/api/user.types.ts"],
  "forbidden_files": ["src/ui/**"],
  "acceptance": ["exported types cover the agreed fields"],
  "verify": ["commands from project-*.mdc if any"]
}
```

4. Validate: no two packets share an `allowed_files` entry. Run `.cursor/ai-dev-team/tasks/validate.js`.
5. Plan dispatch:

```bash
node .cursor/ai-dev-team/execution/dispatch.js path/to/TASK.json
```

Optional on the task:

```json
"execution": { "mode": "shared", "isolation": "disabled" }
```

- `mode`: `shared` (default) | `worktree`
- `isolation`: `disabled` | `preferred` | `required`

Use worktrees only when `useWorktree` is true in the plan (skill `worktree-isolation`).

6. Dispatch only packets whose `depends_on` are done. Same-wave packets must have zero path overlap (`conflicts` from dispatch).
7. Hand each worker the parent `contract` + packet slice (`handoff-management`).
8. On a narrow gap, open repair packet `PKT-R-*`. On failed gate with dirty packet files, offer `packet-rollback` (user must confirm).

## Result

Packet board and a dependency-safe dispatch order (parallel / serial / blocked).

## Limit

Do not use theme buckets. Ownership is files, not vibes. Do not default every task to worktrees.
