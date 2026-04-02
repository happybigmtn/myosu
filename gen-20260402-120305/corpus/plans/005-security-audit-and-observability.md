# 005 - Security Audit and Observability

## Purpose / Big Picture

The project has identified 6 security risks (SR-01 through SR-06 in
ASSESSMENT.md) and has no unified observability. This plan addresses both:
audit the inherited chain fork surface, establish tracing across all binaries,
and create a process for tracking upstream Substrate CVEs.

Prior plan 011 (Security Observability Release) was a stub. This plan replaces
it with concrete milestones.

## Context and Orientation

Security findings from assessment:
- SR-01: Inherited chain vulnerabilities from subtensor fork (Medium/High)
- SR-02: robopoker fork drift (Low/High)
- SR-05: No process for tracking upstream Substrate CVEs (Medium/High)

Observability gaps:
- `myosu-miner` and `myosu-validator` use `tracing` crate
- `myosu-play` and `myosu-games-*` use print-based logging
- No structured log format, no metrics, no alerting

## Architecture

```
All binaries
    → tracing subscriber (fmt layer + optional JSON)
    → env filter (RUST_LOG)
    → optional file output

Security tracking
    → SECURITY.md (disclosure policy)
    → ops/upstream-cve-tracking.md (process doc)
    → Dependabot or cargo-audit in CI
```

## Progress

### Milestone 1: Unified tracing across all binaries

- [ ] M1. Add tracing subscriber to myosu-play, consistent with miner/validator
  - Surfaces: `crates/myosu-play/src/main.rs`, `crates/myosu-tui/src/shell.rs`
  - What exists after: All three binaries use `tracing` with env filter.
    `RUST_LOG=myosu_play=debug` works for play as it does for miner/validator.
  - Why now: Cannot debug production issues without structured logging.
Proof command: `SKIP_WASM_BUILD=1 RUST_LOG=myosu_play=debug cargo run -p myosu-play --quiet -- --smoke-test 2>&1 | head -5`
  - Tests: Smoke test still passes with tracing enabled

### Milestone 2: Add cargo-audit to CI

- [ ] M2. Add `cargo audit` step to CI pipeline
  - Surfaces: `.github/workflows/ci.yml`
  - What exists after: CI fails if any active crate dependency has a known CVE.
  - Why now: Inherited fork may carry vulnerable transitive dependencies.
Proof command: `cargo audit --deny warnings`
  - Tests: CI job passes

### Milestone 3: Document upstream CVE tracking process

- [ ] M3. Write process for monitoring and applying Substrate SDK security patches
  - Surfaces: `ops/upstream-cve-tracking.md` (new), `SECURITY.md` (new)
  - What exists after: Clear process for how to learn about, evaluate, and apply
    upstream patches to the pinned Substrate fork.
  - Why now: The fork will drift further from upstream. Without a process,
    security patches will be missed.
Proof command: `test -s ops/upstream-cve-tracking.md && test -s SECURITY.md`
  - Tests: Files exist and are non-empty

### Milestone 4: Audit unsafe code and memory-mapped files

- [ ] M4. Review all unsafe blocks and document safety invariants
  - Surfaces: `crates/myosu-games-poker/src/codexpoker.rs`
  - What exists after: Each unsafe block has a SAFETY comment explaining the
    invariant that makes it safe, the failure mode if violated, and a test that
    exercises the boundary.
  - Why now: 2 unsafe blocks exist for mmap. Must verify before production.
Proof command: `rg "unsafe" crates/ --type rust -l`
  - Tests: `cargo test -p myosu-games-poker --quiet -- mmap`

## Surprises & Discoveries

- The miner/validator tracing setup is nearly identical (`init_tracing()` in both
  `main.rs` files). Could extract to a shared function in `myosu-chain-client`
  or a new `myosu-common` crate, but premature until a third binary needs it.

## Decision Log

- Decision: `cargo audit` in CI rather than manual review.
  - Why: Automated is better than manual for dependency vulnerability tracking.
  - Failure mode: False positives blocking CI.
  - Mitigation: Allow ignoring specific advisories with justification comments.
  - Reversible: yes

- Decision: Tracing (not metrics/Prometheus) for observability v1.
  - Why: Tracing is already partially adopted. Metrics require a collection
    stack. Start with what exists.
  - Failure mode: Structured logs insufficient for production debugging.
  - Mitigation: Can add metrics layer to tracing subscriber later.
  - Reversible: yes

## Validation and Acceptance

1. All three binaries produce structured log output via `tracing`.
2. `cargo audit` runs in CI and passes.
3. `SECURITY.md` and CVE tracking process exist.
4. All unsafe blocks documented with safety invariants.

## Outcomes & Retrospective
_Updated after milestones complete._
