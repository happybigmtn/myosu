# Specification: Operator Tooling

Status: Draft
Depends-on: None (builds on existing surfaces)

## Objective

Define the tooling surfaces that operators use to manage keys, bootstrap nodes,
and deploy miners and validators. This spec codifies the current state of
`myosu-keys`, the operator bundle, environment variable contracts, and
deployment scripts -- then identifies the gaps that block a reliable
first-operator experience on fresh machines.

The goal: an operator with Rust toolchain experience runs a deterministic
sequence of commands to go from zero to a functioning miner+validator pair,
with key management that never exposes secrets on the command line.

## Evidence Status

All facts below are verified against code as of 2026-04-05.

### myosu-keys (crates/myosu-keys/)

Commands: `create`, `show-active`, `list`, `switch-active`, `import-keyfile`,
`import-mnemonic`, `import-raw-seed`, `export-active-keyfile`,
`print-bootstrap`, `change-password`.

Password input: `MYOSU_KEY_PASSWORD` environment variable. Never accepted as a
CLI argument. This is the correct design -- secrets in argv are visible in
`/proc` and shell history.

Network awareness: commands accept `--network` with values `devnet` and
`testnet`. Key storage is namespaced by network under the config directory.

Config directory: `~/.myosu` by default, overridable via `MYOSU_CONFIG_DIR`.

### Environment variables

| Variable | Default | Purpose |
|---|---|---|
| `MYOSU_KEY_PASSWORD` | (none, required) | Decrypts operator keyfile |
| `MYOSU_CONFIG_DIR` | `~/.myosu` | Root for keys, config, state |
| `MYOSU_SUBNET` | `7` | Target subnet ID |
| `MYOSU_WORKDIR` | (none) | Working directory for miner/validator artifacts |
| `MYOSU_CHAIN` | `ws://127.0.0.1:9955` | Chain WebSocket endpoint |
| `MYOSU_OPERATOR_CHAIN` | (none) | Operator-specific chain override |

### Operator quickstart flow (docs/operator-guide/quickstart.md, 265 lines)

The documented sequence:

1. Build required surfaces (chain node, miner, validator, keys binary)
2. Create operator key via `myosu-keys create`
3. Materialize operator bundle via `prepare_operator_network_bundle.sh`
4. Bring up chain connection (local authority-backed devnet or shared devnet)
5. Bootstrap miner: register on-chain, serve axon, stage training artifacts
6. Bootstrap validator: register on-chain, acquire permit, stake
7. Start miner HTTP server
8. Run validator scoring loop

### Operator bundle

Produced by `.github/scripts/prepare_operator_network_bundle.sh`. Contains
startup scripts (`start-miner.sh`, `start-validator.sh`), chain spec
generators, `verify-bundle.sh`, `bundle-manifest.toml`, and a README.

Fresh-machine proof: `.github/scripts/check_operator_network_fresh_machine.sh`
now runs the bundle inside Ubuntu 22.04 from a bare container, installs the
required toolchain and packages, funds a generated operator key on the local
authority-backed `devnet`, and verifies the miner + validator bootstrap flow
end to end.

### Chain specs

- `devnet`: custom four-authority devnet with subnet `7` bootstrapped in
  genesis and named operator accounts endowed; the quickstart local path often
  launches only `authority-1`, which means blocks land on that path about once
  every 48 seconds
- `test_finney`: multi-authority, used for multi-node testing

### E2E scripts

`local_loop.sh`, `validator_determinism.sh`, `emission_flow.sh`,
`two_node_sync.sh` -- all exercised in CI.

### Architecture (docs/operator-guide/architecture.md, 194 lines)

Component model: `myosu-keys` feeds into registration and staking transactions,
which connect `myosu-miner` and `myosu-validator` through `myosu-chain`.

Data distribution:
- On-chain: subnet IDs, registration, axon endpoints, stake, weights, emissions
- Miner disk: checkpoints, encoder artifacts
- Validator disk: scoring inputs
- `~/.myosu/`: encrypted operator keys

## Acceptance Criteria

### Key management

- `myosu-keys create` produces a network-namespaced encrypted keyfile under
   `$MYOSU_CONFIG_DIR` without requiring any CLI secret arguments.
- `myosu-keys print-bootstrap` emits copy-pasteable miner and validator
   startup commands that reference the active key.
- Key import paths (`import-keyfile`, `import-mnemonic`, `import-raw-seed`)
   each produce a keyfile identical in format to `create`.
- `change-password` re-encrypts the active keyfile without creating a new
   keypair.

### Bootstrap

- The operator bundle produced by `prepare_operator_network_bundle.sh`
   contains all artifacts needed to start a miner and validator against devnet
   without additional downloads.
- `verify-bundle.sh` exits 0 when run inside a correctly produced bundle and
   exits non-zero with a diagnostic message when any artifact is missing.
- The quickstart sequence (steps 1-8 above) completes on a fresh Ubuntu 22.04
   machine with only Rust toolchain pre-installed, without undocumented manual
   steps.

### Deployment surfaces

- Each environment variable in the table above is documented in the operator
   guide with its type, default, and effect.
- `start-miner.sh` and `start-validator.sh` fail fast with actionable error
   messages when required environment variables are unset.
- Chain spec selection (`devnet` vs `test_finney`) is controlled by a single
    flag or environment variable, not by editing files.

### Gaps to close

- First-run guidance: `myosu-keys create` prints next-step instructions when
    no keyfile exists yet (or the quickstart guide covers this explicitly).
- No secrets appear in process listings, shell history, or log output during
    any operator workflow.

## Verification

### Automated (CI)

- `operator-network` job: builds the bundle, runs `verify-bundle.sh`, executes
  `myosu-keys print-bootstrap`, confirms output parses, and runs the Ubuntu
  22.04 fresh-machine operator proof.
- E2E scripts (`local_loop.sh`, `emission_flow.sh`) exercise the full
  miner-chain-validator loop.
- Key management unit tests in `crates/myosu-keys/` cover create, import,
  export, switch, and password change.

### Manual

- Fresh-machine test: run the quickstart on a clean Ubuntu 22.04 VM (or
  container) with only `rustup` pre-installed. Document any step that fails or
  requires undocumented intervention.
- Secret hygiene audit: run the full quickstart while monitoring `/proc/*/cmdline`
  and shell history for leaked secrets.

### Not yet covered

- No Docker/container packaging exists. The operator bundle assumes a bare-metal
  or VM environment with Rust toolchain.
- No systemd units are shipped in the bundle (though `deploy-bootnode.sh`
  generates one for bootnode deployments).
- Multi-node devnet setup requires manual bootnode coordination; no turnkey
  multi-node script exists for operators.

## Open Questions

1. Should the operator bundle include pre-built binaries, or is
   build-from-source the only supported path? Pre-built binaries reduce setup
   time but add a cross-compilation and trust problem.

2. What is the target for "fresh machine bootstrap time"? The current quickstart
   involves a full Rust compilation which can take 15-30 minutes depending on
   hardware. Is this acceptable for the first-operator cohort?

3. Should `myosu-keys` support hardware wallet (Ledger/Trezor) signing, or is
   encrypted-keyfile-on-disk the only supported key storage for the foreseeable
   future?

4. Is `MYOSU_OPERATOR_CHAIN` intended to diverge from `MYOSU_CHAIN` long-term,
   or is it a temporary shim? If temporary, when does it get removed?

5. Should the operator bundle include a systemd unit template for miner and
   validator, given that `deploy-bootnode.sh` already generates one for the
   bootnode?
