Read these inputs before you write anything:

- the workflow goal for this service lane
- any existing `spec.md` / `review.md` artifacts for the lane
- the relevant source tree(s) named in the goal
- the current chain artifacts under `outputs/chain/` if they exist

Your task is to produce or replace the lane's `spec.md`.

This is a service-lane bootstrap run. The `spec.md` must:

- define the service lane boundary and its operator-visible purpose
- state what readiness means before the lane can honestly run
- state what health means while the lane is running
- state what proof posture the lane should eventually expose
- name the exact files, binaries, endpoints, or chain surfaces the lane is
  expected to own
- define the smallest honest next implementation slices in order

Do not implement the service. Produce a durable lane contract that a later
implementation or bringup workflow can consume directly.
