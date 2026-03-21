# Foundations Frontier Review

**Frontier:** `foundations`
**Output root:** `outputs/foundations/`
**Date:** 2026-03-21

## Keep / Reopen / Reset Judgment

**Judgment: KEEP this reviewed slice, REOPEN relative-manifest and detach-path hardening upstream.**

This slice is honest and worth keeping. It establishes a truthful current
operator contract:

- use absolute manifest paths for Raspberry commands
- trust the current portfolio state rendered through that path shape
- preserve the historical `games:multi-game` false-submit as evidence

What still needs reopening is upstream Fabro/Raspberry behavior, not the Myosu
artifact layer:

- relative manifest path handling still produces false blocked state for
  proof-backed programs
- the historical detached `games:multi-game` submission has not yet been rerun
  through a repaired detach path in this checkout

## What Is Trustworthy Today

### 1. Absolute-path `plan/status/watch` over the current portfolio

Using absolute manifest paths, the current checked-in Myosu frontiers render
coherent truth:

- `myosu-bootstrap`: complete=4
- `myosu-chain-core`: complete=2
- `myosu-games-traits-implementation`: complete=1
- `myosu-platform`: complete=3
- `myosu-product`: complete=2
- `myosu-recurring`: complete=4
- `myosu-services`: complete=2
- top-level `myosu`: complete=7

`raspberry watch --manifest "$PWD"/fabro/programs/myosu.yaml` also reports the
same state, so the truthful path is not limited to one command variant.

### 2. The Myosu proof scripts after hardening

The following proof interfaces now exit 0 in this sandbox:

- `fabro/checks/games-traits.sh`
- `fabro/checks/tui-shell.sh`
- `fabro/checks/games-traits-implement.sh`
- `fabro/checks/chain-runtime-reset.sh`
- `fabro/checks/chain-pallet-reset.sh`

They no longer depend on a repo-root shell by accident, and the cargo-based
ones no longer inherit the poisoned read-only cargo target path.

### 3. The historical false-submit as evidence only

The old detached `games:multi-game` run still shows:

- `status.json`: `status = submitted`
- `detach.log`: `unexpected argument .../run-configs/platform/multi-game.toml`

That evidence is still useful, but it is no longer the best description of the
current portfolio state when the operator uses the truthful absolute-path
contract.

## What Is Not Yet Trustworthy

### 1. Relative-manifest Raspberry commands

Relative manifest paths still misrender proof-backed frontiers in this checkout.
The clearest repro is:

    /home/r/coding/fabro/target-local/debug/raspberry status --manifest fabro/programs/myosu-bootstrap.yaml

That command reports:

- `complete=0 blocked=4`
- `proof_state=failed` for all four lanes

The same binary against the same manifest via an absolute path reports:

- `complete=4 blocked=0`
- `proof_state=met` for all four lanes

This is a real control-plane defect, not a documentation typo.

### 2. Detached execute reruns for the old multi-game incident

This slice did not rerun `execute` against a new local foundations canary or a
repaired `games:multi-game` lane. The historical false-submit is documented, but
the detached execute path still needs an upstream repair slice before we can
claim the original incident has been fully converted into a new truthful live
run or truthful live failure inside this checkout.

## Concrete Findings

### Finding 1: The current false truth is path-shaped, not artifact-shaped

The current reviewed artifacts under `outputs/` are internally coherent. The
same repo renders blocked or complete depending on whether Raspberry receives a
relative or absolute manifest path. That pins the remaining defect to manifest
path resolution in the sibling Fabro/Raspberry toolchain.

### Finding 2: Myosu had local proof-script fragility that worsened diagnosis

Before this slice, the proof scripts mixed cwd-sensitive file tests with cargo
commands that inherited a read-only target directory. Those defects are now
fixed locally, which means future failures are more likely to reflect the
control plane rather than the shell wrapper.

### Finding 3: The historical `games:multi-game` false-submit is still real

The stale run directory under `/home/r/.fabro/runs/20260319-01KM2BS4ASVRXVT2ND1GVVMKJ0/`
remains the durable proof that Raspberry once emitted a detached submission
whose worker never became a truthful run. That evidence should stay intact.

### Finding 4: The operator docs were previously teaching the broken path

`README.md`, `fabro/README.md`, and `AGENTS.md` all pointed operators at
relative-manifest Raspberry commands, and two of them also used stale cargo
syntax. Those docs now use absolute-manifest variables and the correct
`cargo run --manifest-path ...` ordering.

## Recommendation

Keep the current slice and use it as the baseline for future frontier work.

The next repair slice should happen in the sibling Fabro/Raspberry repo and
should do two things:

1. Fix relative manifest path resolution so `target_repo: ../..` does not
   collapse the command working directory when the manifest argument is relative.
2. Re-run a detached canary lane, then re-run the historical
   `games:multi-game` scenario or an equivalent foundations canary until
   `execute/status/watch` all agree on the live result.

Until that upstream repair lands, Myosu operators should treat absolute manifest
paths as mandatory for truthful Raspberry interaction in this environment.
