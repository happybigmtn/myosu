# Foundations Frontier — First Honest Reviewed Slice

This ExecPlan is a living document. The sections `Progress`,
`Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must
be kept up to date as work proceeds.

`PLANS.md` is checked into the repository root and this document must be
maintained in accordance with it. This plan depends on `README.md`,
`SPEC.md`, `PLANS.md`, `AGENTS.md`,
`specs/031626-00-master-index.md`, and
`specs/031826-fabro-primary-executor-decision.md`.

## Purpose / Big Picture

After this slice lands, Myosu has a truthful foundations artifact pair that
records what the Fabro/Raspberry control plane can be trusted to say today and
what still belongs to upstream repair work. The operator-visible outcome is
concrete:

1. `raspberry plan/status/watch` are truthful for the current Myosu portfolio
   when they are invoked with absolute manifest paths.
2. The Myosu-side proof scripts no longer depend on a particular shell working
   directory and no longer inherit the poisoned read-only cargo target path
   from this sandbox.
3. The historical `games:multi-game` false-submit is preserved as evidence,
   not mistaken for current frontier truth.

This slice does not claim the entire Fabro detach path is fixed. It narrows the
remaining defect to one sharp boundary: relative manifest path handling in the
sibling Fabro/Raspberry toolchain, plus the still-unresolved rerun of the
historical detached `games:multi-game` submission.

## Progress

- [x] (2026-03-21 04:15Z) Re-read the governing doctrine surfaces:
  `README.md`, `SPEC.md`, `PLANS.md`, `AGENTS.md`,
  `specs/031626-00-master-index.md`, and
  `specs/031826-fabro-primary-executor-decision.md`.
- [x] (2026-03-21 04:18Z) Confirmed the historical `games:multi-game`
  false-submit still exists at
  `/home/r/.fabro/runs/20260319-01KM2BS4ASVRXVT2ND1GVVMKJ0/` with
  `status = submitted` and a detach CLI parse failure.
- [x] (2026-03-21 04:22Z) Verified that relative-manifest `raspberry status`
  is not trustworthy for proof-backed programs in this checkout:
  `myosu-bootstrap`, `myosu-chain-core`, and
  `myosu-games-traits-implementation` rendered as blocked even though their
  proof scripts succeed when run directly.
- [x] (2026-03-21 04:24Z) Hardened the Myosu proof scripts under
  `fabro/checks/` so they self-locate the repository root and force a writable
  cargo target directory in this sandbox.
- [x] (2026-03-21 04:28Z) Re-ran the proof scripts directly and confirmed all
  bootstrap, chain-core, and traits-implementation checks now exit 0 from both
  the repo root and the `fabro/programs/` directory.
- [x] (2026-03-21 04:31Z) Re-ran `raspberry status`, `watch`, and `plan` with
  absolute manifest paths and confirmed the truthful current state:
  `myosu-bootstrap` complete=4, `myosu-chain-core` complete=2,
  `myosu-games-traits-implementation` complete=1, and top-level
  `myosu` complete=7.
- [x] (2026-03-21 04:33Z) Updated the operator-facing command examples in
  `README.md`, `fabro/README.md`, and `AGENTS.md` so they use the correct
  cargo syntax and absolute-manifest variable pattern.
- [x] (2026-03-21 04:36Z) Wrote the required durable artifacts under
  `outputs/foundations/`.
- [ ] Convert the historical Raspberry-dispatched `games:multi-game`
  false-submit into a new truthful detached failure or successful live run once
  the sibling Fabro/Raspberry relative-manifest and detach-path defects are
  repaired upstream.

## Surprises & Discoveries

- Observation: the current control-plane mismatch was two defects layered on top
  of each other.
  Evidence: Myosu's proof scripts were brittle about repo-root assumptions and
  sandbox cargo target paths, but even after fixing those scripts, relative
  manifest paths still produced false blocked state while absolute manifest
  paths rendered truthful completion.

- Observation: `raspberry status` is trustworthy for artifact-only frontiers
  even with relative manifest paths, but not for command-proof frontiers in this
  checkout.
  Evidence: relative-path `status` showed `myosu-platform`, `myosu-product`,
  `myosu-recurring`, and `myosu-services` as complete, while
  `myosu-bootstrap`, `myosu-chain-core`, and
  `myosu-games-traits-implementation` were falsely blocked until the same
  command was rerun with absolute manifest paths.

- Observation: the historical `games:multi-game` false-submit no longer poisons
  the current portfolio view once the operator uses absolute manifest paths.
  Evidence: absolute-path `raspberry watch --manifest "$PWD"/fabro/programs/myosu.yaml`
  reports `complete=7 ready=0 running=0 blocked=0 failed=0`, while the old run
  directory still independently shows `status = submitted`.

- Observation: the checked-in operator docs had become part of the problem.
  Evidence: `README.md` and `fabro/README.md` still used the invalid
  `cargo --manifest-path ... run` ordering, and `AGENTS.md` still showed
  relative-manifest Raspberry commands that reproduce the current path bug.

## Decision Log

- Decision: keep this slice focused on truthful operator surfaces and durable
  evidence rather than widening the repo with a new checked-in foundations
  program.
  Rationale: the user only required `outputs/foundations/` artifacts, and the
  current Myosu checkout can already prove the control-plane mismatch without a
  new frontier manifest.
  Date/Author: 2026-03-21 / Codex

- Decision: fix Myosu-local proof script defects immediately once real execution
  exposed them.
  Rationale: these defects lived in the checked-in Myosu execution surface, were
  cheap to correct, and were directly responsible for false proof failures in
  this sandbox.
  Date/Author: 2026-03-21 / Codex

- Decision: treat absolute manifest paths as the current honest operator
  contract until the sibling Fabro/Raspberry relative-path resolver is repaired.
  Rationale: absolute manifest paths produced consistent `plan/status/watch`
  truth in this checkout, while relative manifest paths still produced false
  blocked state for proof-backed programs.
  Date/Author: 2026-03-21 / Codex

- Decision: preserve the historical `games:multi-game` false-submit as evidence
  rather than mutating or deleting it locally.
  Rationale: the detached run directory is still the cleanest proof of the
  earlier false-submit behavior and should survive until upstream repair work
  defines a cleanup or tombstone policy.
  Date/Author: 2026-03-21 / Codex

## Outcomes & Retrospective

This slice produced an honest frontier baseline.

What is now true:

- Myosu's proof scripts are self-anchored and sandbox-safe.
- The operator docs now point at command forms that work in this environment.
- Absolute-path `raspberry plan/status/watch` over the current portfolio render
  coherent truth.
- The current portfolio can be shown as fully complete in reviewed-artifact
  terms without hand-waving around blocked proof checks.

What is not yet true:

- Relative manifest path handling is still trustworthy.
- The historical detached `games:multi-game` false-submit has not yet been
  rerun through a repaired detach path in this checkout.
- `execute` has not been given a new local foundations canary in the current
  tree.

That is acceptable for a first honest reviewed slice. The frontier is now sharp
enough that the remaining work is clearly upstream Fabro/Raspberry hardening,
not Myosu-local archaeology.

## Context and Orientation

The foundations frontier in this checkout is artifact-first, not manifest-first.
Its durable outputs are:

- `outputs/foundations/foundation-plan.md`
- `outputs/foundations/review.md`

The Myosu execution surfaces touched by this slice are:

- `fabro/checks/games-traits.sh`
- `fabro/checks/tui-shell.sh`
- `fabro/checks/games-traits-implement.sh`
- `fabro/checks/chain-runtime-reset.sh`
- `fabro/checks/chain-pallet-reset.sh`
- `README.md`
- `fabro/README.md`
- `AGENTS.md`

The key historical evidence surface remains:

- `/home/r/.fabro/runs/20260319-01KM2BS4ASVRXVT2ND1GVVMKJ0/`

The key live operator surfaces are the checked-in program manifests under
`fabro/programs/`, especially:

- `fabro/programs/myosu.yaml`
- `fabro/programs/myosu-bootstrap.yaml`
- `fabro/programs/myosu-chain-core.yaml`
- `fabro/programs/myosu-games-traits-implementation.yaml`

In this repository, a "truthful" operator command means the command's printed
lane state matches the current checked-in artifacts, proof checks, and known run
metadata. A "false-submit" means Raspberry or Fabro emitted a run id or status
that looked actionable even though the underlying detached execution never
became a valid live run.

## Plan of Work

The work for this slice is intentionally narrow.

First, preserve the historical false-submit exactly as evidence. Do not edit or
delete the old run directory.

Second, keep Myosu-local proof scripts robust against the current environment by
making them resolve the repository root themselves and by forcing a writable
cargo target path. This prevents sandbox-specific file-system failures from
looking like product or proof failures.

Third, standardize the operator contract around absolute manifest paths for
Raspberry commands. This is the smallest honest workaround for the current
relative-path resolver defect in the sibling Fabro/Raspberry toolchain.

Fourth, use the durable artifacts in `outputs/foundations/` to tell future
operators exactly how to reproduce both the broken and the truthful states.

## Concrete Steps

From the repository root, the key commands for this slice are:

    MYOSU_MANIFEST="$PWD"/fabro/programs/myosu.yaml
    BOOTSTRAP_MANIFEST="$PWD"/fabro/programs/myosu-bootstrap.yaml
    CHAIN_CORE_MANIFEST="$PWD"/fabro/programs/myosu-chain-core.yaml
    TRAITS_IMPL_MANIFEST="$PWD"/fabro/programs/myosu-games-traits-implementation.yaml

    /home/r/coding/fabro/target-local/debug/raspberry status --manifest fabro/programs/myosu-bootstrap.yaml
    /home/r/coding/fabro/target-local/debug/raspberry status --manifest "$BOOTSTRAP_MANIFEST"

    /home/r/coding/fabro/target-local/debug/raspberry status --manifest "$CHAIN_CORE_MANIFEST"
    /home/r/coding/fabro/target-local/debug/raspberry status --manifest "$TRAITS_IMPL_MANIFEST"
    /home/r/coding/fabro/target-local/debug/raspberry plan --manifest "$MYOSU_MANIFEST"
    timeout 3 /home/r/coding/fabro/target-local/debug/raspberry watch --manifest "$MYOSU_MANIFEST" --interval-ms 500

    ./fabro/checks/games-traits.sh
    ./fabro/checks/tui-shell.sh
    ./fabro/checks/games-traits-implement.sh
    ./fabro/checks/chain-runtime-reset.sh
    ./fabro/checks/chain-pallet-reset.sh

To inspect the historical false-submit:

    sed -n '1,40p' /home/r/.fabro/runs/20260319-01KM2BS4ASVRXVT2ND1GVVMKJ0/status.json
    sed -n '1,40p' /home/r/.fabro/runs/20260319-01KM2BS4ASVRXVT2ND1GVVMKJ0/detach.log

## Validation and Acceptance

This first honest slice is accepted when all of the following are true:

- `outputs/foundations/foundation-plan.md` and `outputs/foundations/review.md`
  both exist and describe the same current frontier truth.
- The five Myosu proof scripts under `fabro/checks/` all succeed in this
  checkout.
- Relative-path `raspberry status --manifest fabro/programs/myosu-bootstrap.yaml`
  still demonstrates the current path-resolution defect.
- Absolute-path `raspberry plan/status/watch` over the same manifests render the
  truthful current state:
  `myosu-bootstrap` complete=4,
  `myosu-chain-core` complete=2,
  `myosu-games-traits-implementation` complete=1,
  and top-level `myosu` complete=7.
- The historical `games:multi-game` false-submit is documented with both its
  stale `submitted` status and its detach CLI parse failure.

## Idempotence and Recovery

This slice is safe to rerun.

- Re-running the proof scripts only re-tests the current checkout.
- Re-running `raspberry plan/status/watch` only refreshes `.raspberry/*.json`
  state files in the current worktree.
- The doc changes are additive and can be re-applied cleanly.

Do not edit or remove the historical run directory under `/home/r/.fabro/runs/`
as part of normal reruns. It is part of the evidence set.

## Artifacts and Notes

Representative failure evidence:

    Program: myosu-bootstrap
    Counts: complete=0 ready=0 running=0 blocked=4 failed=0
    games:traits [blocked|platform] ... proof_state=failed

Representative truthful state after switching to absolute manifest paths:

    Program: myosu-bootstrap
    Counts: complete=4 ready=0 running=0 blocked=0 failed=0
    games:traits [complete|platform] ... proof_state=met

Representative top-level truthful state:

    Program: myosu
    Counts: complete=7 ready=0 running=0 blocked=0 failed=0

Historical false-submit evidence:

    {
      "status": "submitted",
      "updated_at": "2026-03-19T06:16:00.090006038Z"
    }

    error: unexpected argument '/home/r/coding/myosu/fabro/programs/../run-configs/platform/multi-game.toml' found

## Interfaces and Dependencies

The Myosu-side surfaces that must remain stable after this slice are:

- `fabro/checks/*.sh` as the proof interface for Raspberry `command_succeeds`
  checks
- `fabro/programs/*.yaml` as the control-plane manifest interface
- `outputs/` as the durable reviewed-artifact contract

The sibling-tool dependency that remains open is the Fabro/Raspberry manifest
path resolver. This slice deliberately does not patch `/home/r/coding/fabro/`;
it captures the repro clearly enough that an upstream fix can start from these
artifacts.
