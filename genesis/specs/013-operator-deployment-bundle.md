# Specification: Operator Deployment Bundle

Source: Reverse-engineered from .github/scripts/prepare_operator_network_bundle.sh, .github/scripts/check_operator_network_bootstrap.sh
Status: Draft
Depends-on: 009-key-management, 012-chain-node

## Purpose

The operator deployment bundle generates a self-contained directory of executable
scripts that allow a miner or validator operator to start their processes,
build chain specifications, and verify the bundle's integrity. It bridges the
gap between key management (identity creation) and operator process startup by
producing ready-to-run scripts with all configuration baked in.

The primary consumer is a new operator onboarding to the network who needs to go
from zero to running miner and validator processes.

## Whole-System Goal

Current state: The bundle generator is implemented as a shell script that
produces 5 executable scripts, 2 chain spec artifacts, a TOML manifest, and a
README. It is validated in CI via the operator-network job.

This spec adds: Nothing new. This documents the existing behavioral contract.

If all ACs land: A new operator can generate a bundle from their key
configuration and have a single directory containing everything needed to start
mining or validating on any configured network.

Still not solved here: Actual miner or validator operation, multi-node networking
setup, and monitoring are separate concerns.

## Scope

In scope:
- Bundle generation from key configuration and environment variables
- Generated scripts: start-miner.sh, start-validator.sh, build-devnet-spec.sh,
  build-test-finney-spec.sh, verify-bundle.sh
- Bundle manifest (TOML) tracking configuration metadata
- Generated README documenting bundle contents and usage
- Chain spec artifact generation (devnet and test-finney JSON files)
- Environment variable indirection for password security
- CI validation of the full bundle generation flow

Out of scope:
- Key generation (handled by myosu-keys)
- Actual miner or validator process behavior
- Multi-node network configuration
- Production deployment or cloud infrastructure
- Monitoring or alerting setup

## Current State

The bundle generator exists at .github/scripts/prepare_operator_network_bundle.sh.
It takes a bundle directory and optional config directory as arguments, reads
configuration from environment variables (MYOSU_OPERATOR_NETWORK,
MYOSU_OPERATOR_CHAIN, MYOSU_OPERATOR_SUBNET, MYOSU_OPERATOR_PASSWORD_ENV), and
produces a complete bundle directory.

If no config.toml exists in the config directory, the generator runs myosu-keys
create to generate a fresh identity. It then runs print-bootstrap to extract the
active address and network, and generates all scripts and artifacts.

Each generated start script sets up the environment, validates the password
environment variable is set, and runs the corresponding cargo binary with all
necessary flags. Chain spec scripts build specs using myosu-chain's build-spec
command with the fast-runtime feature.

The bundle manifest (TOML, format version 1) records: bundle directory, repo
root, config directory, active address, network, chain endpoint, subnet, password
environment variable, script paths, and artifact paths.

The verify script checks manifest existence, script executability, and spec
validity as a self-contained integrity check.

CI validation in the operator-network job creates a temporary environment,
generates keys, produces a bundle, verifies all files are present and
executable, runs verify-bundle.sh, and validates chain spec generation
independently.

## What Already Exists

| Sub-problem | Existing code / flow | Reuse / extend / replace | Why |
|---|---|---|---|
| Bundle generation | prepare_operator_network_bundle.sh | Reuse | Produces complete operator package |
| CI validation | check_operator_network_bootstrap.sh | Reuse | End-to-end bundle verification |
| Start scripts | Generated miner and validator launchers | Reuse | Password-safe process startup |
| Chain spec scripts | Generated devnet and test-finney builders | Reuse | Artifact generation |
| Bundle verification | verify-bundle.sh self-check | Reuse | Integrity validation |
| Manifest | TOML format version 1 | Reuse | Configuration tracking |

## Non-goals

- Providing a daemon manager or process supervisor.
- Supporting Docker or container-based deployment.
- Managing multi-operator or multi-node bundle coordination.
- Implementing automatic updates or version management.
- Providing monitoring, alerting, or log aggregation.

## Behaviors

The bundle generator validates that the password environment variable is
exported and non-empty. It creates the bundle and config directories if needed.

If no operator configuration exists, the generator invokes myosu-keys create to
generate a fresh sr25519 identity with encrypted persistence. It then runs
print-bootstrap to extract the active address and network.

Five executable scripts are generated: start-miner.sh runs myosu-miner with
chain endpoint, subnet, key config directory, and password environment variable
flags. start-validator.sh follows the same pattern for myosu-validator. Both
scripts validate that the password environment variable is set before launching.
build-devnet-spec.sh and build-test-finney-spec.sh invoke myosu-chain build-spec
with the fast-runtime feature and output JSON to the bundle directory.
verify-bundle.sh checks that the manifest exists, all scripts are executable,
and chain specs can be built.

The manifest records all configuration in a structured TOML file for downstream
tooling or audit.

The README documents the bundle source, usage instructions, and lists all
materialized artifacts.

After generating scripts, the generator makes all scripts executable, invokes
both chain spec builders to materialize the devnet and test-finney spec
artifacts, and outputs a summary.

## Acceptance Criteria

- The bundle generator produces all 5 executable scripts, 2 chain spec
  artifacts, a manifest, and a README.
- Generated start scripts refuse to run when the password environment variable
  is unset.
- Generated start scripts pass the correct flags to myosu-miner and
  myosu-validator.
- Chain spec scripts produce valid JSON chain spec files.
- The verify script detects missing or non-executable files.
- The bundle manifest accurately records all configuration metadata.
- The bundle can be generated from a fresh config directory (key creation is
  triggered automatically).
- CI validation succeeds end-to-end: key generation, bootstrap, bundle
  generation, file verification, and self-check.
