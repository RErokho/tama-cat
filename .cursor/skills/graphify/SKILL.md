---
name: graphify
description: Builds and queries a local code graph via the Graphify CLI. Call when module links, a path between nodes, or impact analysis is needed. Do not use the cloud.
---

# Graphify

Local code analysis only. Package `graphifyy`, command `graphify`. Do not connect cloud, MCP, or model keys.

Do not run `graphify install --project` in the core.

## If the command is missing

If `graphify` is missing, say so and work from files. Do not install it via script. Do not connect the cloud.

## Build

From the root of the needed project:

```bash
graphify extract . --code-only
```

The graph appears at `graphify-out/graph.json` in that project root. Do not set `--out`: it moves `graphify-out` into another folder. The command itself reports that docs are skipped and no model is called. Do not run `cluster-only` or `label`: community naming may call a model.

Again, without a model, from the project root:

```bash
graphify update .
```

`--out` is not in the CLI docs for `update`, so do not add it on update. Read the `graph.json` that actually appeared.

Do not add `graphify-out/` to Git or move it into `.cursor/`. Do not build a graph for one known file. Record a build note in `.cursor/ai-dev-team/graphify/last-build.json`: CLI version, date, `codeOnly: true`, path `graphify-out/graph.json`. Do not enable semantic document analysis.

## Queries

Do not put the whole `graph.json` into context. Point `--graph` at the actual `graph.json`.

```bash
graphify query "name from code" --budget 1000 --graph "<graph.json>"
graphify explain "NodeName" --graph "<graph.json>"
graphify path "NodeA" "NodeB" --graph "<graph.json>"
graphify affected "NodeName" --depth 2 --graph "<graph.json>"
```

Exit codes: 0 — answer present, 3 — several nodes, 4 — nothing found. On 3, take a name from the list. On 4, say the node is missing.

Put an excerpt into an agent prompt: path, neighbors, EXTRACTED or INFERRED. Do not treat INFERRED as a source-code fact.
