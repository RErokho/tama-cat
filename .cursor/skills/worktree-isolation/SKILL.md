---
name: worktree-isolation
description: Optional git worktree isolation for parallel packets. Use when execution.mode is worktree or isolation preferred/required for multi-writer waves. Not for tiny point edits.
---

# Worktree isolation

## Purpose

Physically isolate parallel packet writers so they cannot overwrite each other on a shared working tree.

## When

- `task.execution.mode = "worktree"`, or
- `execution.isolation = "required"|"preferred"` and more than one ready writer packet

Default remains `shared` (packets + ownership only).

## Commands

```bash
node .cursor/ai-dev-team/execution/dispatch.js path/to/TASK.json
node .cursor/ai-dev-team/execution/worktree.js add PKT-1
node .cursor/ai-dev-team/execution/worktree.js remove PKT-1
node .cursor/ai-dev-team/execution/worktree.js list
```

Worktrees live under `.cursor/ai-dev-team/worktrees/<packetId>/`.

## Steps

1. Plan dispatch (`dispatch.js`) — get `parallel` vs `serial` and `useWorktree`.
2. For each parallel writer packet that needs isolation: `worktree.js add <id>`.
3. Hand the agent the worktree path + packet `allowed_files` only.
4. After packet done: collect diff/report from that worktree; Team Lead integrates deliberately.
5. `worktree.js remove <id>` when merged or abandoned. Never `git reset --hard` as cleanup shortcut.

## Limits

- Do not use for one-file point edits.
- Do not auto-resolve merge conflicts across worktrees without the user.
- Windows paths supported; still requires a real git repo.
