---
name: handoff-management
description: Builds a compact Task Contract handoff (mandatory/conditional/completion). Use on every Task and when returning a finding to the doer.
---

# Compact handoff

## Purpose

Pass **task state**, not Team Lead chat history. No agent gets context that does not affect its decision, actions, or acceptance.

## Three levels

| Level | Fields | When |
| --- | --- | --- |
| **Mandatory** | goal, context (facts only), allowed_files, forbidden_files, constraints (incl. output bans), acceptance, report_format (expanded), level+reason | Always |
| **Conditional** | inputs, risks, depends_on / packet id, unfinished | Only if non-empty / relevant |
| **Completion** | evidence, verification paths, prior blocker notes | Retries, reviewers, commit-gate |

Do **not** include: model shopping notes, other agents’ essays, whole DECISIONS.md, full gate logs, chat transcript.

## Steps

1. Expand `report_format` preset. Add: `Reply only in report_format. No preamble. No restating this prompt.`
2. Deduplicate: one copy of each constraint; drop repeats already on the agent card protocol.
3. Writers: packet slice + parent goal/acceptance. Reviewers: **diff-first** — changed paths + requirement; not whole repo.
4. Memory: topical bullets only (skill `project-memory`), not entire files.
5. On failure return: symptom + evidence summary + unfinished — same brief format.
6. Bloated return → one short retry naming missing headings.

## Result

One self-contained prompt ≤ what the role needs.
