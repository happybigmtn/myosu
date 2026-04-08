# 005 — Stale Document Cleanup

## Objective

Remove or update stale documentation surfaces that create false impressions about the system's current state. Focus on high-visibility files that new contributors read first.

## Context

Several root-level documents contain stale or misleading content:

1. **AGENTS.md** references `fabro/workflows/`, `fabro/run-configs/`, `fabro/programs/`, `.raspberry/` — none of which exist on disk
2. **OS.md** references the same fabro/raspberry infrastructure
3. **THEORY.MD** (97K bytes) is historical CFR theory with no connection to running code
4. **IMPLEMENTATION_PLAN.md** mixes completed items, blocked items, and stale queue metadata
5. **README.md** includes `fabro run` commands that don't work

## Acceptance Criteria

- AGENTS.md fabro/raspberry references are either removed or marked as "planned, not yet implemented"
- OS.md fabro/raspberry references receive the same treatment
- README.md `fabro run` commands are removed or replaced with working alternatives
- THEORY.MD is either moved to `research/` or `archive/` (not deleted — it has educational value)
- IMPLEMENTATION_PLAN.md completed items are reconciled against ARCHIVED.md; blocked items are reconciled against WORKLIST.md
- No root-level .md file references a path that doesn't exist without explicitly noting it is planned
- `bash .github/scripts/check_doctrine_integrity.sh` still passes (if it validates document presence)

## Verification

```bash
# Confirm no broken fabro references in key docs
for f in AGENTS.md OS.md README.md; do
  if grep -q "fabro/" "$f" 2>/dev/null; then
    echo "WARN: $f still references fabro/ — verify each is marked as planned"
  fi
done

# Confirm THEORY.MD moved
test ! -f THEORY.MD || echo "THEORY.MD still at root"

# Confirm doctrine check passes
bash .github/scripts/check_doctrine_integrity.sh
```

## Dependencies

- None. This is independent of all code changes.
