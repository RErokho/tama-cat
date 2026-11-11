---
name: task-contract
description: Builds the Task Contract for non-point handoffs. Use when recording or updating a TASK JSON before delegation.
---

# Task Contract

Required from `planned` onward (`tasks/validate.js`):

| Field | Tier |
| --- | --- |
| `goal`, `context`, `allowed_files`, `forbidden_files`, `constraints`, `acceptance`, `report_format` | Mandatory |
| `inputs`, `risks`, `unfinished` | Conditional (may be `[]`) |
| `evidence` | Completion (fill as work proves) |

Optional: `max_reply: short|normal`.

## report_format

Use presets in `references/report-formats.md` (`writer-brief` / `reviewer-brief` / `analyst-brief` / `gate-brief`). Never bare `Goal/Result`.

## Steps

1. Fill contract when leaving inbox/clarification.
2. Handoff via `handoff-management` (compact levels).
3. On partial/fail: update risks/unfinished/evidence only.
4. Writers need non-empty allows or packets.

## Limit

`context` = facts to start, not discussion history. No invented token counts.
