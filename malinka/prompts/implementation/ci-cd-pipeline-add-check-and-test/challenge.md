# Add cargo check and cargo test for active crates Lane — Challenge

Perform a cheap adversarial review of the current slice for `ci-cd-pipeline-add-check-and-test` before the expensive final review runs.

Your job is to challenge assumptions, find obvious scope drift, identify weak proof, and catch mismatches between code and artifacts. Do not bless the slice as merge-ready; that belongs to the final review gate.


Verification artifact must cover
- summarize the automated proof commands that ran and their outcomes

Current Slice Contract:
Plan file:
- `genesis/plans/006-ci-cd-pipeline.md`

Child work item: `ci-cd-pipeline-add-check-and-test`

Full plan context (read this for domain knowledge, design decisions, and specifications):

# CI/CD Pipeline Setup

**Plan ID:** 006
**Status:** New
**Priority:** HIGH — no automated quality gates exist

`genesis/PLANS.md` at `genesis/PLANS.md` governs this document.

---

## Purpose / Big Picture

After this plan lands, every pull request to `trunk` will automatically run: `cargo check`, `cargo test`, `cargo clippy`, and `cargo fmt --check`. Failed PRs will block merge. This closes the gap where Fabro lanes can complete with passing proof commands that don't actually run the right tests.

---

## Progress

- [ ] Create `.github/workflows/ci.yml`
- [ ] Add `cargo check` for all workspace members
- [ ] Add `cargo test` for active workspace members (`myosu-games`, `myosu-tui`)
- [ ] Add `cargo clippy` with strict warnings
- [ ] Add `cargo fmt --check`
- [ ] Configure branch protection on `trunk`

---

## Surprises & Discoveries

*(To be written during implementation)*

---

## Decision Log

- Decision: CI runs on PR to `trunk`, not on every push.
  Rationale: The single-author workflow means frequent pushes. CI on PR only reduces noise while maintaining quality gates.
  Date/Author: 2026-03-21 / Interim CEO

- Decision: CI tests only `myosu-games` and `myosu-tui` initially — not the chain pallets.
  Rationale: Chain pallets require Substrate's full toolchain (heavy), and they're already well-tested. The user-facing crates need CI coverage most urgently.
  Date/Author: 2026-03-21 / Interim CEO

- Decision: `cargo clippy` runs with `-D warnings` to treat lint warnings as errors.
  Rationale: Current codebase has no lint discipline. Enforcing clippy prevents new warnings from accumulating.
  Date/Author: 2026-03-21 / Interim CEO

---

## Outcomes & Retrospective

*(To be written at plan completion)*

---

## Milestones

### M1: Create `.github/workflows/ci.yml`
A GitHub Actions workflow that triggers on PR to `trunk`.

Proof: `test -f .github/workflows/ci.yml`; workflow syntax is valid YAML.

### M2: Add `cargo check` and `cargo test` for active crates
The CI runs `cargo check -p myosu-games -p myosu-tui` and `cargo test -p myosu-games -p myosu-tui` on every PR.

Proof: GitHub Actions run shows both check and test steps pass on a PR.

### M3: Add `cargo clippy` with strict warnings
CI fails if any clippy warning exists (enforced via `-D warnings`).

Proof: A deliberately introduced clippy warning (e.g., `unused variable`) causes CI to fail.

### M4: Add `cargo fmt --check`
CI fails if code is not formatted according to `rustfmt.toml`.

Proof: `cargo fmt` without `--check` followed by PR shows diff; `cargo fmt --check` passes only after `cargo fmt`.

### M5: Configure branch protection on `trunk`
Require CI to pass before merge.

Proof: GitHub branch protection settings show `trunk` requires passing CI checks.

---

## Context and Orientation

Current CI state: **NONE**.

The `.github/` directory does not exist. No GitHub Actions, no GitLab CI, no Jenkins.

Key files to create:
- `.github/workflows/ci.yml` — main CI workflow
- `.github/CODEOWNERS` — code owners (optional)

Key files to modify:
- `Cargo.toml` — consider adding a `[profile.ci]` profile for faster builds
- (no other files needed — the workspace is already testable)

---

## Plan of Work

1. Create `.github/workflows/ci.yml` with the standard Rust CI template
2. Configure matrix for `myosu-games` and `myosu-tui` crates
3. Add clippy with `-D warnings`
4. Add fmt check
5. Configure branch protection on GitHub
6. Verify by opening a test PR

---

## Concrete Steps

```bash
# Verify no .github directory exists
ls -la .github 2>/dev/null || echo "No .github directory"

# Create directory structure
mkdir -p .github/workflows

# Create the CI workflow
cat > .github/workflows/ci.yml << 'EOF'
name: CI
on:
  pull_request:
    branches: [trunk, main]
  push:
    branches: [trunk, main]
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt, clippy
      - uses: Swatinem/rust-cache@v2
      - name: cargo check
        run: cargo check -p myosu-games -p myosu-tui
      - name: cargo test
        run: cargo test -p myosu-games -p myosu-tui
      - name: cargo clippy
        run: cargo clippy -p myosu-games -p myosu-tui -- -D warnings
      - name: cargo fmt
        run: cargo fmt --check
EOF

# Test locally (if act is installed)
# act -P ubuntu-latest=catthehacker/ubuntu:rust-latest
```

---

## Validation

- `test -f .github/workflows/ci.yml`
- `cargo fmt --check` passes (no formatting diff)
- `cargo clippy -p myosu-games -p myosu-tui -- -D warnings` passes (may require fixing existing warnings first)
- `cargo test -p myosu-games -p myosu-tui` passes
- GitHub Actions run on a test PR shows all steps passing


Workflow archetype: implement

Review profile: standard

Active plan:
- `genesis/plans/001-master-plan.md`

Active spec:
- `genesis/SPEC.md`

AC contract:
- Where: CI workflow steps for cargo check and cargo test
- How: Add CI steps that run cargo check and cargo test against myosu-games and myosu-tui crates
- Required tests: cargo check -p myosu-games -p myosu-tui && cargo test -p myosu-games -p myosu-tui
- Verification plan: Both cargo check and cargo test pass locally and show as steps in the workflow file
- Rollback condition: cargo check or cargo test fails on the active crates

Proof commands:
- `cargo check -p myosu-games -p myosu-tui`
- `cargo test -p myosu-games -p myosu-tui`

Artifacts to write:
- `spec.md`
- `review.md`

Challenge checklist:
- Is the slice smaller than the plan says, or larger?
- Did the implementation actually satisfy the first proof gate?
- Are any touched surfaces outside the named slice?
- Are the artifacts overstating completion?
- Is there an obvious bug, trust-boundary issue, or missing test the final reviewer should not have to rediscover?

Write a short challenge note in `verification.md` or amend it if needed, focusing on concrete gaps and the next fixup target. Do not write `promotion.md` here.
