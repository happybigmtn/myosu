Read the workflow goal carefully. It names:

- the lane being bootstrapped
- the code or docs you must inspect
- the exact curated artifact paths you must produce

Your task in this stage:

1. Inspect the referenced repo surfaces directly.
2. Decide what the lane is responsible for in Fabro/Raspberry terms.
3. Write or replace the lane's `spec.md` artifact at the exact path named in
   the goal.

The `spec.md` must be self-contained and should cover:

- purpose and user-visible outcome for the lane
- current trusted inputs
- current broken or missing surfaces
- exact code boundaries and deliverables
- proof/check shape for the lane
- next implementation slices

Do real repo research before writing. Do not leave placeholders.
