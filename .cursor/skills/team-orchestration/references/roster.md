# Whom to call

**Default = core** (`employees/core-roster.json`). Optional only on signal. Prefer **minimum** roster.

## Minimum by task type

| Task | Roster |
| --- | --- |
| Point / one known component | Team Lead (no agent) |
| UI change in known area | `frontend-developer` |
| Bug, place unclear | `debugging-specialist` (or layer writer if place known) |
| New API / types boundary | **Contract-first:** `software-architect` → writer (`depends_on`) |
| New feature behavior | `product-analyst` only if gap unclear → writer |
| Server change | `backend-developer` |
| Schema / migration design | `database-architect` → `backend-developer` |
| Security-sensitive | + `security-analyst` |
| Multi-file behavior | writer → `qa-engineer` → `code-reviewer` |
| Trivial Lead fix | gates only; **no** auto reviewer |
| Release / commit | `commit-gate` |

## Core list

`repository-analyst`, `technology-analyst`, `product-analyst`, `software-architect`, `frontend-developer`, `backend-developer`, `debugging-specialist`, `qa-engineer`, `code-reviewer`, `security-analyst`, `commit-gate`, `technical-writer`

Skip architect if boundary unchanged. Skip QA until there is something to check. Do not spawn analyst+architect+QA+reviewer for one small function.
