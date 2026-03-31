# Operator Hardening and Network Packaging

Status: Active next-step plan as of 2026-03-30.

This ExecPlan is a living document. Maintained per `genesis/PLANS.md`.

## Purpose / Big Picture

Stage-0 proof is now honest and green, but the repo is still strongest as a
founder-operated local loop. The next useful lane is not more doctrine cleanup
or more game surface. It is making the current system operable by someone who
does not share the author's local habits.

This plan hardens the next operator-facing layer above the current proof stack:
key handling, named network packaging, and a publishable runbook for bringing
up the current node/miner/validator surface outside the narrow owned smoke.

After this plan, Myosu should still look like an early network, not a product
launch. But it should stop looking like a repo that only the current operator
can run safely.

## Progress

- [x] (2026-03-30) Promoted this plan after the release-gate doctrine sync
  closed the remaining stage-0 governance blockers under `011`.
- [x] (2026-03-30) Re-read the current weakest surfaces in
  `ops/security-audit-stage0.md`, `genesis/ASSESSMENT.md`, and `OS.md` before
  choosing the next lane.
- [x] (2026-03-30) Started the first implementation slice by adding a minimal
  shared `myosu-keys` crate for mnemonic/address/path helpers and by restoring
  honest named `devnet` / `testnet` chain-spec surfaces to the current local
  genesis instead of unconditional placeholder errors.
- [x] (2026-03-30) Verified the first slice with `cargo check -p myosu-keys`,
  `cargo test -p myosu-keys --quiet`, `SKIP_WASM_BUILD=1 cargo check -p
  myosu-chain --features fast-runtime`, and fresh `cargo run -p myosu-chain
  --features fast-runtime -- build-spec --chain devnet|test_finney` proofs.
- [x] (2026-03-30) Published the first operator-facing runbook in
  `docs/execution-playbooks/operator-network.md` and linked it from the repo
  entrypoints so the current named-network and key-helper story is no longer
  tribal knowledge.
- [x] (2026-03-30) Landed the next `021` key-management slice in
  `myosu-keys`: the shared crate now writes `~/.myosu/config.toml` plus an
  encrypted `~/.myosu/keys/<ss58>.json` seed file, and can load the active
  operator pair back from disk without ever persisting the mnemonic phrase.
- [x] (2026-03-30) Wired the first operator-facing consumer path: `myosu-miner`
  and `myosu-validator` now accept either a raw `--key` URI or a
  `--key-config-dir` plus password env var, and resolve the active configured
  account into the standard secret-URI form already accepted by
  `myosu-chain-client`.
- [x] (2026-03-30) Landed the next narrow creation surface: `myosu-keys` now
  ships a minimal `create` / `show-active` CLI so an operator can generate an
  encrypted keystore, record the mnemonic once, and inspect the active account
  without needing a separate test harness.
- [x] (2026-03-30) Landed the next multi-account control surface:
  `myosu-keys` now exposes `list` and `switch-active`, so operators can see
  the stored accounts in a config dir and choose which one miner/validator
  should consume without editing `config.toml` by hand.
- [x] (2026-03-30) Landed the next portability surface: `myosu-keys` now
  exposes encrypted keyfile `import-keyfile` and `export-active-keyfile`
  commands so operators can move a Myosu-managed account between config dirs
  without exposing raw seed material.
- [x] (2026-03-30) Landed the next operator-import surface: `myosu-keys` now
  exposes `import-mnemonic` and `import-raw-seed`, both env-var driven, so an
  operator can recover an account without putting mnemonic or seed material on
  the shell command line.
- [x] (2026-03-30) Landed the next keystore-hygiene surface: `myosu-keys` now
  exposes `change-password`, so operators can re-encrypt the active account
  with a new password env var without regenerating or re-importing the key.
- [x] (2026-03-30) Landed the next bootstrap seam: `myosu-keys` now exposes
  `print-bootstrap`, so the repo can print the exact `myosu-miner` and
  `myosu-validator` commands for the active configured account instead of
  leaving that step to runbook inference.
- [x] (2026-03-30) Landed the next operator-proof seam:
  `.github/scripts/check_operator_network_bootstrap.sh` now turns the printed
  bootstrap path into a repo-owned smoke that reaches miner/validator help
  surfaces and then re-proves named `build-spec` output.
- [x] (2026-03-30) Landed the next operator-helper seam:
  `.github/scripts/prepare_operator_network_bundle.sh` now writes a reusable
  bundle with `start-miner.sh`, `start-validator.sh`,
  `build-devnet-spec.sh`, `build-test-finney-spec.sh`, `verify-bundle.sh`, and
  a local README from the active config and bootstrap output, and now
  materializes `devnet-spec.json`, `test-finney-spec.json`, and
  `bundle-manifest.toml` into the bundle instead of leaving operators to keep
  shell snippets by hand.
- [x] (2026-03-30) Promoted the operator-bundle proof into hosted CI shape by
  adding an `Operator Network` workflow job that runs
  `.github/scripts/check_operator_network_bootstrap.sh` under
  `SKIP_WASM_BUILD=1` with runner-side `protoc` plus the Rust
  `wasm32-unknown-unknown` target installed.
- [x] (2026-03-30) Hardened the hosted proof path so cold runners no longer
  depend on a preexisting cached runtime wasm: `active-crates` now installs
  `protoc`, and `myosu-chain-runtime` now treats `SKIP_WASM_BUILD=1` as a
  cache preference rather than a hard skip when the named-network proof needs
  a fresh runtime wasm.
- [x] (2026-03-30) Synced the remaining cold-runner dependency that hosted CI
  exposed: the active-crates, chain, and operator jobs now install Rust
  `rust-src` in addition to `wasm32-unknown-unknown`, because Substrate's wasm
  builder needs standard-library sources whenever the runtime fallback path
  compiles a fresh wasm artifact.

## Surprises & Discoveries

- Observation: The highest-value remaining risk is not chain correctness.
  Evidence: the current release gate is green, hosted CI is green, and the
  local owned two-subnet proof is already closed.

- Observation: The next operator problem splits cleanly into three pieces:
  key handling, named network packaging, and operator docs.
  Evidence: `ops/security-audit-stage0.md` calls out key management as the
  main accepted stage-1 risk, while `genesis/ASSESSMENT.md` still says
  `devnet.rs` / `testnet.rs` are thinner than the local proof path.

- Observation: the smallest honest keystore slice is library-first, not
  wallet-first.
  Evidence: the repo can already prove encrypted seed-at-rest, active-account
  config persistence, and `0600` file permissions through `myosu-keys` tests
  without pretending it ships a CLI, export flow, or account-switch UX yet.

- Observation: the first operator-facing keystore integration did not require a
  new signing stack in the chain client.
  Evidence: `myosu-chain-client` already accepts standard Substrate secret URIs,
  so the narrow truthful move was to teach `myosu-keys` to recover the active
  configured account into that URI shape and let miner/validator reuse the
  existing client path.

- Observation: the smallest honest creation path is CLI-first, not wallet-first.
  Evidence: a two-command binary (`create`, `show-active`) gives operators a
  real account bootstrap path while avoiding premature import/export/switch UX.

- Observation: the smallest honest multi-account step is config-first, not
  metadata-first.
  Evidence: `list` plus `switch-active` gives a real operator control loop
  using the existing encrypted key files and active-account config, without
  inventing labels, balances, or a richer wallet screen.

- Observation: the smallest honest portability step is keyfile-first, not
  seed-first.
  Evidence: importing and exporting the existing encrypted Myosu keyfile gives
  operators a real backup/transfer path without claiming mnemonic/raw-seed UX
  or broader Substrate-wallet compatibility.

- Observation: the smallest honest secret-input step is env-first, not argv-first.
  Evidence: `import-mnemonic` and `import-raw-seed` take environment-variable
  names, which keeps shell history and process listings cleaner than passing the
  secret material directly as CLI arguments.

- Observation: the smallest honest password-hygiene step is active-first, not
  fleet-first.
  Evidence: rotating the active account password solves the immediate operator
  risk without introducing bulk-keystore mutation or account-management UI.

- Observation: the next honest bootstrap step is command-printing, not another
  keystore primitive.
  Evidence: once the keystore can already create, inspect, switch, recover,
  export, and rotate the active account, the remaining operator friction is
  "what do I run next?" rather than "what other secret format can I store?"

- Observation: command-printing alone still leaves proof posture too manual.
  Evidence: the next honest operator improvement is a checked-in smoke script
  that proves the printed commands still line up with the actual binary
  surfaces and named network packaging.

- Observation: a printed command is still not the same thing as an owned
  operator handoff artifact.
  Evidence: the next honest improvement is a checked-in helper that writes the
  wrapper scripts and README an operator can actually keep and rerun.

- Observation: the bundle should carry named-network proof too, not only
  service wrappers.
  Evidence: the same operator handoff that starts miner/validator also needs a
  repeatable `devnet` / `test_finney` spec-materialization path owned by the
  generated bundle.

- Observation: the bundle should verify itself, not require the repo smoke to
  know every generated filename.
  Evidence: a bundle-local `verify-bundle.sh` keeps the handoff artifact
  cohesive and reduces drift between bundle contents and repo-level checks.

- Observation: the bundle should ship current spec artifacts, not only the
  scripts that know how to rebuild them.
  Evidence: a second operator gains a more usable handoff when the bundle
  already contains `devnet-spec.json` and `test-finney-spec.json` at creation
  time, with refresh scripts still available beside them.

- Observation: the bundle should be machine-readable, not just human-readable.
  Evidence: a checked-in `bundle-manifest.toml` lets another operator, script,
  or agent inspect the handoff artifact without scraping the README.

- Observation: once the operator bundle has a real smoke script, leaving it
  local-only weakens the closure bar for this plan.
  Evidence: the next honest move is a hosted CI lane that runs the same
  bundle/bootstrap proof on GitHub Actions.

- Observation: cold-machine named-network proof still needs the Rust wasm
  target installed.
  Evidence: the first clean-clone publish proof failed until the environment
  installed `wasm32-unknown-unknown`, even after the operator bootstrap
  commands themselves were made target-dir agnostic.

## Decision Log

- Decision: Promote operator hardening before new gameplay or chain-surface
  expansion.
  Rationale: the current bottleneck is safe repeatable operation by a second
  operator, not lack of another feature.
  Date/Author: 2026-03-30 / Codex

## Outcomes & Retrospective

Pending.

## Milestones

### Milestone 1: Define a real key-handling surface

Goal: replace the current "accepted stage-0 risk" posture with a concrete
shared key-management surface suitable for devnet/testnet operators.

Planned work:
- audit [031626-15-key-management.md](/home/r/coding/myosu/specs/031626-15-key-management.md) against the live repo
- decide the minimum `myosu-keys` stage-1 slice worth building now
- add the shared crate or equivalent operator-facing surface
- document the supported key lifecycle for miner, validator, and owner roles

Proof target:

    cargo check -p myosu-keys
    cargo test -p myosu-keys --quiet

### Milestone 2: Make named network packaging honest

Goal: make `devnet` / `testnet` surfaces reflect the current owned proof
instead of remaining thinner placeholders than `localnet`.

Planned work:
- audit `crates/myosu-chain/node/src/chain_spec/`
- decide which named network surfaces are worth keeping in this stage
- align `devnet` and any retained `testnet` path with the current runtime,
  pallet, and operator expectations
- add one honest proof path for the retained named-network surface

Proof target:

    cargo check -p myosu-chain --features fast-runtime
    rg -n "devnet|testnet|localnet" crates/myosu-chain/node/src/chain_spec

### Milestone 3: Publish an operator runbook

Goal: create a real operator-facing bring-up path for the current system that a
second engineer could follow without tribal knowledge.

Planned work:
- add or update docs under `docs/execution-playbooks/`
- cover node bring-up, miner/validator registration, key usage, and expected
  proof commands
- make the runbook line up with the current release gate and no-ship policy

Proof target:

    rg -n "operator|devnet|validator|miner|keys" docs/execution-playbooks README.md OS.md

## Plan of Work

1. Audit the key-management spec and the current chain-spec packaging against
   the live repo.
2. Choose the minimum operator-hardening slice that gives a second operator a
   safe path without overbuilding stage-1 infrastructure.
3. Implement the smallest real key surface and named-network packaging changes.
4. Publish the operator runbook and sync the control plane.
5. Extend the key-helper seam into a minimal persistent operator keystore
   without overbuilding a wallet product.
6. Wire the persistent keystore into miner and validator entrypoints without
   inventing a wallet UX.
7. Add the smallest account-creation CLI that makes the keystore usable without
   depending on a custom helper harness.
8. Add the smallest stored-account selection surface that avoids direct manual
   config editing.
9. Add the smallest encrypted-key portability surface that avoids exposing raw
   seed material.
10. Add the smallest recovery/import surface that avoids argv-level secret
    exposure.
11. Add the smallest password-rotation surface that avoids rebuild/reimport
    rituals.
12. Add the smallest bootstrap-print surface that turns the active account into
    concrete miner/validator startup commands.
13. Add the smallest operator-proof script that validates the printed
    bootstrap commands against live binaries and named build-spec output.
14. Add the smallest operator-bundle helper that writes reusable wrapper
    scripts from the active config without pretending we have a full launcher.

## Concrete Steps

From `/home/r/coding/myosu`:

    sed -n '1,220p' specs/031626-15-key-management.md
    ls crates/myosu-chain/node/src/chain_spec
    rg -n "devnet|testnet|localnet" crates/myosu-chain/node/src/chain_spec
    rg -n "operator|devnet|validator|miner|keys" docs/execution-playbooks README.md OS.md

Proof command:

    rg -n "devnet|testnet|localnet|myosu-keys|operator" \
      crates/myosu-chain/node/src/chain_spec \
      specs/031626-15-key-management.md \
      docs/execution-playbooks \
      README.md \
      OS.md

Additional current proof for the key-persistence slice:

    cargo check -p myosu-keys
    cargo test -p myosu-keys --quiet

Additional current proof for the operator-key consumer slice:

    SKIP_WASM_BUILD=1 cargo test -p myosu-keys -p myosu-miner -p myosu-validator --quiet
    SKIP_WASM_BUILD=1 cargo check -p myosu-keys -p myosu-miner -p myosu-validator

Additional current proof for the minimal CLI slice:

    export MYOSU_KEY_PASSWORD='replace-me'
    cargo run -p myosu-keys --quiet -- create --config-dir /tmp/myosu-keys-demo --network devnet
    cargo run -p myosu-keys --quiet -- show-active --config-dir /tmp/myosu-keys-demo

Additional current proof for the multi-account CLI slice:

    export MYOSU_KEY_PASSWORD='replace-me'
    cargo run -p myosu-keys --quiet -- create --config-dir /tmp/myosu-keys-demo --network devnet
    cargo run -p myosu-keys --quiet -- create --config-dir /tmp/myosu-keys-demo --network devnet
    cargo run -p myosu-keys --quiet -- list --config-dir /tmp/myosu-keys-demo
    cargo run -p myosu-keys --quiet -- switch-active --config-dir /tmp/myosu-keys-demo --address <ss58>
    cargo run -p myosu-keys --quiet -- show-active --config-dir /tmp/myosu-keys-demo

Additional current proof for the encrypted-keyfile portability slice:

    export MYOSU_KEY_PASSWORD='replace-me'
    cargo run -p myosu-keys --quiet -- create --config-dir /tmp/myosu-keys-src --network devnet
    cargo run -p myosu-keys --quiet -- export-active-keyfile --config-dir /tmp/myosu-keys-src --output /tmp/myosu-active.json
    cargo run -p myosu-keys --quiet -- import-keyfile --config-dir /tmp/myosu-keys-dst --source /tmp/myosu-active.json --network test_finney
    cargo run -p myosu-keys --quiet -- show-active --config-dir /tmp/myosu-keys-dst

Additional current proof for the env-var recovery slice:

    export MYOSU_KEY_PASSWORD='replace-me'
    export MYOSU_IMPORT_MNEMONIC='word1 ... word12'
    cargo run -p myosu-keys --quiet -- import-mnemonic --config-dir /tmp/myosu-keys-mnemonic --mnemonic-env MYOSU_IMPORT_MNEMONIC --password-env MYOSU_KEY_PASSWORD --network test_finney
    export MYOSU_IMPORT_RAW_SEED='0x1111111111111111111111111111111111111111111111111111111111111111'
    cargo run -p myosu-keys --quiet -- import-raw-seed --config-dir /tmp/myosu-keys-seed --seed-env MYOSU_IMPORT_RAW_SEED --password-env MYOSU_KEY_PASSWORD --network test_finney

Additional current proof for the password-rotation slice:

    export MYOSU_KEY_PASSWORD='replace-me'
    cargo run -p myosu-keys --quiet -- create --config-dir /tmp/myosu-keys-password --network devnet
    export MYOSU_OLD_PASSWORD='replace-me'
    export MYOSU_NEW_PASSWORD='replace-me-too'
    cargo run -p myosu-keys --quiet -- change-password --config-dir /tmp/myosu-keys-password --old-password-env MYOSU_OLD_PASSWORD --new-password-env MYOSU_NEW_PASSWORD

Additional current proof for the bootstrap-print slice:

    export MYOSU_KEY_PASSWORD='replace-me'
    cargo run -p myosu-keys --quiet -- create --config-dir /tmp/myosu-keys-bootstrap --network devnet
    cargo run -p myosu-keys --quiet -- print-bootstrap --config-dir /tmp/myosu-keys-bootstrap --subnet 7

Additional current proof for the operator-bootstrap smoke slice:

    bash .github/scripts/check_operator_network_bootstrap.sh

Additional current proof for the operator-bundle helper slice:

    export MYOSU_KEY_PASSWORD='replace-me'
    bash .github/scripts/prepare_operator_network_bundle.sh /tmp/myosu-operator-bundle
    /tmp/myosu-operator-bundle/verify-bundle.sh
    test -s /tmp/myosu-operator-bundle/devnet-spec.json
    test -s /tmp/myosu-operator-bundle/test-finney-spec.json
    test -s /tmp/myosu-operator-bundle/bundle-manifest.toml

Additional current proof for the hosted-operator-lane slice:

    rg -n "Operator Network|check_operator_network_bootstrap" .github/workflows/ci.yml

## Validation and Acceptance

Accepted when:

- the repo has a concrete shared key-handling surface instead of only a
  deferred-spec placeholder
- at least one named network surface beyond the owned local smoke is honest and
  directly operable
- the operator docs describe the current bring-up path without local-tribal
  assumptions
- `001`, `OS.md`, and the relevant `ops/` docs all promote this lane

## Idempotence and Recovery

This plan should proceed in narrow slices. If a larger key-management or
network-packaging scope appears, cut it back to the smallest operator-safe
surface instead of letting stage-1 work balloon.

## Interfaces and Dependencies

Depends on:
- `011` release governance
- `020` second-game execution proof

Likely touches:
- `crates/myosu-chain/node/src/chain_spec/`
- `crates/myosu-keys/` if created
- `docs/execution-playbooks/`
- `OS.md`
- `genesis/plans/001-master-plan.md`
