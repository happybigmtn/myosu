# Triage and Remediate Security Advisory Debt (SEC-001)

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds. This document must be maintained in accordance with root `PLANS.md`.

## Purpose / Big Picture

After this plan is complete, the current CI audit suppressions have been
triaged into three buckets: (a) remediated by dependency update, (b) accepted
as not-applicable to Myosu's usage, or (c) deferred with documented rationale
and timeline. `WORKLIST.md` currently tracks 12 active `SEC-001` items, while
the live CI workflow suppresses 19 advisory IDs total. The plan should
reconcile those two surfaces and shrink the allowlist where possible.

This plan runs in parallel with the promotion pipeline and does not block it. However, `bincode 1.3.3` (RUSTSEC-2025-0141) is in the wire codec path for all three dedicated game crates and should be prioritized.

## Requirements Trace

- R1: Each currently suppressed audit advisory is inventoried and classified as remediated, accepted, or deferred
- R2: Remediated advisories are removed from the CI allowlist
- R3: Accepted/deferred advisories have justification comments in the CI workflow
- R4: `bincode 1.3.3` advisory is specifically addressed (migrate to bincode 2.x or document acceptance)
- R5: `cargo audit -D warnings` passes with the updated allowlist
- R6: No existing tests regress

## Scope Boundaries

This plan triages security advisories. It does not upgrade the Polkadot SDK fork (that is CHAIN-SDK-001 in WORKLIST.md), does not change solver behavior, and does not change emission logic. If a dependency update requires code changes (e.g., bincode 2.x API migration), those changes are scoped to the minimum needed for the update.

## Progress

- [ ] Audit each of 12 advisories against Myosu's actual usage
- [ ] Classify `RUSTSEC-2025-0141` (bincode): remediate or accept?
- [ ] Classify `RUSTSEC-2024-0388` (derivative): remediate or accept?
- [ ] Classify `RUSTSEC-2025-0057` (fxhash): remediate or accept?
- [ ] Classify `RUSTSEC-2024-0384` (instant): inherited chain, accept?
- [ ] Classify `RUSTSEC-2020-0168` (mach): inherited chain, accept?
- [ ] Classify `RUSTSEC-2022-0061` (parity-wasm): inherited chain, accept?
- [ ] Classify `RUSTSEC-2024-0436` (paste): remediate or accept?
- [ ] Classify `RUSTSEC-2024-0370` (proc-macro-error): inherited chain, accept?
- [ ] Classify `RUSTSEC-2025-0010` (ring): inherited chain, accept?
- [ ] Classify `RUSTSEC-2021-0127` (serde_cbor): inherited chain, accept?
- [ ] Classify `RUSTSEC-2026-0002` (lru): remediate or accept?
- [ ] Classify `RUSTSEC-2024-0442` (wasmtime-jit-debug): inherited chain, accept?
- [ ] Inventory and classify the 7 additional carry-forward ignores already present in CI
- [ ] Update CI allowlist with justifications
- [ ] Verify `cargo audit -D warnings --ignore <updated-list>` passes
- [ ] Verify `cargo test --workspace --quiet` passes (SKIP_WASM_BUILD=1)

## Surprises & Discoveries

None yet.

## Decision Log

- Decision: Run this plan in parallel with the promotion pipeline rather than before it.
  Rationale: The 12 advisories are in inherited chain dependencies and game wire codecs. None directly affect the canonical policy bundle types or promotion manifest surface. Blocking promotion work on dependency triage would delay the master plan without improving local development safety.
  Date/Author: 2026-04-11 / genesis corpus

## Outcomes & Retrospective

None yet.

## Context and Orientation

The 12 active `SEC-001` advisories from `WORKLIST.md` are:

| Advisory | Crate | Source |
|----------|-------|--------|
| RUSTSEC-2025-0141 | bincode 1.3.3 | Direct dependency in kuhn, liars-dice, poker |
| RUSTSEC-2024-0388 | derivative | Inherited chain |
| RUSTSEC-2025-0057 | fxhash | Inherited chain |
| RUSTSEC-2024-0384 | instant | Inherited chain |
| RUSTSEC-2020-0168 | mach | Inherited chain |
| RUSTSEC-2022-0061 | parity-wasm | Inherited chain |
| RUSTSEC-2024-0436 | paste | Inherited chain or workspace |
| RUSTSEC-2024-0370 | proc-macro-error | Inherited chain |
| RUSTSEC-2025-0010 | ring | Inherited chain |
| RUSTSEC-2021-0127 | serde_cbor | Inherited chain |
| RUSTSEC-2026-0002 | lru | Workspace or inherited |
| RUSTSEC-2024-0442 | wasmtime-jit-debug | Inherited chain |

Plus 7 additional advisories already in the CI allowlist
(`RUSTSEC-2025-0009`, `2025-0055`, `2023-0091`, `2024-0438`,
`2025-0118`, `2026-0020`, `2026-0021`) that were added earlier.

The CI workflow is `.github/workflows/ci.yml`, job `dependency-audit`. The current allowlist is a long `--ignore` chain.

## Plan of Work

1. For each advisory, run `cargo audit` to check the current status and determine if the advisory is still active.

2. For `bincode` (RUSTSEC-2025-0141): Check if bincode 2.x is compatible with the existing wire codec usage in `myosu-games-kuhn/src/solver.rs`, `myosu-games-liars-dice/src/solver.rs`, and `myosu-games-poker/src/wire.rs`. If the migration is straightforward, do it. If not, document the acceptance with a rationale.

3. For inherited chain dependencies: Verify that the advisory is in a transitive dependency from `opentensor/polkadot-sdk` and that Myosu has no direct exposure. Document acceptance with "inherited from chain fork, no direct usage in Myosu-owned code."

4. Update `.github/workflows/ci.yml` `dependency-audit` job with the reduced allowlist and per-ignore justification comments.

## Implementation Units

### Unit 1: Bincode triage

Goal: Remediate or accept the bincode advisory.
Requirements advanced: R4.
Dependencies: None.
Files to modify: Potentially `Cargo.toml` (workspace), game crate `Cargo.toml` files, wire codec files.
Tests to add: Wire codec roundtrip tests if API changes.
Approach: Check bincode 2.x compatibility, migrate if feasible.
Test scenarios: All existing wire codec tests pass with updated bincode.

### Unit 2: Inherited chain advisory triage

Goal: Classify and document all inherited chain advisories.
Requirements advanced: R1, R2, R3.
Dependencies: None.
Files to modify: `.github/workflows/ci.yml`.
Tests to add: None.
Approach: Verify each advisory is transitive from polkadot-sdk. Document acceptance.
Test scenarios: `cargo audit -D warnings --ignore <list>` exits 0.

## Concrete Steps

Use the repo-root sequence below to inventory the current suppressions and then
prove the reconciled state.

## Verification

Run the commands below from the repo root. The important outcome is not merely
“audit still passes,” but “the documented allowlist and the live allowlist now
say the same thing.”

From the repository root:

    cargo audit 2>&1 | head -100
    cargo tree -i bincode
    cargo tree -i derivative
    cargo tree -i fxhash

Then after updates:

    cargo audit -D warnings --ignore <updated-list>
    SKIP_WASM_BUILD=1 cargo test --workspace --quiet

## Acceptance Criteria

- the live audit suppressions are fully inventoried
- the CI allowlist and `WORKLIST.md` no longer disagree silently
- any remaining ignore carries a justification
- direct `bincode` usage has an explicit remediation or acceptance decision

## Validation and Acceptance

1. `cargo audit -D warnings --ignore <list>` exits 0 with a shorter allowlist than the current one.
2. Each remaining `--ignore` has a justification comment in the CI workflow.
3. No existing tests regress.

## Idempotence and Recovery

Dependency updates are reversible via `cargo update` or `Cargo.lock` revert. If a bincode 2.x migration breaks wire compatibility, revert and accept the advisory with documentation.

## Artifacts and Notes

Expected modifications:
    .github/workflows/ci.yml (dependency-audit job)
    Cargo.toml (if bincode version changes)
    Cargo.lock (if dependencies update)
    Potentially game crate wire codec files

## Interfaces and Dependencies

The wire codec interface in each game crate must remain backward-compatible with existing checkpoint and query/response file formats. If bincode 2.x changes the serialization format, the migration must include a format version header or a compatibility path.

Revision note (2026-04-11 / Codex review): corrected the live suppressions
count, reconciled the plan with the current CI allowlist, and added explicit
verification/acceptance headings.
