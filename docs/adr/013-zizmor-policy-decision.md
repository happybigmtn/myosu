# ADR 013: zizmor policy direction for `dtolnay/rust-toolchain` superfluous-actions findings

- Status: Accepted
- Date: 2026-06-04
- Deciders: codexworker (CI-SEC-001 row in `IMPLEMENTATION_PLAN.md`)
- Consulted: `ops/decision_log.md`, `docs/adr/README.md`, `WORKLIST.md` CI-SEC-001 follow-up
- Informed: myosu operators, future CI contributors
- Related: `.github/zizmor.yml`, `.github/workflows/ci.yml`, `tests/e2e/zizmor_policy.sh`, plan rows `SEC-001` (cargo-audit allowlist) and `CI-SEC-001` (this row)

## Context

`SEC-001` in `IMPLEMENTATION_PLAN.md` shipped cargo-audit dependency
scanning to CI in commits `36b9460`, `2dca9a3`, and `fb5995a`, and the
SEC-001 family of gates is currently the dependency-audit CI job plus
the two e2e harnesses `sec001_allowlist_consistency.sh` and
`sec001_allowlist_staleness_probe.sh`. The `WORKLIST.md` follow-up
`CI-SEC-001` notes that the repo still emits raw zizmor `superfluous-actions`
advisories against `dtolnay/rust-toolchain` (the canonical Rust
toolchain pin action used in 7 places in `.github/workflows/ci.yml`).

`zizmor .github/workflows/ci.yml` reports the following on a clean
checkout before this change:

- 1 HIGH `unpinned-uses` finding on line 458 (the developer-quickstart
  job's `actions/checkout@de0fac2e4500dabe0009e67214f5f5447ce83dd # v6`
  reference — a single-nibble typo `5xxx` instead of the real
  `4fxx`, returning HTTP 404 on github.com so zizmor cannot resolve
  the action reference at all).
- 7 informational `superfluous-actions` findings on
  `dtolnay/rust-toolchain@29eef336d9b2848a0b548edc03f92a220660cdb8 # stable`
  (one per job that installs the toolchain: repo-shape, active-crates,
  chain-core, dependency-audit, developer-quickstart, operator-network,
  chain-clippy).

`superfluous-actions` is a low-confidence informational audit: the
underlying claim is that `rustup` is preinstalled on `ubuntu-latest`
runners and so `dtolnay/rust-toolchain` is "redundant." The audit
notebook's confidence band is "Medium" for this rule, and the audit
itself recommends the `rustup` alternative only for jobs that do not
also use a Rust-aware cache action.

The repo already uses `Swatinem/rust-cache` in 5 of the 7 affected
jobs (active-crates, developer-quickstart, operator-network,
chain-clippy, and one variant of the dependency-audit job), and
`Swatinem/rust-cache` keys its `CARGO_HOME` hash off the
`dtolnay/rust-toolchain` action's toolchain selection, not the
default `~/.cargo` location rustup uses. Replacing the action with
a raw `rustup` script would either force us to pin rustup globally
to a known version (and lose the cache key coherence) or
hand-roll a `curl https://sh.rustup.rs | sh` chain that zizmor's
`artipacked` and `template-injection` audits routinely flag.

The 1 HIGH `unpinned-uses` finding on line 458 is a real bug: the
typo'd SHA returns HTTP 404 on github.com, so the action reference
could not be resolved even if zizmor did not flag it. The other 11
`actions/checkout` references in `ci.yml` use the real v6 release
SHA `de0fac2e4500dabe0009e67214ff5f5447ce83dd` (verified 200 on
github.com/actions/checkout/commit/...). The developer-quickstart
job is the only place that has the typo, introduced in commit
`36b9460` (the DX-001 developer-quickstart consolidation).

If the project did nothing:

- The 1 HIGH `unpinned-uses` finding would silently fail the
  developer-quickstart job in production (zizmor would block the
  PR; even without zizmor, the GitHub Actions runner would error
  out trying to resolve the action reference).
- The 7 informational `superfluous-actions` findings would remain
  a raw advisory with no documented rationale for why the action
  is still used. Future contributors would have no signal that
  the choice is intentional, and a well-meaning cleanup PR could
  swap the action for a raw `rustup` script and break the
  `Swatinem/rust-cache` cache key coherence without realizing it.

## Decision

We **carry an explicit zizmor policy file** (`.github/zizmor.yml`)
that per-line silences the 7 `dtolnay/rust-toolchain` `superfluous-actions`
findings (one per job, all in `ci.yml`) with a one-line `# reason:`
comment above each rule, and we **fix the phantom-SHA typo** on
`actions/checkout` line 458. A new `zizmor-policy` CI job installs
zizmor and runs `tests/e2e/zizmor_policy.sh` on every PR and push to
`trunk` / `main`. The proof harness asserts all four acceptance
criteria (zizmor exits 0 with 0 findings, every ignore rule has a
`# reason:` justification, the phantom-SHA bug is fixed, the same
zizmor invocation with `--no-config` still emits >=7 informational
findings so the policy is verified to be the silencer, not a
stale-cache accident).

The alternative — **swap `dtolnay/rust-toolchain` for a raw `rustup`
script step** in all 7 jobs — is **rejected** because:

- The `Swatinem/rust-cache` action's `CARGO_HOME` cache key is
  computed from the toolchain it sees at install time. The cache
  action deliberately detects the `dtolnay/rust-toolchain` action
  and keys off its pinned `channel` field; replacing the action
  with a raw `rustup` script forces the cache action to fall back
  to the default `~/.cargo` location, which is a different key
  namespace and would invalidate the existing cache for every
  contributor on the next run.
- The 7 `superfluous-actions` findings are low-confidence
  informational and the underlying rustup preinstall claim is
  correct, but `dtolnay/rust-toolchain` adds two real pieces of
  value on top of rustup: a single-line pin to a specific stable
  release (so a runner base-image bump cannot silently change the
  Rust toolchain) and a `components: clippy` / `targets: ...` knob
  that the `rustup` install script would have to replicate in
  bash. Both pieces of value are exactly the kind of single-line,
  fully-pinned action that zizmor's `unpinned-uses` audit is happy
  with.
- Replacing 7 instances of a 1-line action with 7 instances of a
  multi-line `curl ... | sh` chain expands the attack surface for
  the chain-build path and creates 7 new places where zizmor's
  `artipacked` and `template-injection` audits would fire.

The chosen direction also subsumes the long-standing
`SEC-001`-family CI hygiene: by CI-gating zizmor and verifying the
allowlist, every new zizmor finding is loud (a failing CI job) and
every ignored finding is explicit and reviewable (a `# reason:`
comment in `.github/zizmor.yml`).

## Consequences

- `tests/e2e/zizmor_policy.sh` is the new executable gate for
  CI-SEC-001. It is wired into the new `zizmor-policy` CI job in
  `.github/workflows/ci.yml`.
- `.github/zizmor.yml` is the single source of truth for
  per-finding zizmor suppressions. New ignore rules must come with
  a `# reason:` justification directly above the rule, enforced by
  the proof harness (rule_count == `# reason:` comment count).
- The phantom-SHA bug on `actions/checkout` line 458 is fixed. All
  13 (was 12) checkout references now use the real v6 release SHA.
- A future zizmor release that adds a new audit against
  `dtolnay/rust-toolchain` would fail the zizmor-policy CI job
  until either the audit is silenced with a documented reason or
  the workflow is updated; this is the intended loud-failure mode.
- The `SEC-001` row in `IMPLEMENTATION_PLAN.md` already documents
  the cargo-audit allowlist shape; the new `CI-SEC-001` row
  documents the zizmor allowlist shape and cross-references this
  ADR. `WORKLIST.md`'s CI-SEC-001 entry is updated to mark the
  direction chosen.

## Alternatives considered

1. **Disable the `superfluous-actions` audit entirely.**
   Rejected: zizmor's own documentation warns that disabled audits
   do not appear in ignored/suppressed counts, making it easy to
   miss important new findings. A blanket disable would also
   silence any future legitimate `superfluous-actions` finding
   (e.g., a CI job that genuinely should not need a third-party
   action at all). Per-line ignores keep the audit on for the rest
   of the file.

2. **Switch all 7 jobs to `dtolnay/rust-toolchain` with a `+nightly`
   or `+stable` matrix and call it "intentional" without a policy
   file.** Rejected: the policy file is the auditable artifact;
   a comment in the workflow is not the same as a structured
   rule that zizmor can read.

3. **Add a `--no-audit superfluous-actions` flag to the
   `zizmor-policy` CI job's command line.** Rejected: that hides
   the rule from the developer-facing output (a `zizmor` run from
   a developer shell would still emit the 7 findings) and
   contradicts the chosen direction of "make every suppression
   auditable."

4. **Move the toolchain install into a single reusable composite
   action and call it once.** Rejected: the 7 jobs are otherwise
   heterogeneous (different cache knobs, different `targets:`,
   different `components:`, different `env:`). A composite action
   would have to expose all of those knobs and would be more
   surface area to review than 7 single-line action references.

## Decision Log

- 2026-06-04: ADR-013 accepted; `.github/zizmor.yml` and the
  `zizmor-policy` CI job land in the same commit as the
  phantom-SHA fix on `actions/checkout` line 458 and the
  `CI-SEC-001` `[x]` row in `IMPLEMENTATION_PLAN.md`.
