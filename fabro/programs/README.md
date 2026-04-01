# Fabro Programs

This directory defines the checked-in Raspberry program surface for the Genesis
plans. These manifests are the durable control-plane view of the repo. They are
not a continuation of any legacy executor loop.

## Primary Entry Point

`myosu-bootstrap.yaml` is the only promoted operational entrypoint today.
It is the narrow bootstrap program referenced by `OS.md`, `README.md`, and
`AGENTS.md`.

Status: active.

Current bootstrap lanes:
- `games:traits`
- `tui:shell`
- `chain:runtime`
- `chain:pallet`

## Secondary Manifests

The remaining manifests are checked-in control-plane assets, but they are not
equally current operator entrypoints. Treat them as secondary or queued until an
explicit operator decision broadens the live program set.

- `myosu-chain-core.yaml`
  Status: secondary. Narrows the chain runtime/pallet follow-on surface after
  bootstrap.
- `myosu-services.yaml`
  Status: secondary. Covers miner and validator service surfaces beyond the
  bootstrap proof.
- `myosu-product.yaml`
  Status: secondary. Covers play/TUI and agent-experience follow-on work.
- `myosu-platform.yaml`
  Status: secondary. Covers poker-engine, multi-game, and SDK follow-on work.
- `myosu-recurring.yaml`
  Status: secondary. Covers recurring strategy, security, operations, and
  learning reports.
- `myosu.yaml`
  Status: secondary. Portfolio root that expresses the broader checked-in
  program graph, but it is not the primary live entrypoint while the repo
  remains bootstrap-first.

## Dependency Map

```text
myosu-bootstrap
├── proves games:traits
├── proves tui:shell
├── proves chain:runtime
└── proves chain:pallet

myosu-platform
└── depends on outputs/games/traits/review.md

myosu-product
└── depends on outputs/tui/shell/review.md

myosu-chain-core
└── depends on outputs/chain/{runtime,pallet}/review.md

myosu-services
└── depends on outputs/chain/{runtime,pallet}/review.md

myosu-recurring
└── runs independently
```

Read that graph as "available control-plane structure," not "everything here is
equally active today."

## Truthfulness Rules

- A manifest may depend only on reviewed artifacts that already exist.
- If a proof surface is stale, refresh the artifact first and only then mark the
  program ready in `.raspberry/`.
- These manifests describe lane ownership and dependency edges. They do not
  imply that every manifest is equally promoted in the live operator loop.
- Until explicitly widened, `myosu-bootstrap.yaml` remains the only promoted
  operational entrypoint.

No checked-in program manifest is currently labeled experimental or historical.
The present distinction is promotion level, not validity.
