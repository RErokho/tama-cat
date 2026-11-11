---
name: tasks
description: Records and validates an AI Dev Team task as JSON under .cursor. Read before non-point work.
---

# Tasks

Do not record a point file edit. Everything else — one JSON per task.

Path: `.cursor/ai-dev-team/tasks/TASK-<number>.json` in the project where the work runs. Do not put a foreign task into the core.

Required fields: `id`, `status`, `priority`, `title`, `request`, `interpretation`, `acceptance`, `assignees`, `history`, `budget`.
`budget.effortCap` — only low, medium, or high. `budget.maxAttempts` — 2.

From `planned` onward also require `contract` (skill `task-contract`): goal, context, inputs, constraints, allowed_files, forbidden_files, acceptance, report_format (brief preset), risks, unfinished, evidence. Prefer `max_reply: short`.

Ownership:
- `allowed_files` / `contract.allowed_files` — editable paths. Empty only for read-only research.
- `forbidden_files` — do-not-touch.
- `packets` — file-owned slices (skill `work-packet`). No overlapping allows across packets.
- Optional `execution`: `{ "mode": "shared"|"worktree", "isolation": "disabled"|"preferred"|"required" }`.

Statuses: see `tasks/lifecycle.js` (includes `failed`, `verified`). Use lifecycle transitions; do not jump to `completed` without `verified`.

Validate with `.cursor/ai-dev-team/tasks/validate.js` after each status change.

Append facts to `history` only. Do not mark completed until gates close (`quality-gates` / `run-gates`).
