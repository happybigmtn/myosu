# Specification: Security Audit Process

Source: Genesis Plan 005 M2-M4, ASSESSMENT.md security risks SR-01 through SR-06
Status: Draft
Depends-on: none

## Purpose

The project carries six identified security risks including inherited chain
vulnerabilities from the subtensor fork, robopoker fork drift, two unsafe blocks
for memory-mapped files, and no process for tracking upstream Substrate CVEs.
Without a reproducible security audit process, the project cannot assess its own
risk posture or communicate it to future operators. Establishing automated
dependency auditing, documenting unsafe code invariants, and creating a CVE
tracking process gives the project a defensible security baseline before external
operators join the network.

## Whole-System Goal

Current state: No dependency audit runs in CI. Two unsafe blocks exist in
`myosu-games-poker` for memory-mapped blueprint files with SAFETY comments but
no boundary tests. No process exists for learning about or evaluating upstream
Substrate security patches. No SECURITY.md or vulnerability disclosure process.

This spec adds: Automated dependency vulnerability scanning in CI, documented
safety invariants for all unsafe blocks, and a structured process for tracking
and evaluating upstream CVEs in Substrate and robopoker dependencies.

If all ACs land: CI fails on known dependency vulnerabilities, every unsafe
block has a documented invariant and failure mode, and an operator can follow a
documented process to evaluate whether an upstream security patch applies.

Still not solved here: Formal security audit by third party, HSM-based key
management, runtime upgrade security, and network-level threat modeling.

## Scope

In scope:
- Adding dependency vulnerability scanning to CI
- Documenting safety invariants for all existing unsafe blocks
- Creating a CVE tracking process for upstream Substrate and robopoker
  dependencies
- Publishing a SECURITY.md with vulnerability disclosure guidance

Out of scope:
- Third-party security audit engagement
- Hardware security module (HSM) integration
- Network-level security (peer authentication, transport encryption beyond
  Substrate defaults)
- Smart contract security (EVM is being removed per 001-chain-runtime-reduction)
- Penetration testing

## Current State

The workspace enforces `clippy::unwrap_used = "deny"` and
`clippy::expect_used = "warn"`, resulting in zero unwrap calls in active crates.
All unwrap calls reside in the inherited chain fork.

Two unsafe blocks exist in `crates/myosu-games-poker/src/codexpoker.rs` for
memory-mapped file access to blueprint artifacts. Both have SAFETY comments
explaining the invariant (read-only access to validated artifact files).

The Polkadot SDK dependency is pinned to a specific opentensor fork revision.
The robopoker dependency is pinned by git rev to the happybigmtn fork. Neither
has an automated check for upstream security advisories.

No SECURITY.md exists. No `cargo audit` or equivalent runs in CI.

## What Already Exists

| Sub-problem | Existing code / flow | Reuse / extend / replace | Why |
|---|---|---|---|
| Clippy strictness | Workspace `Cargo.toml` lint config | Reuse | Already prevents common safety issues |
| Unsafe blocks | `crates/myosu-games-poker/src/codexpoker.rs` | Extend | Add boundary test documentation |
| CI pipeline | `.github/workflows/ci.yml` (7 jobs) | Extend | Add audit job |
| Dependency pins | `Cargo.toml` workspace deps | Reuse | Pins are the audit input |
| Fork changelog | `docs/robopoker-fork-changelog.md` | Extend | Add security-relevant diff tracking |
| INV-006 | `INVARIANTS.md` robopoker fork coherence | Reuse | Governance already requires tracking |

## Non-goals

- Achieving zero known vulnerabilities in the inherited chain fork (the fork
  carries transitive dependencies that may have advisories unrelated to stage-0
  usage).
- Automated patching of upstream dependencies.
- Runtime binary signing or attestation.
- Auditing the Python research stack for security issues.

## Behaviors

The CI pipeline includes a dependency audit step that scans active workspace
crate dependencies for known vulnerabilities. The audit covers direct and
transitive dependencies of the active crates. When a known vulnerability is
found in an active crate dependency, CI fails. Inherited chain fork dependencies
may be excluded from the failure gate with documented justification.

Each unsafe block in the codebase has a SAFETY comment documenting: the
invariant that must hold, the failure mode if the invariant is violated, and the
boundary conditions that are tested. The unsafe inventory is enumerable by
searching the codebase.

A documented process describes how the project tracks upstream security
advisories for Substrate (via Polkadot SDK), robopoker, and other pinned
dependencies. The process specifies where to monitor, how to evaluate
applicability, and how to apply patches when needed.

A SECURITY.md file at the repository root describes how to report
vulnerabilities and what response to expect.

## Acceptance Criteria

- A dependency audit step in CI scans active crate dependencies and fails on
  known vulnerabilities.
- Every unsafe block in active crates has a SAFETY comment documenting the
  invariant, failure mode, and boundary conditions.
- A CVE tracking process document exists describing monitoring sources,
  evaluation criteria, and patch application steps for Substrate and robopoker
  dependencies.
- A SECURITY.md exists at the repository root with vulnerability disclosure
  guidance.
