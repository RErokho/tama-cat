---
name: full-readaptation
description: Full rebuild of project-*.mdc only when the user explicitly asks for a full adapt rebuild. Not for ordinary adapt.
disable-model-invocation: true
---

# Full readaptation

Only when the user asks for a full rebuild. Otherwise use `adapt-team-to-project` / `incremental-adaptation`.

1. Re-scan manifests and structure (skill `adapt-team-to-project` steps).
2. Rewrite `project-*.mdc` from fresh facts. Do not keep stale verify commands.
3. Refresh memory templates if empty/stub.
4. Do not rewrite core kit rules or the constitution.
