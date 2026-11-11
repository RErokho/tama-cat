---
name: docs-lookup
description: Grounds library/API usage in current docs before coding. Prefer Context7 MCP or project docs over training memory.
---

# Docs lookup

## Purpose

Stop inventing APIs that changed after training data.

## Trigger

New library call, unfamiliar SDK, version-sensitive API, or failing type/docs mismatch.

## Steps

1. Prefer project docs named in `project-*.mdc` / `AGENTS.md`.
2. Else use Context7 MCP (`resolve-library-id` → `query-docs`) for the exact library/version.
3. Else official CLI/docs URL via WebFetch — not a dump of Stack Overflow.
4. Quote the symbol/signature you will use. If docs conflict with code, stop and ask.
5. Only then call a developer agent with the grounded signature in the handoff facts.

## Limit

Do not attach whole doc pages. Keep three facts max in the handoff.
