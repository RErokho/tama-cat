# Report format presets

Put the preset **name** or the template text into `contract.report_format`. Prefer the name; Team Lead expands it in the handoff.

## Presets

### `writer-brief` (default for writing assignees)

```text
Changed: <paths>
Done: <1-3 bullets>
Risks/Unfinished: <none | bullets>
```

Rules: no preamble; no restating the prompt; do not paste full file bodies.

### `reviewer-brief` (code-reviewer, qa, a11y, security-analyst, performance)

```text
Verdict: accept | rework
Blockers: <none | location + issue>
Nits: <optional>
```

### `analyst-brief` (architects, analysts, designers — report-only)

```text
Finding: <1-5 bullets; mark facts vs guesses>
Evidence: <paths or commands>
Recommendation|Question: <one>
```

### `gate-brief` (commit-gate)

```text
Clearance: yes | no
Report: <path + status>
Reasons: <bullets>
Missing evidence: <none | paths>
Commits: <optional draft slices>
```

### `lead-brief` (Team Lead → user)

2–5 sentences. Details only if asked. No restating the task. No fake token counts.

## Optional `contract.max_reply`

| Value | Meaning |
| --- | --- |
| `short` | Prefer the brief preset; max ~8 lines in the agent return |
| `normal` | Brief preset allowed; slightly more bullets if acceptance needs it |

Do not put a numeric token budget here — the system has no ledger.
