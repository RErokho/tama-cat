# When to load a skill

Read **one** row. Role work → Task an agent (read **one** card after registry). Graphify stays available on signal.

| Situation | Load |
| --- | --- |
| Record task | `tasks` |
| Contract | `task-contract` |
| Compact handoff / thrift | `handoff-management`, `context-optimization` |
| Split request vs ownership | `task-decomposition` then `work-packet` |
| Whom to call | `roster.md` + `employees/registry.json` |
| Memory (topical) | `project-memory` |
| Close / evidence | `quality-gates` → `regression-awareness` → `run-gates` |
| Adapt | `connect` → `adapt-team-to-project` |
| Library docs | `docs-lookup` (on demand) |
| Code graph | `graphify` (narrow query only) |
| Noise | `noise-isolate` |
| Compact | `compact-checkpoint` |
| Rollback | `packet-rollback` |
| Worktrees | `worktree-isolation` |
| Full rebuild | `full-readaptation` |
| Status / usage | `team-status` / `/kit-observability` |
| Reviews | `/review-bugbot`, `/review-security`, `/autopilot` |
