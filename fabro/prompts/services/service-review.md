Read these inputs before you write anything:

- the lane's `spec.md`
- the relevant source tree(s) named in the workflow goal
- any current chain artifacts under `outputs/chain/` if they exist

Your task is to produce or replace the lane's `review.md`.

The `review.md` must answer:

- what service-related code or docs already exist and are worth keeping
- what is missing or too speculative to trust
- what upstream, chain, or abstraction dependencies block honest bringup
- what the very next slice should do
- which proof and health checks should be added first

Keep the review tied to actual files and proof surfaces in this repository.
