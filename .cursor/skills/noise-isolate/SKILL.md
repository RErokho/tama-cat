---
name: noise-isolate
description: Forces noisy research (broad search, logs, browser dumps) into a subagent so the main chat keeps only a summary.
---

# Noise isolate

## Purpose

Protect Team Lead context from search/log/DOM floods.

## Trigger

Unknown area, large grep, log triage, browser snapshots, “find where X lives”.

## Steps

1. Do not read dozens of files in the main chat.
2. Spawn `explore` or the matching read-only analyst (`repository-analyst`, etc.) with a tight goal and path budget.
3. Require a short return: locations, 3–7 facts, open questions. Ban pasting raw dumps.
4. For UI verification noise, use the browser subagent/tools; main chat gets pass/fail + selector notes only.
5. Continue orchestration from the summary only.

## Limit

One known file is not noise — edit it yourself.
