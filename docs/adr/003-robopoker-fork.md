# ADR 003: Narrow Robopoker Fork Anchored To `v1.0.0`

- Status: Accepted
- Date: 2026-04-02
- Deciders: Myosu maintainers
- Consulted: `ops/decision_log.md`, `AGENTS.md`, `docs/robopoker-fork-changelog.md`, `specs/031626-02b-poker-engine.md`
- Informed: poker, miner, validator, and security-maintenance contributors
- Related: `crates/myosu-games-poker/Cargo.toml`, `crates/myosu-games-kuhn/Cargo.toml`, `docs/robopoker-fork-changelog.md`, `ops/cve-tracking-process.md`, `crates/myosu-games-poker/src/robopoker.rs`

## Context

This is a retroactive record of the 2026-03-16/17 decision to build Myosu on a
robopoker baseline while allowing a narrow downstream fork.

Myosu needs a production-grade MCCFR engine with real poker abstractions and a
credible exploitability path. Robopoker `v1.0.0` provides that baseline. At the
same time, Myosu needed downstream changes for wire-safe serialization and local
artifact handling, and later work may still require additional forked surfaces.

The repo already reflects this choice: game crates pin the
`happybigmtn/robopoker` fork at a specific revision, while local wrappers keep
the rest of the repo insulated from direct upstream type churn.

## Decision

Myosu consumes a narrow robopoker fork pinned close to the upstream
`v1.0.0` release and records every downstream divergence explicitly.

That means:

- pinning the fork by exact revision in workspace crates
- keeping the fork changelog in-repo so INV-006 stays inspectable
- wrapping upstream types behind local `myosu-games-poker` surfaces instead of
  letting fork details leak everywhere
- preferring narrow fork growth over vendoring or freeform downstream mutation

## Alternatives Considered

### Option A: Narrow pinned fork near `v1.0.0`

This won because it preserves a proven MCCFR baseline while still allowing the
small extensions Myosu needs.

### Option B: Depend on upstream robopoker without any fork

This was rejected because Myosu already needed serde-enabled NLHE surfaces and
other downstream control points not available in the untouched baseline.

### Option C: Vendor or rewrite the solver stack locally

This was rejected because it would discard upstream provenance and greatly
increase maintenance burden for highly specialized solver code.

## Consequences

### Positive

- Myosu gets a credible poker solver baseline instead of inventing one.
- Fork drift is explicit and auditable through a repo-local changelog and CVE
  tracking process.
- Local wrappers let the rest of the repo depend on stable Myosu-facing types.

### Negative

- Security and feature drift must be monitored continuously.
- Some future work still depends on fork changes outside the currently pinned
  serde-only delta.

### Follow-up

- Update `docs/robopoker-fork-changelog.md` in the same slice as any revision
  bump.
- Keep downstream changes minimal and tied to concrete Myosu proof needs.

## Reversibility

Moderate.

Because Myosu wraps most direct upstream types, it could move back toward
upstream or to a different revision with bounded refactoring. The harder part
is preserving checkpoint, wire, and validation behavior across such a move. The
decision should be reopened only when a new upstream revision or a replacement
engine can satisfy the same proof surfaces with lower long-term risk.

## Validation / Evidence

- `crates/myosu-games-poker/Cargo.toml` and `crates/myosu-games-kuhn/Cargo.toml`
  pin the `happybigmtn/robopoker` fork revision.
- `docs/robopoker-fork-changelog.md` records the current divergence from
  `v1.0.0`.
- `crates/myosu-games-poker/src/robopoker.rs` and related wrapper modules keep
  the rest of the repo on Myosu-owned seams instead of raw upstream imports.
