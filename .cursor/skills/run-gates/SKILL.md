---
name: run-gates
description: Runs project gates and writes verification-report.json with per-check summary. Use before verified or commit-gate.
---

# Run gates

```bash
node .cursor/ai-dev-team/quality/discover-gates.js .
node .cursor/ai-dev-team/quality/regression-awareness.js .
node .cursor/ai-dev-team/quality/run-gates.js --task TASK-1
```

`--only` from regression report · `--skip-reason` only with user skip permission.

## Context thrift

- Full stdout/stderr → disk under `quality/logs/runs/`.
- Into chat/handoff: each check’s `summary` (headline + first_errors) + exit_code — **not** the full log.
- Fetch log fragments only when debugging a failure.

Report: `verification-report.json`. Lifecycle `verified` needs passed/allowed-skipped report.
