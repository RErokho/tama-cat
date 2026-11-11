# AI Dev Team (v0.8)

| Plane | Holds |
| --- | --- |
| Control | tasks, compact contract/handoff, topical memory |
| Execution | agents (lazy card load), packets, optional worktrees |
| Trust | gates + check `summary`, hooks, commit-gate |
| Observability | local events + qos |

**Thrift:** registry → one card → compact handoff → gate summaries (full logs on disk).

```bash
node .cursor/ai-dev-team/evals/run.js
node .cursor/ai-dev-team/evals/adapt-e2e.js
node .cursor/ai-dev-team/evals/harness-e2e.js
node .cursor/ai-dev-team/evals/smoke-project.js <project-root>
```
