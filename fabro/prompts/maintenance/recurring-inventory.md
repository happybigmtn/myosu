Read these inputs before you write anything:

- the workflow goal for this recurring lane
- any existing `spec.md` / `review.md` artifacts for the lane
- the relevant doctrine, outputs, or ops surfaces named in the goal

Your task is to produce or replace the lane's `spec.md`.

This is a recurring-oversight bootstrap run. The `spec.md` must:

- define the recurring lane's operator-facing purpose
- define what repository surfaces it reads as doctrine or source truth
- define what durable outputs it should eventually write or refresh
- define what "healthy recurring behavior" means for this lane
- define the smallest honest next implementation slices

Do not implement the recurring engine. Produce a durable contract that a later
maintenance or conformance workflow can consume directly.
