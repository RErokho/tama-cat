# Quick start

Open this folder in Cursor. Model: Grok 4.7 low / medium / high.

```text
adapt the rules
```

Writes `project-*.mdc` and should produce `gates.project.json`:

```bash
node .cursor/ai-dev-team/quality/discover-gates.js <project-root> <project-root>/.cursor/ai-dev-team/quality/gates.project.json
```

Before commit clearance:

```bash
node .cursor/ai-dev-team/quality/regression-awareness.js .
node .cursor/ai-dev-team/quality/run-gates.js --task TASK-1
```

Optional parallel isolation: `execution.mode=worktree` + `dispatch.js` / `worktree.js`. Default: shared packets.

MCP: Context7 only. Map: `AGENTS.md`.
