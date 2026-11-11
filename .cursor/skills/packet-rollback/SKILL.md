---
name: packet-rollback
description: Rolls back only the allowed_files of a failed work packet after the user asks to revert. Never git reset --hard. Refuses when dirty files outside the packet make scope ambiguous.
---

# Packet rollback

## Purpose

Restore a clean tree for one failed packet without wiping unrelated dirty work.

## Trigger

User says to roll back / revert the failed packet. Not automatic on gate fail.

## Input

Task id, failed packet id, its `allowed_files` list.

## Who

Team Lead. Prefer Team Lead only.

## Steps

1. Confirm the packet’s `allowed_files`. Refuse if empty or missing.
2. Collect dirty paths: `git status --porcelain`.
3. Partition:
   - **in-packet** — dirty paths matching `allowed_files` (exact or under a directory/glob allow)
   - **outside** — other dirty paths
4. If there are **outside** dirty paths that share a parent directory with an in-packet change and ownership is ambiguous, **stop** and ask the user. Do not guess.
5. Show the in-packet restore list. Leave outside paths untouched.
6. Restore only in-packet paths: `git restore -- <paths>` or `git checkout -- <paths>`. Never `git reset --hard`, never force-push, never delete unlisted files.
7. Update task `contract.unfinished` / `risks` / `history` with restored paths and any refused outside dirties.
8. Re-validate the task JSON (`tasks/validate.js`).

## Result

Failed packet files restored; evidence recorded; outside dirties preserved.

## Limit

No silent auto-rollback from hooks. Escalation first; this skill runs only on ask.
