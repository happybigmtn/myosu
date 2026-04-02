0a. Study `AGENTS.md` for repo-specific build, validation, and staging rules.
0b. Study `specs/*` with full repo context to understand the application specifications.
0c. Study `IMPLEMENTATION_PLAN.md`.

1. Your task is to implement functionality per the specifications using the full repository context. Follow `IMPLEMENTATION_PLAN.md` in order and take the next unchecked task from top to bottom. Do not reprioritize the queue yourself. Before making changes, search the codebase and existing planning artifacts. Do not assume a surface is missing until you verify it.

2. Implement the task completely:
   - Stay within the task contract's owned surfaces plus the minimum adjacent integration edits needed to make the code compile and work.
   - Run the relevant proof commands for the task and debug until they pass.
   - If the repo is still greenfield, perform the bootstrap work the plan requires instead of pretending later tasks are ready.
   - Do not leave placeholders, TODOs, or half-wired scaffolding.

3. Keep the planning artifacts current:
   - When you discover important implementation facts or blockers, update `IMPLEMENTATION_PLAN.md`.
   - When you finish a task, check it off in `IMPLEMENTATION_PLAN.md`.
   - Append a concise record to `COMPLETED.md` with task id, validation command, and commit sha.
   - Update `AGENTS.md` only when you learn something operational that will help future loops run or validate the repo correctly.

4. When validation passes, commit the increment:
   - Stage only the files relevant to the completed task plus `IMPLEMENTATION_PLAN.md`, `COMPLETED.md`, and `AGENTS.md`.
   - Do not sweep unrelated pre-existing churn into the commit.
   - Commit with a message like `myosu: TASK-ID short description`.
   - After committing, run `git status` to verify no implementation files were left unstaged. If any were, amend the commit.
   - Push directly to `origin/trunk` after the commit.

5. If you hit a real blocker after genuine debugging:
   - Record the blocker under the task in `IMPLEMENTATION_PLAN.md`.
   - Commit the planning update if it materially changes the execution record.
   - Move to the next ready task instead of repeating the same failed attempt.

6. Task-order rule:
   - Treat the order in `IMPLEMENTATION_PLAN.md` as authoritative.
   - Work on the first unchecked task unless its explicit dependencies are still unchecked.
   - If the current task is already satisfied, mark it truthfully and continue downward.

7. Branch rule:
   - Work only on branch `trunk`.
   - Do not create or push feature branches, lane branches, or topic branches.

99999. Important: keep `AGENTS.md` operational only.
999999. Important: prefer complete working increments over placeholders.
9999999. Important: if unrelated tests fail and they prevent a truthful green result, fix them as part of the increment.
99999999. CRITICAL: Do not assume functionality is missing — search the codebase to confirm before implementing anything new.
999999999. Every new module must be reachable from its crate root (`pub mod` in lib.rs, re-exports where the task's integration touchpoints specify). Unreachable code is an island — wire it before committing.
9999999999. When you learn something new about how to build, run, or validate the repo, update `AGENTS.md` — but keep it brief and operational only.
99999999999. As soon as there are no build or test errors, create a git tag. If no git tags exist start at 0.0.0 and increment patch by 1 (e.g. 0.0.1).
