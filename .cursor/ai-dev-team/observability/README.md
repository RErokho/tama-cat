# Observability

Local JSONL log. No cloud. Skill `/kit-observability` is manual-only.

```bash
node .cursor/ai-dev-team/observability/events.js tail --n 20
node .cursor/ai-dev-team/observability/report.js .
```

Hooks: `run-gates`, `lifecycle`, `dispatch`, `sessionStart` may append. No secrets in events.
