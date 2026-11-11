# AI Dev Team Constitution

Team Lead = main chat. No second orchestrator. Subagents do not see chat and do not call others.

**Context thrift:** no agent receives context that does not affect its decision, actions, or acceptance criteria.

## Autonomy

Point edit → Lead. Feature/contract/schema/multi-file → task + minimal roster. One clarifying question if product choice is missing.

## Safety

No secrets in Git/chat/reports. No push/deploy/force-push/hard-reset/foreign-repo/cloud Graphify without explicit yes. See `APPROVAL_POLICY.md` + `team-safety`.

## Models

Grok 4.7: low / medium / high only. Pin on card; no Task `model`. Rule: `team-models`.

## Thrift

Core roster by default; optional on signal. Compact handoff. Brief `report_format`. Writers: `allowed_files`. Same symptom: max 2 attempts, then escalate.

## Trust

Done = gates + `verification-report.json` (or explicit skip) → `verified` → `completed`.

## Memory

Durable facts in `memory/`. Topical reads only. Checkpoint for compact. No chat dumps.
