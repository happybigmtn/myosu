# Execution Playbooks

This directory describes the direct-execution playbooks for Myosu. These are
repo-native guides for current truthful execution surfaces, not generic future
operator stories.

## Families

### Bootstrap
Use when the goal is to verify and refresh curated artifacts under `outputs/`
through the current Raspberry bootstrap program.

See [bootstrap.md](bootstrap.md).

### Local Advisor
Use when the goal is to exercise the currently proven human/agent gameplay
surface in `myosu-play`.

See [local-advisor.md](local-advisor.md).

### Stage-0 Local Loop
Use when the goal is to prove the smallest honest local chain loop end to end.
This is the preferred first-class service proof.

See [stage0-local-loop.md](stage0-local-loop.md).

### Operator Network
Use when the goal is to verify the current named chain packaging and the shared
operator key surface without overclaiming a finished wallet or public network
story.

See [operator-network.md](operator-network.md).

### Implementation
Use when building or revising code in an active crate or feature slice.

See [implementation.md](implementation.md).

### Services
Use when diagnosing or revising runnable binaries and daemons beyond the
preferred node-owned proof surfaces.

See [services.md](services.md).

### Maintenance
Use for repeatable cleanup, docs sync, audit, and health checks.

See [maintenance.md](maintenance.md).

### Planning
Use when converting broad goals into a concrete sequence of artifact-backed
steps.

See [planning.md](planning.md).

### Review
Use for final truth checks before claiming a slice is complete.

See [review.md](review.md).
