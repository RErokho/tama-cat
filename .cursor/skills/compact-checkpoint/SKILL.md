---
name: compact-checkpoint
description: Saves task facts and next step before context compaction so work survives summarization. Use when the context is large or compaction is about to run.
---

# Compact checkpoint

## Purpose

Keep the plan after context summarization.

## Trigger

Long session, many tool results, or a `preCompact` hook / manual compact.

## Steps

1. Find the active task under `.cursor/ai-dev-team/tasks/TASK-*.json` (status `in_progress` / `review` / `testing`).
2. Append a history fact: what is done, what is open, gates left, current `allowed_files`.
3. Write `.cursor/ai-dev-team/memory/CHECKPOINT.md` (project `.cursor/` when adapted):

```markdown
# Checkpoint
- Task: <id>
- Done: <bullets>
- Open: <bullets>
- Allowed: <paths>
- Next: <one line>
- Not fact: <guesses>
```
Keep under ~15 lines. No chat dump.

4. After compact, read `CHECKPOINT.md` before new tool calls.

## Result

Recoverable next step without re-deriving the chat.
