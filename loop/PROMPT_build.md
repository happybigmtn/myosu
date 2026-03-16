${COMPLETED_CONTEXT}
## Spec contract for this task:
${SPEC_EXCERPT}

## Focus task block for this iteration:
${TASK_BLOCK}

## Instructions

1. Before editing, search the codebase. Do not assume missing functionality.

2. Implement only the selected task scope:
   - Satisfy every AC linked in that task.
   - Map task requirements to:
     - concrete files/modules (`Where`)
     - expected state writes
     - acceptance artifacts (test output)
   - If required tests are missing and cannot be safely added, stop and
     return BLOCKED.

3. Run all required tests/checks listed for that task before updating
   the plan.

4. Update `ralph/IMPLEMENT.md` with outcomes/discoveries, then stop.

5. End with exactly one machine-readable final line:
   - `RESULT: <item_id> <proof_summary> <commit_or_none> <next_item>`
   - `BLOCKED: <item_id> <blocker_tag> <prerequisite> <next_item>`
   - The final line must be the last non-empty line in your response.
   - Use the real item ID from the task block.
   - Tokens must use underscores instead of spaces.
   - The loop runner handles commits — use `none` for the commit field.

6. A task is incomplete until all listed required tests pass.
7. Keep changes single-purpose. No placeholders or deferred core logic.
8. Do not create manual git commits — the loop runner handles that.
