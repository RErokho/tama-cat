---
name: kit-observability
description: Manual local usage log/report for kit events. Call only when the user asks for usage summary.
disable-model-invocation: true
---

# Kit observability

Local only. No cloud.

```bash
node .cursor/ai-dev-team/observability/events.js tail --n 30
node .cursor/ai-dev-team/observability/report.js .
```

Report `qos` includes gate pass rate, avg gate duration, handoffs, ownership denies, task status counts. Still no billed token meter from Cursor.

`run-gates` / `lifecycle` / `dispatch` / ownership hook may append events. No secrets in events.
