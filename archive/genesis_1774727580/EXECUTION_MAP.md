# Execution Map

## Dependency Map

```text
bootstrap
  |
  +--> chain-core
  |      |
  |      +--> services
  |      |      |
  |      |      +--> end-to-end demo
  |      |
  |      +--> recurring
  |
  +--> platform
  |      |
  |      +--> product
  |      |
  |      +--> multi-game proof
  |
  +--> quality gates
         |
         +--> all later implementation work
```

## Frontier Definitions

### bootstrap
Owns the initial truthful orientation artifacts for the earliest active
surfaces:

- `games:traits`
- `tui:shell`
- `chain:runtime`
- `chain:pallet`

Primary outputs live under `outputs/`.

### chain-core
Owns runtime and pallet restoration work plus the path to a runnable local
chain.

Primary plans:

- `007-chain-restart.md`

### services
Owns miner and validator binaries plus chain-facing operational behavior.

Primary plans:

- `011-miner-binary.md`
- `012-validator-binary.md`

### product
Owns the user-facing gameplay path and agent experience.

Primary plans:

- `009-tui-full-implementation.md`
- `015-agent-experience.md`
- `016-e2e-demo.md`

### platform
Owns reusable engine surfaces, SDK work, and multi-game architecture.

Primary plans:

- `008-nlhe-game-engine.md`
- `013-game-engine-sdk.md`
- `014-liars-dice-proof.md`

### recurring
Owns documentation, operational readiness, and ongoing review loops.

Primary plans:

- `017-documentation.md`
- `018-operational-setup.md`

## Execution Rules

1. Bootstrap artifacts must be current before later plans rely on them.
2. Quality gates are cross-cutting and should harden early.
3. Chain-core must reach an honest compile/build state before services can be
   considered real.
4. Platform work can start before services, but product and demo work depend on
   platform progress.
5. Recurring work should reflect shipped reality rather than placeholder
   doctrine.

## Artifact Gates

| Frontier | Minimum honest gate |
|----------|---------------------|
| bootstrap | reviewed `outputs/` artifacts match current code |
| chain-core | chain packages compile honestly from repo root |
| services | miner/validator binaries do more than compile |
| product | a human or agent can traverse the gameplay path |
| platform | a game engine crate proves the trait and wire contracts |
| recurring | docs and runbooks describe current repo reality |
