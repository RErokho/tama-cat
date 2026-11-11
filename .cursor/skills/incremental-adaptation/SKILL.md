---
name: incremental-adaptation
description: Updates existing project-*.mdc only where project facts changed. Use on repeat adaptation without an explicit full rebuild request.
disable-model-invocation: true
---

# Incremental adaptation

When the project is already adapted and the user asks to adapt again (not a full rebuild):

1. Diff each `project-*.mdc` fact against the repo (manifests/scripts).
2. Change only stale lines. Do not rewrite the whole file.
3. If the user edited a rule by hand, append — do not clobber.
4. Do not touch core kit rules.

List what changed. No install script.
