# Specification: CI Verification Pipeline

Source: Reverse-engineered from .github/workflows/ci.yml, .github/scripts/check_stage0_repo_shape.sh, check_doctrine_integrity.sh, check_plan_quality.sh
Status: Draft
Depends-on: none

## Purpose

The CI verification pipeline gates code quality, structural integrity, and
operational readiness across the full Myosu workspace. It runs on every push
and pull request to trunk/main, validating that the repository maintains its
stage-0 shape, active crates compile and pass tests, chain core functions
correctly, doctrine documents are internally consistent, genesis plans meet
quality standards, and the operator network bundle generates successfully.

The primary consumer is the development process itself: the pipeline prevents
regressions from landing on trunk.

## Whole-System Goal

Current state: The pipeline is implemented with 7 parallel CI jobs (after the
foundation repo-shape check), covering workspace structure, active crate health,
chain core, doctrine integrity, plan quality, operator bootstrap, and chain
linting.

This spec adds: Nothing new. This documents the existing behavioral contract.

If all ACs land: No commit can land on trunk that breaks workspace structure,
crate compilation, test suites, linting, doctrine integrity, plan quality, or
operator bundle generation.

Still not solved here: Runtime verification of stage-0 exit criteria
(cross-validator determinism, emission distribution) requires live chain testing
beyond CI scope.

## Scope

In scope:
- 7 CI jobs with dependency ordering (repo-shape is foundation)
- Repo shape validation: required files and workspace members
- Active crate validation: compile, focused smoke tests, full test suite,
  clippy, rustfmt
- Chain core validation: runtime check, pallet tests, runtime tests
- Doctrine integrity: master index consistency, no orphaned or empty specs
- Plan quality: milestone headings and proof commands in genesis plans
- Operator network: key generation, bootstrap, bundle generation, verification
- Chain clippy: strict linting on runtime, pallet, and node

Out of scope:
- Runtime integration testing (live chain required)
- Performance benchmarking
- Security scanning or vulnerability auditing
- Deployment to staging or production
- Multi-node network testing

## Current State

The pipeline exists at .github/workflows/ci.yml with 7 jobs triggered on
pushes and PRs to trunk/main branches. Concurrency control cancels in-progress
runs for the same branch.

The repo-shape job is the foundation: all other jobs depend on it. It validates
that 8 crate Cargo.toml files exist, that specific genesis plans and spec files
are present, and that the workspace Cargo.toml lists all required members.

The active-crates job compiles 9 crates with SKIP_WASM_BUILD=1, runs focused
smoke tests (serialization roundtrip for myosu-games, shell state for myosu-tui,
preflop-to-flop progression for myosu-play, binary smoke-test), runs the full
test suite, checks clippy with -D warnings, and verifies rustfmt compliance
(edition 2024).

The chain-core job installs protobuf-compiler, then checks runtime (with
fast-runtime feature), pallet, and node compilation. It runs pallet
stage-0-specific tests and the runtime test suite.

The doctrine job runs check_doctrine_integrity.sh, which verifies: the master
index file exists, all 031626-*.md spec files are indexed, all indexed specs
exist as files, and no spec file is empty.

The plan-quality job runs check_plan_quality.sh, which finds all genesis plans
matching 0[0-2][0-9]-*.md (excluding 001-*) and verifies each has at least one
milestone heading and at least one proof command section.

The operator-network job runs check_operator_network_bootstrap.sh, which
generates a temporary key, produces bootstrap commands, validates their content,
generates an operator bundle, verifies all bundle files, and runs the bundle's
self-check.

The chain-clippy job runs clippy with -D warnings on runtime, pallet, and node
targets.

## What Already Exists

| Sub-problem | Existing code / flow | Reuse / extend / replace | Why |
|---|---|---|---|
| Repo shape | check_stage0_repo_shape.sh | Reuse | Validates workspace structure |
| Crate health | 9-crate compile/test/lint pipeline | Reuse | Comprehensive crate validation |
| Chain core | Runtime + pallet + node checks and tests | Reuse | Chain-specific validation |
| Doctrine | check_doctrine_integrity.sh | Reuse | Spec consistency enforcement |
| Plan quality | check_plan_quality.sh | Reuse | Genesis plan structure enforcement |
| Operator bootstrap | check_operator_network_bootstrap.sh | Reuse | End-to-end bundle validation |
| Chain lint | Clippy -D warnings on chain crates | Reuse | Strict chain code quality |

## Non-goals

- Providing continuous deployment or release automation.
- Running live chain integration tests.
- Benchmarking runtime performance or weight calibration.
- Scanning for security vulnerabilities in dependencies.
- Managing environment-specific configuration or secrets.

## Behaviors

On every push or PR to trunk/main, the pipeline starts the repo-shape job. This
job fails fast if required files are missing or workspace members are
unregistered, preventing all downstream jobs from running on a malformed
workspace.

Once repo-shape passes, 6 jobs run in parallel: active-crates, chain-core,
doctrine, plan-quality, operator-network, and chain-clippy.

The active-crates job uses SKIP_WASM_BUILD=1 to avoid WASM compilation overhead.
It first runs cargo check on all 9 crates, then focused smoke tests that
validate specific behavioral properties (serialization round-trip, shell state
machine, poker game progression, binary smoke-test mode). After smoke tests, it
runs the full test suite, clippy with deny-warnings, and rustfmt.

The chain-core job requires protobuf-compiler for SCALE codec generation. It
checks compilation of runtime, pallet, and node, then runs pallet-specific
stage-0 tests and the runtime test suite separately.

The doctrine job validates bidirectional consistency between the master index and
spec files on disk. It uses ripgrep when available and falls back to grep. Any
mismatch (extra files, missing indexed specs, empty specs) fails the job.

The plan-quality job finds genesis plans by number pattern, excluding the master
plan (001-*), and verifies structural requirements. Missing milestone headings
or proof command sections fail the job with the specific plan path.

The operator-network job performs a complete round-trip: create keys, generate
bootstrap commands, validate command content (config directory and password
environment variable flags), test command help output, generate a full operator
bundle, verify all bundle files exist and are executable, and run the bundle's
self-verification script.

## Acceptance Criteria

- A commit missing a required crate Cargo.toml file fails the repo-shape job.
- A commit with a missing workspace member in root Cargo.toml fails the
  repo-shape job.
- A commit with a compilation error in any active crate fails the active-crates
  job.
- A commit with a failing test in any active crate fails the active-crates job.
- A commit with a clippy warning (deny level) fails either active-crates or
  chain-clippy.
- A commit with a rustfmt deviation fails the active-crates job.
- A commit with an orphaned or missing spec fails the doctrine job.
- A commit with a genesis plan missing milestones or proof commands fails the
  plan-quality job.
- A commit that breaks operator bundle generation fails the operator-network
  job.
- All 6 parallel jobs can run concurrently after repo-shape passes.
