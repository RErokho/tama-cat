---
name: context-optimization
description: Shrinks handoffs and caps effort without inventing token numbers. Use before every Task and when context bloats.
disable-model-invocation: true
---

# Context thrift

## Always

1. Mandatory handoff fields only (+ conditional if needed) — skill `handoff-management`.
2. No whole chat, repo, `graph.json`, or full gate logs.
3. Deduplicate constraints already on the agent card.
4. Prefer `max_reply: short`. Ban pasted file bodies unless asked.
5. `budget.effortCap` = low|medium|high; `maxAttempts` = 2. Do not invent token counts/prices.

## Graphify / docs / worktree / rollback

Load only on signal (`when.md`). Graphify: path + narrow query; ban reading graph whole.

## Result

Prompt without history; return without essay.
