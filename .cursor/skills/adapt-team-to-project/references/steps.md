# Adapt rules — detail

## Find the project

Root is the open project unless it is the core. Core: `.cursor/ai-dev-team/core.json` with `"role": "core"`.

If core is open, target is another workspace root or a path from the message. Neither — one question. Do not scan disks.

## Manifest scan (before deep study)

From the target root, inspect what exists (do not invent files):

- JS/TS: `package.json`, `pnpm-workspace.yaml`, `turbo.json`, `nx.json`, lockfiles
- Rust: `Cargo.toml`
- Go: `go.mod`
- Containers: `docker-compose.yml` / `compose.yaml`
- Other build manifests clearly at root

From `package.json` (or equivalent), record **real** script names for lint, typecheck, test, build. If a category has no script, write `none` for that category. Do not invent `tsc`/`eslint`/`jest` commands that are not in scripts or docs.

Then generate machine-readable gates:

```bash
node .cursor/ai-dev-team/quality/discover-gates.js <target-root> <target-root>/.cursor/ai-dev-team/quality/gates.project.json
node .cursor/ai-dev-team/quality/validate-gates.js
```

(Use the kit’s `discover-gates.js` path from the core checkout, or copy/run equivalently so `gates.project.json` lands in the **target** project.) Mirror the same verify lines into `project-*.mdc`.

## Study

Look only at this project. Roster depends on it. A small library does not get backend, database, or devops.

Parallel readers when large:

- `repository-analyst`
- `technology-analyst`
- `documentation-analyst`

Then by signal only: layers/contract → `software-architect`; UI/server → note in rules, no code; access/secrets → `security-analyst`. Do not search for or run tests during adaptation.

Do not change shared core rule files during research. Graphify only if `graphify` already exists: from target root `graphify extract . --code-only`. No `--out`. Missing CLI — read files and say so.

## Write rules

Names: `project-*.mdc`. Keep short: stack, docs location, layers, **verify commands discovered above**, what must not be invented.

Suggested always-on project rule includes a verify block, for example:

```markdown
## Verify (from manifests only)
- typecheck: `pnpm typecheck` | none
- lint: `pnpm lint` | none
- test: `pnpm test` | none
- build: `pnpm build` | none
```

`alwaysApply: true` only on one short always-needed rule. Others: description only.

Also create durable memory stubs if missing: `memory/DECISIONS.md`, `KNOWN_ISSUES.md`, `CONVENTIONS.md`, `README.md` (see `profile/layout.json`).

Do not rewrite core `team-*.mdc` or the constitution. Do not create essay profile markdown trees.


Do not touch foreign `.mdc`. Change own `project-*.mdc` only where facts changed; show the diff first. If hand-edited, append new facts — do not silent overwrite.

## Repeat

Without “from scratch” / «заново» / «полная реадаптация», do not rewrite all rules.
