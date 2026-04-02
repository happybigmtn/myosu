# Specification: Operator Onboarding

Source: Genesis Plan 009 (Operator Onboarding and Documentation)
Status: Draft
Depends-on: 006-multi-node-devnet

## Purpose

Before external operators can run miners and validators on the devnet, they need
documentation that takes them from zero familiarity to a running setup. The
current operator experience requires reading scattered doctrine files, inferring
setup steps from CI scripts, and understanding Substrate concepts without
guidance. A structured onboarding path with a quickstart guide, architecture
explanation, and troubleshooting reference reduces the barrier to entry for the
first external operators joining the network.

## Whole-System Goal

Current state: `myosu-keys print-bootstrap` generates formatted miner and
validator startup commands (CI-verified). The operator bundle contains startup
scripts, chain specs, and a self-verification script. Execution playbooks in
`docs/execution-playbooks/` are partially stale. No quickstart guide, no
architecture overview for non-developers, and no troubleshooting FAQ exist.

This spec adds: A structured operator guide covering the complete path from
prerequisites through running miner and validator on the devnet, with
architecture explanation accessible to non-developers and troubleshooting for
common failure modes.

If all ACs land: A new operator with Rust toolchain experience but no myosu
familiarity can go from zero to running a miner and validator on the devnet by
following the guide, and can diagnose the most common failure modes without
external help.

Still not solved here: Video walkthroughs, interactive tutorials, operator
community infrastructure, and SLA or support commitments.

## Scope

In scope:
- Quickstart guide: prerequisites through running miner and validator
- Architecture overview accessible to technical non-developers
- Troubleshooting guide for common failure modes
- Ensuring every command in the guide works when copy-pasted

Out of scope:
- Non-technical marketing or pitch documentation
- Video or multimedia content
- Automated setup scripts beyond what the operator bundle provides
- Operator community infrastructure (forums, chat, support channels)
- Documentation for chain development or protocol modification

## Current State

The operator bootstrap command `myosu-keys print-bootstrap` generates formatted
instructions for starting a miner and validator. This is CI-verified by the
`operator-network` job.

The operator bundle produced by CI contains: `start-miner.sh`,
`start-validator.sh`, chain spec generators, `verify-bundle.sh`,
`bundle-manifest.toml`, a README, and pre-built chain specs. The bundle README
provides basic instructions but assumes familiarity with the project.

Execution playbooks in `docs/execution-playbooks/` describe operational
procedures but some reference outdated file paths and surfaces. The assessment
identified these as partially stale.

Architecture documentation exists in `SPEC.md`, `THEORY.MD`, `AGENTS.md`, and
`OS.md`, but is written for the protocol engineer, not for an external operator.

## What Already Exists

| Sub-problem | Existing code / flow | Reuse / extend / replace | Why |
|---|---|---|---|
| Bootstrap command generation | `myosu-keys print-bootstrap` | Reuse | Already generates startup instructions |
| Operator bundle | `.github/scripts/prepare_operator_network_bundle.sh` | Reuse | Contains scripts, specs, verification |
| Bundle README | Operator bundle `README.md` | Extend | Add quickstart context |
| Architecture docs | `SPEC.md`, `THEORY.MD`, `AGENTS.md` | Reference | Source material for operator-facing overview |
| Execution playbooks | `docs/execution-playbooks/` | Replace | Partially stale; need fresh content |
| CI verification | `operator-network` CI job | Reuse | Proves bundle is functional |

## Non-goals

- Writing documentation for protocol developers or contributors.
- Creating an automated installer or one-click setup.
- Providing support SLAs or response time commitments.
- Documenting the Python research stack for operators.
- Writing chain governance or upgrade documentation (see 008-release-governance).

## Behaviors

The quickstart guide walks an operator through: installing prerequisites (Rust
toolchain, system dependencies), building the binaries, creating keys with
`myosu-keys`, starting a chain node connected to the devnet, registering and
running a miner, and registering and running a validator. Every command in the
guide is copy-pasteable and produces the expected output when run on a supported
platform.

The architecture overview explains the four-layer system (chain, miners,
validators, gameplay) using diagrams and plain language accessible to someone
with general technical literacy but no Substrate or game theory background.
The overview explains what each component does, why it exists, and how they
interact — without implementation details.

The troubleshooting guide documents the most common failure modes an operator
encounters during setup and operation: build failures, chain connection issues,
registration problems, miner/validator startup errors, and scoring
discrepancies. Each entry describes the symptom, likely cause, and resolution.

The documentation builds on existing artifacts — particularly the output of
`myosu-keys print-bootstrap` and the operator bundle — rather than duplicating
their content.

## Acceptance Criteria

- A quickstart guide enables an operator to go from zero to running a miner and
  validator on the devnet without external assistance.
- Every command in the quickstart guide works when copy-pasted on a fresh
  checkout.
- An architecture overview explains the four-layer system in terms accessible
  to a technical non-developer.
- A troubleshooting guide covers at least 10 common failure modes with
  symptom, cause, and resolution.
